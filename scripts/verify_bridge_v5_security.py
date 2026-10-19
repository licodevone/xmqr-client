#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C149: direct MQTT5 bridge TLS/ACL negatives and one-flight QoS0 bound.

Linux/WSL, explicit ELF paths, private disposable brokers and wire peers only.
No fixture password, certificate key, application payload or raw log is reported.
"""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace

sys.dont_write_bytecode = True
from verify_bridge_v5_brokers import (Lab, client_environment, digest, markers,
                                      private, stop)
from verify_bridge_v5_wire import (Case, Cursor, field, packet, publish, quiet,
                                   read_publish, receive, require)


def replace_section(path, name, changes):
    text = path.read_text()
    start = text.index('[' + name + ']\n')
    end = text.find('\n[', start + 1)
    end = len(text) if end < 0 else end
    section = text[start:end]
    for field, value in changes.items():
        lines = section.splitlines()
        require(any(line.startswith(field + '=') for line in lines), 'fixture field absent')
        section = '\n'.join(field + '=' + json.dumps(value) if line.startswith(field + '=')
                            else line for line in lines) + '\n'
    private(path, text[:start] + section.rstrip('\n') + text[end:])


def checked_launch(lab, config):
    checked = subprocess.run([str(lab.args.bridge), '--check', '--config', str(config['config'])],
                             capture_output=True, timeout=8, env=client_environment())
    require(checked.returncode == 0, 'security fixture rejected statically instead of at runtime')
    require(not list(config['state'].iterdir()), 'static check mutated state')
    return lab.spawn([lab.args.bridge, '--config', config['config']], stdin=subprocess.DEVNULL,
                     stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def failed_output(lab, child):
    output, errors = child.communicate(timeout=18)
    require(child.returncode != 0, 'runtime security refusal exited successfully')
    require(lab.secret.encode() not in output + errors, 'fixture credential leaked in diagnostics')
    require(len(output) + len(errors) <= 65536, 'security diagnostic exceeded bound')
    return errors


def document(config):
    path = config['state'] / 'route/bridge-forward.json'
    require(path.is_file(), 'runtime refusal did not preserve its private state')
    record = json.loads(path.read_text())
    require(record['version'] == 2, 'security fixture journal version mismatch')
    require(path.stat().st_mode & 0o777 == 0o600, 'journal is not exactly private0600')
    return record['document']


def runtime_negative(lab, source, target, side, failure):
    config = lab.bridge_config('sec-' + side[:1] + '-' + failure, source, target,
                               [('route', 'input/q1/', 'output/q1/', 1)])
    private(config['config'], config['config'].read_text().replace('max_retries=2', 'max_retries=1'))
    changes = {'ca': str(lab.root / 'other.crt')} if failure == 'ca' else (
        {'host': 'localhost'} if failure == 'san' else (
        {'password_file': str(lab.root / 'wrong-password')} if failure == 'password' else
        {'cert': str(lab.root / 'untrusted.crt'), 'key': str(lab.root / 'untrusted.key')}))
    replace_section(config['config'], side, changes)
    if failure == 'san':
        topology = lab.root / (config['name'] + '-topology.toml')
        text = topology.read_text().replace(f'id="{(source if side == "source" else target)["id"]}"\nhost="127.0.0.1"',
                                             f'id="{(source if side == "source" else target)["id"]}"\nhost="localhost"')
        private(topology, text)
    observer = lab.subscribe(target, 'output/q1/retained', total=2)
    child = checked_launch(lab, config)
    failed_output(lab, child)
    lab.absent(observer)
    state = document(config)
    require(state['pending'] is None and not state['source_known'],
            'connection refusal admitted or confirmed a source obligation')
    require(state['destination_known'] is (side == 'source'), 'unexpected authenticated leg state')
    retained = lab.message(lab.subscribe(source, 'input/q1/retained'))
    require(retained['retain'] and retained['qos'] == 1, 'source retained obligation became unavailable')
    return dict(runtime_refusal=True, source_unconfirmed=True, no_destination_message=True,
                source_retained_preserved=True, pending=False)


def source_acl(lab, target):
    security = lab.root / 'security.toml'
    original = security.read_text()
    # Broker ACL matches requested subscription filters exactly; observer's exact
    # topic remains authorized while the bridge's input/q1/# is deliberately absent.
    private(security, original.replace(', "input/q1/#"', ''))
    try:
        source = lab.endpoint('source-denied')
    finally:
        private(security, original)
    lab.publish(source, 'input/q1/retained', b'fixture', 1, retain=True)
    from verify_wss_mtls import connect_packet, secure_socket
    with secure_socket(lab.root, source['port']) as probe:
        probe.sendall(connect_packet(5, b'c149-acl-probe', b'device', lab.secret.encode()))
        header, body = receive(probe)
        require(header == 0x20 and body[:2] == b'\x00\x00', 'ACL probe did not authenticate')
        probe.sendall(packet(0x82, b'\x00\x01\x00' + field('input/q1/#') + b'\x01'))
        header, body = receive(probe)
        cursor = Cursor(body)
        require(header == 0x90 and cursor.number(2) == 1, 'ACL probe received unexpected SUBACK')
        cursor.properties()
        require(cursor.take(len(body) - cursor.offset) == b'\x87', 'broker did not refuse subscription by ACL')
    config = lab.bridge_config('sec-suback', source, target,
                               [('route', 'input/q1/', 'output/q1/', 1)])
    observer = lab.subscribe(target, 'output/q1/retained', total=2)
    failed_output(lab, checked_launch(lab, config))
    lab.absent(observer)
    state = document(config)
    require(state['pending'] is None and state['source_known'] and state['destination_known'],
            'SUBACK refusal lost leg identity or admitted a publication')
    require(lab.message(lab.subscribe(source, 'input/q1/retained'))['retain'],
            'source retained value was consumed by refused subscription')
    return dict(actual_acl_suback_refused=True, no_destination_message=True,
                pending=False, source_retained_preserved=True)


@contextmanager
def wire_case(lab, name):
    fixture = SimpleNamespace(bridge=lab.args.bridge, root=lab.root, secret=lab.secret,
                              fingerprint=lab.fingerprint)
    case = Case(fixture, name)
    try:
        yield case
    finally:
        case.close()


def destination_acl(lab):
    security = lab.root / 'security.toml'
    original = security.read_text()
    # Subscribe-only grant permits an independent observer, never a PUBLISH.
    position = original.index('subscribe=')
    changed = original[:position] + original[position:].replace('subscribe=[', 'subscribe=["dst/value", ', 1)
    private(security, changed)
    try:
        target = lab.endpoint('target-denied')
    finally:
        private(security, original)
    with wire_case(lab, 'acl-destination') as case:
        replace_section(case.config, 'destination', {'port': target['port'], 'username': 'device'})
        topology = case.root / 'topology.toml'
        private(topology, topology.read_text().replace(f'port={case.destination_peer.port}',
                                                       f'port={target["port"]}'))
        observer = lab.subscribe(target, 'dst/value', total=2)
        case.launch()
        case.source_connect()
        case.source.sendall(publish(qos=1, payload=b'fixture'))
        require(case.child.wait(timeout=12) != 0, 'destination ACL refusal exited successfully')
        quiet(case.source, closed=True)
        state = case.document()
        require(state['pending'] is not None and state['pending']['phase'] == 'refused',
                'destination ACL did not durably preserve refused obligation')
        require(state['pending']['envelope']['source_pkid'] == 37, 'source obligation identity lost')
        before = case.journal.read_bytes()
        lab.absent(observer)
        case.launch()
        require(case.child.wait(timeout=8) != 0, 'refused obligation replayed after restart')
        require(case.journal.read_bytes() == before, 'refused obligation changed on restart')
        quiet(case.source, closed=True)
    return dict(actual_acl_publish_refused=True, no_source_ack=True, no_destination_message=True,
                preserved_phase='refused', restart_refuses_unchanged=True)


def qos0_bound(lab):
    with wire_case(lab, 'qos0-burst') as case:
        case.start()
        case.source.sendall(publish(qos=0, payload=b'first'))
        first = read_publish(case.destination)
        require(first['payload'] == b'first' and first['qos'] == 1, 'first bounded delivery missing')
        case.phase('sent')
        before = case.journal.read_bytes()
        quiet(case.source)
        case.source.sendall(publish(qos=0, payload=b'second'))
        case.failed('sent')
        require(case.journal.read_bytes() == before, 'additional QoS0 replaced pending obligation')
        require(case.document()['pending']['envelope']['source_qos'] == 0, 'pending QoS0 identity changed')
    return dict(first_destination_publish=1, extra_destination_publish=0, no_source_ack=True,
                preserved_phase='sent', receive_maximum_does_not_protect_qos0=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['bridge', 'client', 'broker-root', 'report']:
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    if os.name != 'posix':
        parser.error('Linux/WSL required')
    args.bridge, args.client = args.bridge.resolve(strict=True), args.client.resolve(strict=True)
    args.broker_root = args.broker_root.resolve(strict=True)
    with args.report.open('x', encoding='utf-8') as report:
        result = dict(schema='xmqr-bridge-mqtt5-security-c149-v1', status='FAIL',
                      started_unix=int(time.time()), hashes={
                          'bridge_elf_sha256': digest(args.bridge), 'client_elf_sha256': digest(args.client),
                          'broker_elf_sha256': digest(args.broker_root / 'target/debug/mqtt-broker'),
                          'admin_elf_sha256': digest(args.broker_root / 'target/debug/mqtt-admin'),
                          'fixture_sha256': digest(Path(__file__).resolve())}, scenarios=[])

        def scenario(name, action):
            item = dict(name=name, status='FAIL')
            result['scenarios'].append(item)
            item['evidence'] = action()
            item['status'] = 'PASS'
            print('PASS:', name, flush=True)

        try:
            with tempfile.TemporaryDirectory(prefix='xmqr-c149-security-') as directory:
                root = Path(directory)
                root.chmod(0o700)
                lab = Lab(args, root)
                try:
                    lab.initialize()
                    import hashlib
                    import ssl
                    lab.fingerprint = hashlib.sha256(ssl.PEM_cert_to_DER_cert(
                        (root / 'client.crt').read_text())).hexdigest()
                    private(root / 'wrong-password', 'invalid-' + lab.secret + '\n')
                    from verify_wss_mtls import openssl
                    openssl(root, 'req', '-new', '-newkey', 'rsa:2048', '-nodes', '-keyout', 'untrusted.key',
                            '-out', 'untrusted.csr', '-subj', '/CN=untrusted-client')
                    private(root / 'untrusted.ext', 'basicConstraints=CA:FALSE\nextendedKeyUsage=clientAuth\n')
                    openssl(root, 'x509', '-req', '-in', 'untrusted.csr', '-CA', 'other.crt', '-CAkey',
                            'other.key', '-set_serial', '9', '-days', '1', '-extfile', 'untrusted.ext',
                            '-out', 'untrusted.crt')
                    (root / 'untrusted.key').chmod(0o600)
                    (root / 'untrusted.crt').chmod(0o600)
                    source, target = lab.endpoint('source'), lab.endpoint('target')
                    lab.publish(source, 'input/q1/retained', b'fixture', 1, retain=True)
                    for side in ['source', 'destination']:
                        for failure in ['ca', 'san', 'password', 'clientcert']:
                            scenario(side + ': reject ' + failure,
                                     lambda s=side, f=failure: runtime_negative(lab, source, target, s, f))
                    scenario('source: actual ACL refuses SUBACK', lambda: source_acl(lab, target))
                    scenario('destination: actual ACL refuses PUBLISH without source ACK', lambda: destination_acl(lab))
                    scenario('one-flight: additional QoS0 refuses without replacing obligation', lambda: qos0_bound(lab))
                    result['status'] = 'PASS'
                finally:
                    lab.close()
        except Exception as error:
            result['error_type'] = type(error).__name__
            trace = error.__traceback__
            while trace and trace.tb_next:
                trace = trace.tb_next
            if trace:
                result['error_line'] = trace.tb_lineno
            if isinstance(error, (AssertionError,)) or type(error).__name__ == 'CheckError':
                result['reason'] = str(error)[:180]
            print('FAIL: security fixture stopped; sanitized immutable report preserved.', file=sys.stderr)
        finally:
            result['finished_unix'] = int(time.time())
            result['passed'] = sum(item['status'] == 'PASS' for item in result['scenarios'])
            result['total'] = len(result['scenarios'])
            json.dump(result, report, indent=2)
            report.write('\n')
        return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    sys.exit(main())
