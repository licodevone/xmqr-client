#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C148: retomada e armazenamento em recursos mTLS descartáveis próprios."""
import argparse
from contextlib import contextmanager
import json
import hashlib
import os
from pathlib import Path
import signal
import select
import sys
import tempfile
import time

from verify_bridge_v5_wire import (Case, Fixture, acknowledgement, digest,
                                   packet, publish, read_ack, read_publish,
                                   require, quiet, exact)


def checkpoint(case, name):
    deadline = time.monotonic() + 6
    while time.monotonic() < deadline:
        if case.log.exists() and f'checkpoint {name}'.encode() in case.log.read_bytes():
            return
        require(case.child.poll() is None, 'processo terminou antes do checkpoint')
        time.sleep(.02)
    raise AssertionError('checkpoint não observado no prazo')


@contextmanager
def owned(fixture, name, qos=1):
    case = Case(fixture, name, route_qos=qos)
    try:
        yield case
    finally:
        case.close()


def first_source_refusal(fixture):
    with owned(fixture, 'first-source-refusal') as case:
        case.launch()
        case.destination_connect()
        case.source_peer.connect('c148-origin-route', reason=0x87)
        # A sessão do destino foi aceita por este processo na tentativa anterior.
        case.destination_connect(session=True)
        case.source_connect()
        require(case.child.poll() is None, 'retentativa própria foi confundida com sessão desconhecida')
        require(case.document()['destination_known'], 'aceite do destino não foi persistido')


def graceful_busy(fixture, stop_signal=signal.SIGINT):
    name = signal.Signals(stop_signal).name
    with owned(fixture, 'graceful-during-durable-step-' + name.lower()) as case:
        case.start(checkpoint='after-source-persist')
        case.source.sendall(publish(qos=1))
        checkpoint(case, 'after-source-persist')
        case.phase('prepared')
        began = time.monotonic()
        case.child.send_signal(stop_signal)
        require(case.child.wait(timeout=4) == 0, name + ' não encerrou com sucesso')
        require(time.monotonic() - began < 4, 'encerramento excedeu o limite')
        require(case.document()['pending']['phase'] == 'prepared', 'encerramento perdeu obrigação durável')


def permission_contract(fixture):
    for mode in [0o400, 0o700]:
        with owned(fixture, f'permissions-{mode:o}') as case:
            case.start()
            case.stop(crash=True)
            case.journal.chmod(mode)
            previous = case.journal.read_bytes()
            case.launch()
            require(case.child.wait(timeout=4) != 0, 'journal fora de0600 foi aceito')
            require(case.journal.read_bytes() == previous, 'recusa de modo alterou o arquivo')


def durable_discards(fixture):
    with owned(fixture, 'durable-discard-counters') as case:
        case.start()
        case.source.sendall(publish(qos=1, props=[(2, 0)]))
        case.discarded()
        case.source.sendall(publish(qos=1, pkid=38,
                                   props=[(0x26, ('xmqr.bridge.path', 'wire-c148'))]))
        case.discarded(pkid=38)
        document = case.document()
        require(document.get('discarded_expired') == 1, 'descarte expiry sem contador durável')
        require(document.get('discarded_path') == 1, 'descarte de percurso sem contador durável')
        case.stop(crash=True)
        case.start(destination_session=True, source_session=True)
        require(case.document()['discarded_expired'] == 1 and
                case.document()['discarded_path'] == 1, 'reinício perdeu contagem de descartes')


