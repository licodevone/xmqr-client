#!/usr/bin/env python3
"""C147: duas rotas mTLS reais, QoS mistos e diagnóstico sem conexões."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import ssl
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
from verify_bridge_brokers import BRIDGE, CLIENT, command, private, ready_subscriber, wait_markers


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def exercise(broker_root, rows):
    sys.path.insert(0, str(broker_root / 'scripts'))
    from verify_multi_endpoint import free_port, hash_fixture, wait_port
    from verify_wss_mtls import certificates
    broker = broker_root / 'target/debug/mqtt-broker'
    admin = broker_root / 'target/debug/mqtt-admin'
    processes = []
    with tempfile.TemporaryDirectory(prefix='xmqr-c147-') as directory:
        root = Path(directory)
        root.chmod(0o700)
        certificates(root)
        secret = secrets.token_urlsafe(18)
        private(root / 'password', secret + '\n')
        fingerprint = hashlib.sha256(ssl.PEM_cert_to_DER_cert(
            (root / 'client.crt').read_text())).hexdigest()
        password_hash = hash_fixture(admin, secret)
        private(root / 'users.toml', f'[[users]]\nusername="device"\n'
                f'password_hash="{password_hash}"\ncert_sha256="sha256:{fingerprint}"\n')
        source, target = free_port(), free_port()
        assert source != target
        state = root / 'bridge-state'
        state.mkdir(mode=0o700)
        routes = [('temperatura', 'sensors/', 'mirror/', 1),
                  ('umidade', 'metrics/', 'archive/', 2)]
        topology = 'version=1\n'
        for name, port in [('source', source), ('target', target)]:
            topology += f'[[brokers]]\nid="{name}"\nhost="127.0.0.1"\nport={port}\n'
        for name, origin, destination, qos in routes:
            topology += (f'[[links]]\nid="{name}"\nsource_broker="source"\n'
                         f'destination_broker="target"\nsource_prefix="{origin}"\n'
                         f'target_prefix="{destination}"\nqos={qos}\n')
        private(root / 'topology.toml', topology)
        text = (f'version=2\nsource_protocol="mqtt5"\nlegacy_metadata_policy="drop"\nmax_retries=8\n'
                f'topology_file={json.dumps(str(root / "topology.toml"))}\n'
                f'state_dir={json.dumps(str(state))}\n')
        for name, port in [('source', source), ('destination', target)]:
            identity = 'source' if name == 'source' else 'target'
            text += (f'[{name}]\nbroker_id="{identity}"\nhost="127.0.0.1"\n'
                     f'port={port}\nclient_id="c147-{identity}"\nusername="device"\n')
            for field, filename in [('password_file', 'password'), ('ca', 'ca.crt'),
                                    ('cert', 'client.crt'), ('key', 'client.key')]:
                text += f'{field}={json.dumps(str(root / filename))}\n'
        for name, origin, destination, qos in routes:
            text += (f'[[routes]]\nid="{name}"\nsource_prefix="{origin}"\n'
                     f'target_prefix="{destination}"\nqos={qos}\n')
        config = root / 'bridge.toml'
        private(config, text)

        def passed(name):
            rows.append(dict(name=name, status='PASS'))

        def check(contents=text, expected=0, alternate=False):
            private(config, contents)
            args = ['--config', str(config)]
            args = args + ['--check'] if alternate else ['--check'] + args
            result = subprocess.run([str(BRIDGE)] + args, capture_output=True, timeout=8)
            assert result.returncode == expected, (result.returncode, result.stderr[-1200:])
            assert secret.encode() not in result.stdout + result.stderr
            return result

        traps = []
        try:
            for port in [source, target]:
                trap = socket.socket()
                trap.bind(('127.0.0.1', port))
                trap.listen()
                trap.setblocking(False)
                traps.append(trap)
            check()
            check(alternate=True)
            assert list(state.iterdir()) == []
            for trap in traps:
                try:
                    connection, _ = trap.accept()
                except BlockingIOError:
                    continue
                connection.close()
                raise AssertionError('--check conectou ao broker')
            passed('check_duas_ordens_sem_conexao_ou_mutacao')
        finally:
            for trap in traps:
                trap.close()

        negatives = {
            'sem_politica_descarte_metadata': text.replace('legacy_metadata_policy="drop"\n', ''),
            'politica_descarte_metadata_desconhecida': text.replace('legacy_metadata_policy="drop"', 'legacy_metadata_policy="ignore"'),
            'id_duplicado': text.replace('id="umidade"', 'id="temperatura"'),
            'prefixo_sobreposto': text.replace('source_prefix="metrics/"', 'source_prefix="sensors/sala/"'),
            'identidade_longa': text.replace('client_id="c147-source"', 'client_id="' + 'a' * 128 + '"'),
            'rota_ausente_manifesto': text.replace('id="umidade"', 'id="ausente"'),
            'mistura_route_routes': text + '\n[route]\nsource_prefix="a/"\ntarget_prefix="b/"\n',
            'sem_rotas': text.split('[[routes]]')[0],
            'mais_de_oito': text + text[text.index('[[routes]]'):] * 4,
            'sem_estado_qos2': '\n'.join(line for line in text.splitlines() if not line.startswith('state_dir=')),
        }
        for name, contents in negatives.items():
            check(contents, 2)
            assert list(state.iterdir()) == []
            passed(name)
        state.chmod(0o777)
        check(expected=2)
        state.chmod(0o700)
        passed('estado_permissivo_recusado')
        (state / 'umidade').symlink_to(root / 'nao-existe', target_is_directory=True)
        check(expected=2)
        (state / 'umidade').unlink()
        passed('symlink_pendente_recusado')
        (root / 'topology.toml').chmod(0o644)
        check(expected=2)
        (root / 'topology.toml').chmod(0o600)
        passed('manifesto_permissivo_recusado')
        private(config, text)
        config.chmod(0o644)
        result = subprocess.run([str(BRIDGE), '--check', '--config', str(config)],
                                capture_output=True, timeout=8)
        assert result.returncode == 2
        config.chmod(0o600)
        passed('configuracao_permissiva_recusada')

        base = {key: value for key, value in os.environ.items() if not key.startswith('MQTT_')}
        base.update(MQTT_MODE='secure-mtls', MQTT_SERVER_CERT=str(root / 'server.crt'),
                    MQTT_SERVER_KEY=str(root / 'server.key'), MQTT_CLIENT_CA=str(root / 'ca.crt'),
                    MQTT_USERS_FILE=str(root / 'users.toml'))
        def start_bridge():
            child = subprocess.Popen([str(BRIDGE), '--config', str(config)],
                                     stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            processes.append(child)
            wait_markers(child, ['origem assinada (rota temperatura)', 'origem assinada (rota umidade)'])
            return child

        def deliver(origin, destination, qos):
            payload = f'c147-{origin}-{qos}-{secrets.token_hex(6)}'
            subscriber = ready_subscriber(root, target, destination)
            processes.append(subscriber)
            result = subprocess.run(command(root, source, 'pub', origin, payload, qos=qos),
                                    capture_output=True, timeout=10)
            assert result.returncode == 0, result.stderr[-1200:]
            stdout, stderr = subscriber.communicate(timeout=10)
            assert subscriber.returncode == 0 and payload.encode() in stdout, (stdout, stderr)

        try:
            for index, (port, publish, subscribe) in enumerate([
                (source, ['sensors/value', 'metrics/value', 'sensors/denied'], ['sensors/#', 'metrics/#']),
                (target, ['mirror/value', 'archive/value'],
                 ['mirror/#', 'archive/#', 'mirror/value', 'archive/value'])]):
                acl = root / f'acl-{index}.toml'
                private(acl, '[[rules]]\nusername="device"\n'
                        f'publish={json.dumps(publish)}\nsubscribe={json.dumps(subscribe)}\n')
                env = dict(base, MQTT_BIND=f'127.0.0.1:{port}',
                           MQTT_STATE_DIR=str(root / f'state-{index}'), MQTT_ACL_FILE=str(acl))
                with (root / f'broker-{index}.log').open('ab') as log:
                    child = subprocess.Popen([str(broker)], env=env, stdout=log, stderr=log)
                processes.append(child)
                wait_port(child, port)
            bridge = start_bridge()
            for origin, destination, qos_values in [
                ('sensors/value', 'mirror/value', [0, 1]),
                ('metrics/value', 'archive/value', [0, 1, 2])]:
                for qos in qos_values:
                    deliver(origin, destination, qos)
                    assert bridge.poll() is None
                    passed(f'encaminhamento_{origin}_qos{qos}')
            journal = state / 'umidade/bridge-qos2.json'
            assert journal.is_file() and not (state / 'temperatura').exists()
            document = json.loads(journal.read_text())
            assert all(row['pkid'] != 0 for row in document['pending'])
            identity = document['identity_sha256']
            passed('journal_qos2_isolado_sem_pid_zero')
            bridge.terminate()
            bridge.communicate(timeout=8)
            bridge = start_bridge()
            deliver('sensors/value', 'mirror/value', 1)
            deliver('metrics/value', 'archive/value', 2)
            assert json.loads(journal.read_text())['identity_sha256'] == identity
            passed('reinicio_preserva_identidades_e_duas_rotas')
            bridge.terminate()
            bridge.communicate(timeout=8)
            journal.write_bytes(b'{corrompido')
            failed = subprocess.run([str(BRIDGE), '--config', str(config)],
                                    capture_output=True, timeout=10)
            assert failed.returncode == 1 and journal.read_bytes() == b'{corrompido'
            passed('worker_fatal_encerra_grupo_preserva_corrupcao')
        finally:
            for child in reversed(processes):
                if child.poll() is None:
                    child.send_signal(signal.SIGTERM)
                    try:
                        child.communicate(timeout=8)
                    except subprocess.TimeoutExpired:
                        child.kill()
                        child.communicate(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--broker-root', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    if args.report.exists() or args.report.is_symlink():
        parser.error('use novo relatório; falhas anteriores devem ser preservadas')
    report = dict(prompt='C147', binary_sha256=digest(BRIDGE), client_sha256=digest(CLIENT),
                  harness_sha256=digest(Path(__file__)), scenarios=[])
    began = time.monotonic()
    try:
        exercise(args.broker_root.resolve(), report['scenarios'])
    except Exception as error:
        report['scenarios'].append(dict(name='execucao', status='FAIL', error=str(error)[:1800]))
    report['binary_unchanged'] = digest(BRIDGE) == report['binary_sha256']
    report['counts'] = {status: sum(row['status'] == status for row in report['scenarios'])
                        for status in ['PASS', 'FAIL']}
    report['status'] = 'PASS' if not report['counts']['FAIL'] and report['binary_unchanged'] else 'FAIL'
    report['elapsed_seconds'] = round(time.monotonic() - began, 3)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2)
    print(json.dumps(report, indent=2))
    return int(report['status'] != 'PASS')


if __name__ == '__main__':
    raise SystemExit(main())
