#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C148: real private mTLS brokers, MQTT 5 metadata, retention and bounded paths.

Existing Docker image only; no pulls, package installation, or operator services.
Acknowledgements are per MQTT leg, never proof of business processing.
"""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import select
import signal
import socket
import ssl
import stat
import subprocess
import sys
import tempfile
import time


PATH_KEY = 'xmqr.bridge.path'
COMBINATIONS = [(0, 0), (1, 0), (1, 1), (2, 0), (2, 1), (2, 2)]


def private(path, content):
    path.write_text(content, encoding='utf-8')
    path.chmod(0o600)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def free_port():
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 0))
        return probe.getsockname()[1]


def until(check, seconds=10, message='owned fixture deadline'):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        value = check()
        if value:
            return value
        time.sleep(.04)
    raise AssertionError(message)


def stop(child):
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=8)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=3)


def bound(number, child=None):
    def ready():
        if child:
            assert child.poll() is None, 'owned broker exited before listener readiness'
        try:
            with socket.create_connection(('127.0.0.1', number), timeout=.1):
                return True
        except OSError:
            return False
    until(ready, 15, 'owned TLS broker did not bind')


def markers(child, expected, seconds=15):
    output = bytearray()
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if select.select([child.stderr], [], [], .1)[0]:
            chunk = os.read(child.stderr.fileno(), 4096)
            output.extend(chunk)
            assert len(output) <= 32768, 'owned CLI diagnostics exceeded fixture bound'
            if all(item.encode() in output for item in expected):
                return
            assert chunk or child.poll() is None, 'owned CLI exited before protocol readiness'
        else:
            assert child.poll() is None, 'owned CLI exited before protocol readiness'
    raise AssertionError('owned CLI protocol readiness deadline')


def docker(*args, timeout=30):
    return subprocess.run(['docker', *map(str, args)], check=True, capture_output=True,
                          text=True, timeout=timeout)


def client_environment():
    return {key: value for key, value in os.environ.items()
            if not key.startswith(('MQTT_', 'XMQR_BRIDGE_TEST_'))}


def pinned_image(image):
    inspected = json.loads(docker('image', 'inspect', image).stdout)[0]
    candidates = inspected.get('RepoDigests') or []
    assert candidates, 'cached Mosquitto image has no immutable repository digest'
    selected = next((item for item in candidates if item.startswith('eclipse-mosquitto@sha256:')), None)
    assert selected and re.fullmatch(r'.+@sha256:[0-9a-f]{64}', selected)
    return dict(requested=image, repository_digest=selected, image_id=inspected['Id'])


class Lab:
    def __init__(self, args, root, image=None):
        self.args, self.root, self.image = args, root, image
        self.children, self.containers, self.serial = [], [], 0

    def spawn(self, argv, **kwargs):
        kwargs.setdefault('env', client_environment())
        child = subprocess.Popen([str(item) for item in argv], **kwargs)
        self.children.append(child)
        return child

    def close(self):
        for child in reversed(self.children):
            stop(child)
        for container in reversed(self.containers):
            assert re.fullmatch(r'[0-9a-f]{64}', container), 'refuse ambiguous container cleanup'
            subprocess.run(['docker', 'rm', '-f', container], capture_output=True, timeout=20)

    def initialize(self):
        sys.path.insert(0, str(self.args.broker_root / 'scripts'))
        from verify_wss_mtls import certificates
        from verify_multi_endpoint import hash_fixture
        certificates(self.root)
        self.secret = secrets.token_urlsafe(24)
        private(self.root / 'password', self.secret + '\n')
        fingerprint = hashlib.sha256(ssl.PEM_cert_to_DER_cert(
            (self.root / 'client.crt').read_text())).hexdigest()
        password_hash = hash_fixture(self.args.broker_root / 'target/debug/mqtt-admin', self.secret)
        grants = [prefix + suffix for prefix in ['input/q0/', 'input/q1/', 'input/q2/',
                                                  'output/q0/', 'output/q1/', 'output/q2/']
                  for suffix in ['value', 'retained', 'live', 'return', 'hop-limit', 'stripped']]
        grants += ['chain/value', 'relay/value', 'final/value']
        subscriptions = [*grants, 'input/q0/#', 'input/q1/#', 'input/q2/#', 'chain/#', 'relay/#']
        private(self.root / 'security.toml', 'version=1\n[[users]]\nusername="device"\n'
                f'password_hash="{password_hash}"\ncert_sha256="sha256:{fingerprint}"\n'
                f'publish={json.dumps(grants)}\nsubscribe={json.dumps(subscriptions)}\n')
        for name in ['ca.crt', 'server.crt', 'server.key', 'client.crt', 'client.key']:
            (self.root / name).chmod(0o600)

    def endpoint(self, name, kind='xmqr'):
        directory = self.root / name
        directory.mkdir(mode=0o700)
        number = free_port()
        if kind == 'xmqr':
            config = directory / 'broker.toml'
            text = ('version=1\n[broker]\nmode="secure-mtls"\n'
                    f'bind="127.0.0.1:{number}"\nstate_dir={json.dumps(str(directory / "state"))}\n[tls]\n')
            for field, filename in [('server_cert', 'server.crt'), ('server_key', 'server.key'), ('client_ca', 'ca.crt')]:
                text += f'{field}={json.dumps(str(self.root / filename))}\n'
            text += f'[security]\nbundle={json.dumps(str(self.root / "security.toml"))}\n'
            private(config, text)
            environment = {key: value for key, value in os.environ.items() if not key.startswith('MQTT_')}
            environment['MQTT_CONFIG'] = str(config)
            with (directory / 'broker.log').open('ab') as log:
                child = self.spawn([self.args.broker_root / 'target/debug/mqtt-broker'],
                                   env=environment, stdin=subprocess.DEVNULL, stdout=log, stderr=log)
            bound(number, child)
        else:
            assert self.image, 'Mosquitto image not configured'
            # Password conversion uses a private file; password never enters argv.
            private(directory / 'passwords', 'device:' + self.secret + '\n')
            docker('run', '--rm', '--user', '0:0', '--entrypoint', 'mosquitto_passwd',
                   '-v', f'{directory}:/work', self.image['repository_digest'], '-U', '/work/passwords')
            state = directory / 'state'
            state.mkdir(mode=0o700)
            private(directory / 'acl', 'user device\ntopic readwrite #\n')
            private(directory / 'mosquitto.conf', 'user root\nlistener 1883 0.0.0.0\n'
                    'allow_anonymous false\nrequire_certificate true\n'
                    'use_identity_as_username false\npassword_file /fixture/' + name + '/passwords\n'
                    'acl_file /fixture/' + name + '/acl\n'
                    'cafile /fixture/ca.crt\ncertfile /fixture/server.crt\nkeyfile /fixture/server.key\n'
                    f'persistence true\npersistence_location /fixture/{name}/state/\nautosave_interval 1\n')
            container_name = 'xmqr-c148-' + secrets.token_hex(8)
            created = docker('run', '-d', '--name', container_name, '--label', 'xmqr.fixture=c148-brokers',
                             '--user', '0:0', '-p', f'127.0.0.1:{number}:1883',
                             '-v', f'{self.root}:/fixture', self.image['repository_digest'],
                             'mosquitto', '-c', f'/fixture/{name}/mosquitto.conf').stdout.strip()
            assert re.fullmatch(r'[0-9a-f]{64}', created), 'Docker did not identify owned container'
            self.containers.append(created)
            bound(number)
        return dict(id=name, port=number)

    def cli(self, endpoint, mode, topic, qos, *, properties=None, retain=False, total=12):
        self.serial += 1
        command = [str(self.args.client), mode, '--host', '127.0.0.1', '--port', str(endpoint['port']),
                   '--protocol-version', '5', '--username', 'device', '--password-file', str(self.root / 'password'),
                   '--ca', str(self.root / 'ca.crt'), '--cert', str(self.root / 'client.crt'),
                   '--key', str(self.root / 'client.key'), '--client-id', f'c148-{self.serial}-{secrets.token_hex(3)}',
                   '--topic', topic, '--qos', str(qos), '--output', 'jsonl', '--timeout-seconds', '6',
                   '--total-timeout-seconds', str(total)]
        if mode == 'pub':
            command += ['--stdin', '--retain', str(retain).lower()]
            fields = properties or {}
            for key, flag in [('payload_format_indicator', '--payload-format'),
                              ('content_type', '--content-type'), ('response_topic', '--response-topic'),
                              ('correlation_data_hex', '--correlation-data-hex'),
                              ('message_expiry_interval', '--message-expiry-seconds')]:
                if key in fields:
                    command += [flag, str(fields[key])]
            for key, value in fields.get('user_properties', []):
                command += ['--publish-user-property', key + '=' + value]
        else:
            command += ['--count', '1', '--retain-as-published']
        return command

    def publish(self, endpoint, topic, payload, qos, *, properties=None, retain=False):
        result = subprocess.run(self.cli(endpoint, 'pub', topic, qos, properties=properties, retain=retain),
                                input=payload, capture_output=True, timeout=15, env=client_environment())
        assert result.returncode == 0, 'MQTT 5 publisher failed'
        rows = [json.loads(line) for line in result.stdout.splitlines()]
        assert any(row.get('event') == 'publish_complete' and row.get('qos') == qos for row in rows)

    def subscribe(self, endpoint, topic, *, total=12):
        child = self.spawn(self.cli(endpoint, 'sub', topic, 2, total=total),
                           stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        markers(child, ['Assinatura ativa'], min(total, 10))
        return child

    def message(self, child):
        output, _ = child.communicate(timeout=15)
        assert child.returncode == 0, 'MQTT 5 subscriber failed'
        rows = [json.loads(line) for line in output.splitlines()]
        publications = [row for row in rows if row.get('event') == 'publish']
        assert len(publications) == 1, 'observer did not receive exactly one publication'
        return publications[0]

    def absent(self, child):
        output, _ = child.communicate(timeout=6)
        assert child.returncode != 0 and not output, 'unexpected MQTT publication during negative observation'

    def bridge_config(self, name, source, target, routes, *, policy='preserve', hops=2, topology=None):
        state = self.root / (name + '-state')
        state.mkdir(mode=0o700)
        topology = topology or [(source, target, *route) for route in routes]
        manifest = 'version=1\n'
        endpoints = {item['id']: item for edge in topology for item in edge[:2]}
        for endpoint in endpoints.values():
            manifest += f'[[brokers]]\nid="{endpoint["id"]}"\nhost="127.0.0.1"\nport={endpoint["port"]}\n'
        for origin, destination, route_id, from_prefix, to_prefix, qos in topology:
            manifest += (f'[[links]]\nid="{route_id}"\nsource_broker="{origin["id"]}"\n'
                         f'destination_broker="{destination["id"]}"\nsource_prefix="{from_prefix}"\n'
                         f'target_prefix="{to_prefix}"\nqos={qos}\n')
        topology_file = self.root / (name + '-topology.toml')
        private(topology_file, manifest)
        config = self.root / (name + '.toml')
        text = ('version=2\nsource_protocol="mqtt5"\ndestination_protocol="mqtt5"\n'
                f'bridge_id="{name}"\nmax_hops={hops}\nresponse_topic_policy="{policy}"\n'
                f'state_dir={json.dumps(str(state))}\ntopology_file={json.dumps(str(topology_file))}\nmax_retries=2\n')
        for side, endpoint, suffix in [('source', source, 's'), ('destination', target, 'd')]:
            text += (f'[{side}]\nbroker_id="{endpoint["id"]}"\nhost="127.0.0.1"\nport={endpoint["port"]}\n'
                     f'client_id="{name}-{suffix}"\nusername="device"\n')
            for field, filename in [('password_file', 'password'), ('ca', 'ca.crt'), ('cert', 'client.crt'), ('key', 'client.key')]:
                text += f'{field}={json.dumps(str(self.root / filename))}\n'
        for route_id, from_prefix, to_prefix, qos in routes:
            text += (f'[[routes]]\nid="{route_id}"\nsource_prefix="{from_prefix}"\n'
                     f'target_prefix="{to_prefix}"\nqos={qos}\n')
        private(config, text)
        return dict(config=config, state=state, routes=routes, name=name)

    def start_bridge(self, config):
        result = subprocess.run([str(self.args.bridge), '--check', '--config', str(config['config'])],
                                capture_output=True, timeout=10, env=client_environment())
        assert result.returncode == 0, 'static MQTT 5 bridge configuration check failed'
        child = self.spawn([self.args.bridge, '--config', config['config']], stdin=subprocess.DEVNULL,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        markers(child, [f'origem assinada (rota {route[0]})' for route in config['routes']])
        return child

    def settled(self, config):
        def done():
            for route in config['routes']:
                path = config['state'] / route[0] / 'bridge-forward.json'
                if not path.exists():
                    return False
                if json.loads(path.read_text())['document']['pending'] is not None:
                    return False
            return True
        until(done, 10, 'forwarding journal did not settle')


def properties(prefix, expiry=60):
    return dict(payload_format_indicator=1, content_type='application/json',
                response_topic=prefix + 'reply', correlation_data_hex='000aff807f',
                user_properties=[['tag', 'first'], ['tag', 'second'], ['tail', 'end']],
                message_expiry_interval=expiry)


def expected(message, topic, payload, qos, path, response, *, retain=False, expiry_max=60):
    assert message['topic'] == topic and bytes(message['payload_bytes']) == payload, 'forwarded application message mismatch'
    assert message['qos'] == qos and message['retain'] is retain, 'QoS or retained flag changed'
    metadata = message.get('metadata', {})
    assert metadata.get('payload_format_indicator') == 1
    assert metadata.get('content_type') == 'application/json'
    assert metadata.get('response_topic') == response
    assert metadata.get('correlation_data_hex') == '000aff807f'
    assert metadata.get('user_properties') == [['tag', 'first'], ['tag', 'second'], ['tail', 'end'], [PATH_KEY, path]], 'ordered/repeated user properties or bounded path changed'
    assert 0 < metadata.get('message_expiry_interval', 0) <= expiry_max, 'message expiry was reset or lost'


def matrix(lab, source, target, label, scenario, *, path_cases=False):
    routes = [(f'q{qos}', f'input/q{qos}/', f'output/q{qos}/', qos) for qos in range(3)]
    config = lab.bridge_config('c148-' + label, source, target, routes)
    initial = b'{"fixture":"retained-initial"}'
    with scenario(label + ': retained source before bridge and elapsed expiry on destination replay'):
        lab.publish(source, 'input/q2/retained', initial, 2, retain=True, properties=properties('input/q2/'))
        time.sleep(3.1)
        observer = lab.subscribe(target, 'output/q2/retained')
        bridge = lab.start_bridge(config)
        first = lab.message(observer)
        expected(first, 'output/q2/retained', initial, 2, config['name'], 'input/q2/reply', retain=True, expiry_max=57)
        lab.settled(config)
        replay = lab.message(lab.subscribe(target, 'output/q2/retained'))
        expected(replay, 'output/q2/retained', initial, 2, config['name'], 'input/q2/reply', retain=True, expiry_max=57)
    for route_qos, source_qos in COMBINATIONS:
        with scenario(label + f': route QoS {route_qos} source QoS {source_qos} and complete metadata'):
            topic = f'input/q{route_qos}/value'
            payload = json.dumps(dict(fixture='qos', route=route_qos, source=source_qos)).encode()
            observer = lab.subscribe(target, f'output/q{route_qos}/value')
            lab.publish(source, topic, payload, source_qos, properties=properties(f'input/q{route_qos}/'))
            message = lab.message(observer)
            expected(message, f'output/q{route_qos}/value', payload, route_qos,
                     config['name'], f'input/q{route_qos}/reply')
            lab.settled(config)
    with scenario(label + ': all routes keep separate private identities locks and durable state'):
        identities, inodes = set(), set()
        for route_id, *_ in routes:
            directory = config['state'] / route_id
            path = directory / 'bridge-forward.json'
            assert stat.S_IMODE(directory.stat().st_mode) == 0o700
            assert stat.S_IMODE(path.stat().st_mode) == 0o600
            record = json.loads(path.read_text())['document']
            assert record['source_known'] and record['destination_known'] and record['pending'] is None
            identities.add(record['identity'])
            inodes.add(path.stat().st_ino)
            assert (directory / 'bridge-forward.lock').is_file()
            assert bool(record['completed']) is (route_id == 'q2')
        assert len(identities) == len(inodes) == 3
    with scenario(label + ': live retained message preserves metadata and retained flag'):
        payload = b'{"fixture":"retained-live"}'
        observer = lab.subscribe(target, 'output/q2/live')
        lab.publish(source, 'input/q2/live', payload, 2, retain=True, properties=properties('input/q2/'))
        expected(lab.message(observer), 'output/q2/live', payload, 2, config['name'], 'input/q2/reply', retain=True)
        lab.settled(config)
    with scenario(label + ': empty retained tombstone removes destination retained value'):
        observer = lab.subscribe(target, 'output/q2/live')
        prior = lab.message(observer)
        assert prior['retain'] is True
        # Suppress initial replay to observe the live zero-byte tombstone itself.
        command = lab.cli(target, 'sub', 'output/q2/live', 2)
        command += ['--retain-handling', '2']
        observer = lab.spawn(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        markers(observer, ['Assinatura ativa'])
        lab.publish(source, 'input/q2/live', b'', 2, retain=True, properties=properties('input/q2/'))
        expected(lab.message(observer), 'output/q2/live', b'', 2, config['name'], 'input/q2/reply', retain=True)
        lab.settled(config)
        lab.absent(lab.subscribe(target, 'output/q2/live', total=2))
    if path_cases:
        returned = b'{"fixture":"bounded-return"}'
        with scenario('external cooperative finite forwarder returns path and bridge discards without replay'):
            observer = lab.subscribe(target, 'output/q2/return')
            lab.publish(source, 'input/q2/return', returned, 2, properties=properties('input/q2/'))
            message = lab.message(observer)
            lab.settled(config)
            observer = lab.subscribe(target, 'output/q2/return', total=2)
            lab.publish(source, 'input/q2/return', returned, 2, properties=message['metadata'])
            lab.absent(observer)
            lab.settled(config)
            assert bridge.poll() is None
        with scenario('maximum-hop path is durably consumed without new forwarding'):
            metadata = properties('input/q2/')
            metadata['user_properties'].append([PATH_KEY, 'external-one,external-two'])
            observer = lab.subscribe(target, 'output/q2/hop-limit', total=2)
            lab.publish(source, 'input/q2/hop-limit', returned, 2, properties=metadata)
            lab.absent(observer)
            lab.settled(config)
            assert bridge.poll() is None
        with scenario('marker-stripping boundary is explicit: finite stripped return is forwarded again'):
            metadata = dict(message['metadata'])
            metadata['user_properties'] = [pair for pair in metadata['user_properties'] if pair[0] != PATH_KEY]
            observer = lab.subscribe(target, 'output/q2/stripped')
            lab.publish(source, 'input/q2/stripped', returned, 2, properties=metadata)
            result = lab.message(observer)
            assert bytes(result['payload_bytes']) == returned
            assert result['metadata']['user_properties'][-1] == [PATH_KEY, config['name']]
            lab.settled(config)
    stop(bridge)
    return config


def chain(lab, source, middle, target, scenario):
    routes_a = [('hop-a', 'chain/', 'relay/', 2)]
    routes_b = [('hop-b', 'relay/', 'final/', 2)]
    topology = [(source, middle, *routes_a[0]), (middle, target, *routes_b[0])]
    a = lab.bridge_config('c148-chain-a', source, middle, routes_a, policy='map-prefix', topology=topology)
    b = lab.bridge_config('c148-chain-b', middle, target, routes_b, policy='map-prefix', topology=topology)
    with scenario('three real XMQR brokers in a chain preserve ordered path and map-prefix response topics'):
        second = lab.start_bridge(b)
        first = lab.start_bridge(a)
        observer = lab.subscribe(target, 'final/value')
        payload = b'{"fixture":"chain"}'
        lab.publish(source, 'chain/value', payload, 2, properties=properties('chain/'))
        message = lab.message(observer)
        expected(message, 'final/value', payload, 2, 'c148-chain-a,c148-chain-b', 'final/reply')
        lab.settled(a)
        lab.settled(b)
        assert first.poll() is None and second.poll() is None
        stop(first)
        stop(second)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bridge', type=Path, required=True)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--broker-root', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--mosquitto', nargs='?', const='eclipse-mosquitto:2.0.22',
                        help='existing Docker image tag/digest; omitted means incomplete matrix')
    args = parser.parse_args()
    if os.name != 'posix':
        parser.error('Linux/WSL is required')
    args.bridge, args.client = args.bridge.resolve(strict=True), args.client.resolve(strict=True)
    args.broker_root = args.broker_root.resolve(strict=True)
    with args.report.open('x', encoding='utf-8') as output:
        result = dict(schema='xmqr-bridge-mqtt5-real-brokers-c148-v1', status='FAIL',
                      started_unix=int(time.time()),
                      hashes=dict(bridge_elf_sha256=digest(args.bridge), client_elf_sha256=digest(args.client),
                                  broker_elf_sha256=digest(args.broker_root / 'target/debug/mqtt-broker'),
                                  admin_elf_sha256=digest(args.broker_root / 'target/debug/mqtt-admin'),
                                  fixture_sha256=digest(Path(__file__).resolve())),
                      mosquitto='not requested', scenarios=[])

        @contextmanager
        def scenario(name):
            item = dict(name=name, status='FAIL')
            result['scenarios'].append(item)
            try:
                yield
                item['status'] = 'PASS'
                print('PASS:', name, flush=True)
            except Exception as failure:
                item['error_type'] = type(failure).__name__
                trace = failure.__traceback__
                while trace and trace.tb_next:
                    trace = trace.tb_next
                if trace:
                    item['error_line'] = trace.tb_lineno
                if isinstance(failure, AssertionError):
                    item['reason'] = str(failure)[:160]
                raise

        try:
            image = pinned_image(args.mosquitto) if args.mosquitto else None
            if image:
                result['mosquitto'] = image
            for label, source_kind, target_kind in [('xx', 'xmqr', 'xmqr'),
                                                     *([('xm', 'xmqr', 'mosquitto'), ('mx', 'mosquitto', 'xmqr')] if image else [])]:
                with tempfile.TemporaryDirectory(prefix='xmqr-c148-' + label + '-') as directory:
                    root = Path(directory)
                    root.chmod(0o700)
                    lab = Lab(args, root, image)
                    try:
                        lab.initialize()
                        source, target = lab.endpoint('source', source_kind), lab.endpoint('target', target_kind)
                        matrix(lab, source, target, label, scenario, path_cases=label == 'xx')
                        if label == 'xx':
                            third = lab.endpoint('third')
                            chain(lab, source, target, third, scenario)
                    finally:
                        lab.close()
            result['status'] = 'PASS' if image else 'INCOMPLETE'
            if not image:
                result['missing'] = ['XMQR-to-Mosquitto', 'Mosquitto-to-XMQR']
        except Exception as failure:
            result['error_type'] = type(failure).__name__
            trace = failure.__traceback__
            while trace and trace.tb_next:
                trace = trace.tb_next
            if trace:
                result['error_line'] = trace.tb_lineno
            if isinstance(failure, AssertionError):
                result['reason'] = str(failure)[:160]
            print('FAIL: real-broker fixture stopped; see sanitized immutable report.', file=sys.stderr)
        finally:
            result['finished_unix'] = int(time.time())
            result['passed'] = sum(item['status'] == 'PASS' for item in result['scenarios'])
            result['total'] = len(result['scenarios'])
            json.dump(result, output, indent=2)
            output.write('\n')
        return 0 if result['status'] == 'PASS' else 2 if result['status'] == 'INCOMPLETE' else 1


if __name__ == '__main__':
    sys.exit(main())