def expiry_before_enqueue(fixture):
    with owned(fixture, 'expiry-before-enqueue') as case:
        case.start(checkpoint='before-destination-publish')
        case.source.sendall(publish(qos=1, props=[(2, 2)]))
        checkpoint(case, 'before-destination-publish')
        deadline = time.monotonic() + 35
        while time.monotonic() < deadline:
            readable, _, _ = select.select([case.source, case.destination], [], [], .2)
            for stream in readable:
                stream.settimeout(3)
                header = exact(stream, 1)[0]
                remaining = 0
                for index in range(4):
                    byte = exact(stream, 1)[0]
                    remaining += (byte & 127) << (7 * index)
                    if byte < 128:
                        break
                else:
                    raise AssertionError('Remaining Length inválido')
                body = exact(stream, remaining)
                if header == 0xC0:
                    require(not body, 'PINGREQ inválido')
                    stream.sendall(b'\xd0\x00')
                    continue
                require(not (stream is case.destination and header >> 4 == 3),
                        'prazo expirou durante commit/checkpoint mas novo PUBLISH foi enviado')
                if stream is case.source and header == 0x40:
                    require(int.from_bytes(body[:2], 'big') == 37, 'ACK de descarte divergente')
                    case.wait_journal(lambda d: d['pending'] is None and d['discarded_expired'] == 1)
                    quiet(case.destination)
                    return
        raise AssertionError('descarte pós-prazo não foi confirmado')


def recover_pre_publish(fixture, name, phase):
    with owned(fixture, 'recover-' + name, qos=2) as case:
        case.start(checkpoint=name)
        case.source.sendall(publish(qos=2, retain=True, payload=b'recovery-sample', props=[(2, 12)]))
        checkpoint(case, name)
        case.phase(phase)
        quiet(case.source)
        quiet(case.destination)
        case.stop(crash=True)
        time.sleep(1.05)
        case.start(destination_session=True, source_session=True)
        message = read_publish(case.destination)
        require(message['payload'] == b'recovery-sample' and message['retain'], 'replay alterou conteúdo/retained')
        require(message['duplicate'] == (phase == 'sent'), 'DUP não corresponde à fase persistida')
        expiry = next(value for key, value in message['properties'] if key == 2)
        require(0 < expiry <= 10, 'replay renovou o prazo')
        case.finish(2)


def recover_destination_rel(fixture):
    with owned(fixture, 'recover-destination-rel', qos=2) as case:
        case.start(checkpoint='before-destination-pubrel')
        case.source.sendall(publish(qos=2))
        read_publish(case.destination)
        case.destination.sendall(acknowledgement(0x50, 1))
        checkpoint(case, 'before-destination-pubrel')
        case.phase('rel')
        quiet(case.source)
        quiet(case.destination)
        case.stop(crash=True)
        case.start(destination_session=True, source_session=True)
        read_ack(case.destination, 0x60, 1)
        case.destination.sendall(acknowledgement(0x70, 1))
        read_ack(case.source, 0x50, 37)
        case.source.sendall(acknowledgement(0x60, 37))
        read_ack(case.source, 0x70, 37)
        case.wait_journal(lambda document: document['pending'] is None)
        quiet(case.destination)


def recover_after_destination_ack(fixture):
    with owned(fixture, 'recover-destination-acked') as case:
        case.start(checkpoint='after-destination-ack')
        case.source.sendall(publish(qos=1))
        read_publish(case.destination)
        case.destination.sendall(acknowledgement(0x40, 1))
        checkpoint(case, 'after-destination-ack')
        case.phase('done')
        quiet(case.source)
        case.stop(crash=True)
        case.start(destination_session=True, source_session=True)
        read_ack(case.source, 0x40, 37)
        case.wait_journal(lambda document: document['pending'] is None)
        quiet(case.destination)


def recover_source_qos2(fixture, name):
    with owned(fixture, 'recover-' + name, qos=2) as case:
        case.start(checkpoint=name)
        case.source.sendall(publish(qos=2))
        read_publish(case.destination)
        case.destination.sendall(acknowledgement(0x50, 1))
        read_ack(case.destination, 0x60, 1)
        case.destination.sendall(acknowledgement(0x70, 1))
        read_ack(case.source, 0x50, 37)
        if name == 'before-source-pubcomp':
            case.source.sendall(acknowledgement(0x60, 37))
        checkpoint(case, name)
        case.stop(crash=True)
        case.start(destination_session=True, source_session=True)
        if name == 'after-source-pubrec':
            read_ack(case.source, 0x50, 37)
        case.source.sendall(acknowledgement(0x60, 37))
        read_ack(case.source, 0x70, 37)
        case.wait_journal(lambda document: document['pending'] is None)
        quiet(case.destination)


def stored_negatives(fixture):
    mutations = ['corrupt', 'version', 'old_version', 'identity', 'clock', 'oversize', 'legacy', 'symlink']
    for name in mutations:
        with owned(fixture, 'stored-' + name) as case:
            case.start()
            case.stop(crash=True)
            if name == 'legacy':
                (case.journal.parent / 'bridge-qos2.json').write_text('{}')
            elif name == 'symlink':
                original = case.root / 'preserved-original.json'
                case.journal.rename(original)
                case.journal.symlink_to(original)
            elif name == 'oversize':
                case.journal.write_bytes(b'x' * (2 * 1024 * 1024 + 1))
            elif name == 'corrupt':
                case.journal.write_bytes(b'{invalid')
            else:
                container = json.loads(case.journal.read_text())
                if name in ['version', 'old_version']:
                    container['version'] = 99 if name == 'version' else 1
                elif name == 'identity':
                    container['document']['identity'] = '0' * 64
                else:
                    container['document']['saved_ms'] += 86400000
                serialized = json.dumps(container['document'], separators=(',', ':'), ensure_ascii=False).encode()
                container['checksum_sha256'] = hashlib.sha256(serialized).hexdigest()
                case.journal.write_text(json.dumps(container))
            previous = case.journal.read_bytes()
            case.launch()
            require(case.child.wait(timeout=4) != 0, 'journal inválido foi aceito')
            require(case.journal.read_bytes() == previous, 'journal inválido foi modificado')


def exclusive_lock(fixture):
    import subprocess
    with owned(fixture, 'exclusive-lock') as case:
        case.start()
        before = case.journal.read_bytes()
        result = subprocess.run([str(fixture.bridge), '--config', str(case.config)],
                                capture_output=True, timeout=5)
        require(result.returncode != 0, 'segundo escritor aceitou o lock ativo')
        require(case.journal.read_bytes() == before, 'segundo escritor modificou journal ativo')
        case.source.sendall(publish(qos=1))
        read_publish(case.destination)
        case.finish(1)


def discard_counter_overflow(fixture):
    with owned(fixture, 'counter-overflow') as case:
        case.start()
        case.stop(crash=True)
        container = json.loads(case.journal.read_text())
        container['document']['discarded_expired'] = 2**64 - 1
        serialized = json.dumps(container['document'], separators=(',', ':'), ensure_ascii=False).encode()
        container['checksum_sha256'] = hashlib.sha256(serialized).hexdigest()
        case.journal.write_text(json.dumps(container))
        case.start(destination_session=True, source_session=True)
        case.source.sendall(publish(qos=1, props=[(2, 0)]))
        case.failed('prepared')
        require(case.document()['discarded_expired'] == 2**64 - 1, 'overflow alterou contagem durável')


def refusal_survives_restart(fixture):
    for kind in [0x40, 0x50, 0x70]:
        with owned(fixture, f'refusal-restart-{kind:x}', qos=1 if kind == 0x40 else 2) as case:
            case.start()
            case.source.sendall(publish(qos=case.qos))
            read_publish(case.destination)
            if kind == 0x70:
                case.destination.sendall(acknowledgement(0x50, 1))
                read_ack(case.destination, 0x60, 1)
            case.destination.sendall(acknowledgement(kind, 1, 0x87 if kind != 0x70 else 0x92))
            case.failed()
            require(case.document()['pending']['phase'] == 'refused', 'recusa remota não foi registrada duravelmente')
            previous = case.journal.read_bytes()
            case.launch()
            require(case.child.wait(timeout=3) != 0, 'reinício tentou reenviar publicação recusada')
            require(case.journal.read_bytes() == previous, 'reinício alterou a obrigação recusada')
            for peer in [case.source_peer, case.destination_peer]:
                peer.listener.settimeout(.15)
                try:
                    connection, _ = peer.listener.accept()
                except TimeoutError:
                    continue
                connection.close()
                raise AssertionError('reabertura de recusa iniciou conexão MQTT')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bridge', type=Path, required=True)
    parser.add_argument('--broker-root', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--audit-only', action='store_true')
    parser.add_argument('--expiry-only', action='store_true')
    parser.add_argument('--negative-only', action='store_true')
    args = parser.parse_args()
    args.bridge = args.bridge.resolve(strict=True)
    sys.path.insert(0, str(args.broker_root.resolve() / 'scripts'))
    from verify_wss_mtls import certificates
    # Criar o relatório de forma exclusiva antes de usar recursos temporários.
    with args.report.open('x', encoding='utf-8') as report_file:
        report = dict(status='FAIL', prompt='C148', bridge_sha256=digest(args.bridge),
                      fixture_sha256=digest(Path(__file__)), scenarios=[],
                      scope='Administração de processos e peers mTLS próprios; queda de processo não simula power loss')
        scenarios = [('retentativa_apos_recusa_primeira_origem', first_source_refusal),
                     ('sigint_durante_etapa_assincrona', graceful_busy),
                     ('sigterm_durante_etapa_assincrona', lambda f: graceful_busy(f, signal.SIGTERM)),
                     ('permissoes_exatas_0600', permission_contract),
                     ('contadores_descarte_duraveis', durable_discards)]
        if not args.audit_only:
            scenarios += [('queda_prepared', lambda f: recover_pre_publish(f, 'after-source-persist', 'prepared')),
                          ('queda_sent_antes_envio', lambda f: recover_pre_publish(f, 'before-destination-publish', 'sent')),
                          ('queda_destino_pubrel', recover_destination_rel),
                          ('queda_apos_ack_destino', recover_after_destination_ack),
                          ('queda_apos_pubrec_origem', lambda f: recover_source_qos2(f, 'after-source-pubrec')),
                          ('queda_antes_pubcomp_origem', lambda f: recover_source_qos2(f, 'before-source-pubcomp')),
                          ('estado_invalido_preservado', stored_negatives),
                          ('lock_exclusivo_sem_afetar_escritor', exclusive_lock),
                          ('overflow_descarte_sem_ack', discard_counter_overflow)]
        scenarios.append(('expiry_recalculado_antes_enqueue', expiry_before_enqueue))
        if args.expiry_only:
            scenarios = scenarios[-1:]
        elif args.negative_only:
            scenarios = [('recusa_remota_preservada_sem_reenvio', refusal_survives_restart)]
        else:
            scenarios.append(('recusa_remota_preservada_sem_reenvio', refusal_survives_restart))
        try:
            with tempfile.TemporaryDirectory(prefix='xmqr-c148-recovery-', dir='/var/tmp') as directory:
                root = Path(directory)
                root.chmod(0o700)
                certificates(root)
                fixture = Fixture(args.bridge, root)
                for name, action in scenarios:
                    row = dict(name=name, status='FAIL')
                    try:
                        action(fixture)
                        row['status'] = 'PASS'
                    except Exception as error:
                        row['error_type'] = type(error).__name__
                        if isinstance(error, AssertionError) or type(error).__name__ == 'CheckError':
                            row['reason'] = str(error)[:180]
                    report['scenarios'].append(row)
                    print(json.dumps(row, ensure_ascii=False), flush=True)
            report['status'] = 'PASS' if all(row['status'] == 'PASS' for row in report['scenarios']) else 'FAIL'
        finally:
            report['bridge_unchanged'] = digest(args.bridge) == report['bridge_sha256']
            if not report['bridge_unchanged']:
                report['status'] = 'FAIL'
            json.dump(report, report_file, indent=2, ensure_ascii=False)
        return int(report['status'] != 'PASS')


if __name__ == '__main__':
    raise SystemExit(main())
