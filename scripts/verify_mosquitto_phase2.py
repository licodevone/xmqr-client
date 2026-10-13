#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C43: pinned external mTLS interoperability in test-only Docker namespaces."""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import ssl
import subprocess
import sys
import tempfile
import time
import uuid

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('docker_fixture', ROOT.parent/'xmqr/scripts/verify_docker.py')
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)
PAYLOAD = b'phase2\0\xff\r\n'


def invoke(*args, check=True):
    result = subprocess.run(['docker', *map(str, args)], capture_output=True, timeout=35)
    if check and result.returncode:
        # Do not echo command arguments, paths, certificates or tool errors.
        raise AssertionError('test-owned Docker operation failed')
    return result


def metadata(reference):
    info = json.loads(invoke('image', 'inspect', reference).stdout)[0]
    if info['Os'] != 'linux' or info['Architecture'] != 'amd64':
        raise ValueError('only Linux/amd64 is accepted')
    return dict(id=info['Id'], repo_digests=info.get('RepoDigests', []), labels=info['Config'].get('Labels', {}))


def validate_xmqr(output, qos, retained=False, payload=PAYLOAD):
    events = [json.loads(line) for line in output.decode().splitlines() if line.startswith('{')]
    if len(events) != 1 or events[0] != dict(event='publish', topic='phase2/value', qos=qos,
                                           retain=retained, payload_bytes=list(payload)):
        raise AssertionError('XMQR message/bytes/QoS/retain mismatch')


def validate_mosquitto(output, qos, retained=False, payload=PAYLOAD):
    events = [line.strip() for line in output.decode().splitlines() if line.startswith('DATA|')]
    if events != [f'DATA|phase2/value|{qos}|{int(retained)}|{payload.hex()}']:
        raise AssertionError('Mosquitto message/bytes/QoS/retain mismatch')


class Campaign:
    def __init__(self, args, images):
        self.args, self.images = args, images
        self.prefix = 'xmqr-c43-'+uuid.uuid4().hex[:10]
        self.containers, self.volumes, self.results = [], [], []

    def client_args(self, kind, command, qos, topic='phase2/value', count=1, retain=False, negative=None, persistent=None, will=False):
        if kind == 'xmqr':
            options = [command, '--host', '127.0.0.1', '--port', '8883', '--username', 'fixture-device',
                       '--ca', '/run/certs/ca.crt', '--cert', '/run/certs/client.crt', '--key', '/run/certs/client.key',
                       '--password-file', '/run/certs/password', '--output', 'jsonl', '--qos', str(qos),
                       '--timeout-seconds', '3', '--topic', topic]
            if command == 'sub':
                options += ['--count', str(count)]
            else:
                options += ['--message-file', '/run/certs/payload.bin', '--retain', str(retain).lower()]
            field = {'ca': '--ca', 'password': '--password-file'}.get(negative)
            if field:
                options[options.index(field)+1] = '/run/certs/'+('other.crt' if negative == 'ca' else 'wrong-password')
            if persistent:
                options += ['--clean-session', 'false', '--client-id', persistent]
            if will:
                options += ['--will-topic', 'phase2/value', '--will-message', 'phase2-will', '--will-qos', '1']
            return self.images['client']['id'], [], options
        options = ['-h', '127.0.0.1', '-p', '8883', '-V', 'mqttv311', '-i', 'c43-'+uuid.uuid4().hex[:12], '-u', 'fixture-device',
                   '-P', 'wrong-fixture' if negative == 'password' else 'fixture-pass',
                   '--cafile', '/run/certs/other.crt' if negative == 'ca' else '/run/certs/ca.crt',
                   '--cert', '/run/certs/client.crt', '--key', '/run/certs/client.key', '-q', str(qos), '-t', topic]
        if command == 'sub':
            options += ['-C', str(count), '-W', '8', '-d', '-F', 'DATA|%t|%q|%r|%x']
        else:
            options += ['-f', '/run/certs/payload.bin']
            if retain:
                options += ['-r']
        if persistent:
            options[options.index('-i')+1] = persistent
            options += ['-c']
        if will:
            options += ['--will-topic', 'phase2/value', '--will-payload', 'phase2-will', '--will-qos', '1']
        return self.images['mosquitto']['id'], ['--entrypoint', 'mosquitto_'+command], options

    def job(self, server, secrets, kind, command, qos, detached=False, **kwargs):
        image, entry, cli = self.client_args(kind, command, qos, **kwargs)
        name = self.prefix+'-job-'+uuid.uuid4().hex[:8]
        self.containers.append(name)
        options = ['run', '--name', name, '--network', 'container:'+server, '--user', '10001:10001',
                   '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges',
                   '-v', secrets+':/run/certs:ro']
        if detached:
            options += ['-d', '-t']
        result = invoke(*options, *entry, image, *cli, check=False)
        if detached:
            if result.returncode:
                raise AssertionError('subscriber launch failed')
            try:
                fixture.wait(lambda: (b'Assinatura ativa' if kind == 'xmqr' else b'received SUBACK')
                             in fixture.logs(name))
            except AssertionError:
                state = json.loads(invoke('inspect', name).stdout)[0]['State']
                output = fixture.logs(name).lower()
                print(json.dumps(dict(status='FAIL', subscriber=kind, exit_code=state['ExitCode'],
                      tls_error=b'certificate' in output or b'tls' in output,
                      refused=b'refused' in output, permission=b'permission' in output,
                      invalid_option=b'unknown option' in output, missing_binary=b'not found' in output)), flush=True)
                raise
            return name
        return result

    def finished(self, name):
        fixture.wait(lambda: json.loads(invoke('inspect', name).stdout)[0]['State']['Status'] == 'exited')
        if json.loads(invoke('inspect', name).stdout)[0]['State']['ExitCode'] != 0:
            raise AssertionError('external subscriber failed')
        return fixture.logs(name)

    def start_server(self, kind, secrets, wrong_name=False):
        name = self.prefix+'-'+kind+('-wrong-san' if wrong_name else '')
        self.containers.append(name)
        options = ['run', '-d', '--name', name, '--network', 'none', '--user', '10001:10001',
                   '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges', '-v', secrets+':/run/test']
        if kind == 'xmqr':
            options += ['-v', secrets+':/var/lib/xmqr', '-e', 'MQTT_BIND=127.0.0.1:8883',
                        '-e', 'MQTT_STATE_DIR=/run/test/'+('state2' if wrong_name else 'state'),
                        '-e', 'MQTT_SERVER_CERT=/run/test/'+('wrong-name.crt' if wrong_name else 'server.crt'),
                        '-e', 'MQTT_SERVER_KEY=/run/test/'+('wrong-name.key' if wrong_name else 'server.key'), '-e', 'MQTT_CLIENT_CA=/run/test/ca.crt',
                        '-e', 'MQTT_SECURITY_BUNDLE=/run/test/security.toml', '-e', 'RUST_LOG=info']
            image, cli = self.images['broker']['id'], []
        else:
            options += ['--entrypoint', 'mosquitto']
            image, cli = self.images['mosquitto']['id'], ['-c', '/run/test/'+('wrong-name.conf' if wrong_name else 'mosquitto.conf')]
        invoke(*options, image, *cli)
        fixture.wait(lambda: (b'listener started' if kind == 'xmqr' else b'mosquitto version 2.0.22 running')
                     in fixture.logs(name))
        return name

    def execute(self, root):
        volume = self.prefix+'-fixtures'
        invoke('volume', 'create', volume)
        self.volumes.append(volume)
        fingerprint = hashlib.sha256(ssl.PEM_cert_to_DER_cert((root/'client.crt').read_text())).hexdigest()
        password_hash = self.args.hash_file.read_text(encoding='utf-8-sig').strip()
        if not password_hash.startswith('$argon2id$v=19$m=19456,t=2,p=1$'):
            raise ValueError('public Argon2 fixture hash required')
        bundle = f"version=1\n[[users]]\nusername='fixture-device'\npassword_hash='{password_hash}'\ncert_sha256='sha256:{fingerprint}'\npublish=['phase2/value','phase2/will']\nsubscribe=['phase2/#','phase2/value','phase2/will']\n"
        config = ('listener 8883 127.0.0.1\nallow_anonymous false\nrequire_certificate true\n'
                  'cafile /run/test/ca.crt\ncertfile /run/test/server.crt\nkeyfile /run/test/server.key\n'
                  'password_file /run/test/password.db\nacl_file /run/test/acl\n'
                  'persistence false\nlog_dest stdout\nlog_type all\n')
        files = {name:(root/name).read_bytes() for name in ['ca.crt','server.crt','server.key','client.crt','client.key','other.crt','wrong-name.crt','wrong-name.key']}
        files.update({'security.toml':bundle.encode(), 'mosquitto.conf':config.encode(),
                      'wrong-name.conf':config.replace('server.crt','wrong-name.crt').replace('server.key','wrong-name.key').encode(),
                      'acl':b'user fixture-device\ntopic write phase2/value\ntopic write phase2/will\ntopic read phase2/#\n',
                      'password':b'fixture-pass','wrong-password':b'wrong-fixture','payload.bin':PAYLOAD})
        fixture.seed(self.images['broker']['id'], volume, files)
        invoke('run', '--rm', '--network', 'none', '--user', '10001:10001', '--entrypoint', 'sh',
               '-v', volume+':/run/test', self.images['mosquitto']['id'], '-c',
               'mkdir /run/test/state /run/test/state2 && mosquitto_passwd -b -c /run/test/password.db fixture-device fixture-pass')
        for server_kind in ('xmqr', 'mosquitto'):
            server = self.start_server(server_kind, volume)
            for qos in (0, 1, 2):
                for pub_kind, sub_kind in [('xmqr', 'mosquitto'), ('mosquitto', 'xmqr')]:
                    subscriber = self.job(server, volume, sub_kind, 'sub', qos, detached=True)
                    published = self.job(server, volume, pub_kind, 'pub', qos)
                    if published.returncode:
                        raise AssertionError('external publisher failed')
                    output = self.finished(subscriber)
                    (validate_xmqr if sub_kind == 'xmqr' else validate_mosquitto)(output, qos)
                    self.results.append(dict(server=server_kind, publisher=pub_kind, subscriber=sub_kind,
                                             qos=qos, scenario='live_binary_mtls', status='PASS'))
                    retained = self.job(server, volume, pub_kind, 'pub', qos, retain=True)
                    if retained.returncode:
                        raise AssertionError('retained publication failed')
                    replay = self.job(server, volume, sub_kind, 'sub', qos)
                    if replay.returncode:
                        raise AssertionError('retained replay failed')
                    (validate_xmqr if sub_kind == 'xmqr' else validate_mosquitto)(replay.stdout, qos, True)
                    self.results.append(dict(server=server_kind, publisher=pub_kind, subscriber=sub_kind,
                                             qos=qos, scenario='retained_binary_replay', status='PASS'))
                    # Zero-byte retained deletes; --retained-only/hex check is
                    # replaced by a live subscriber barrier after deletion.
                    invoke('run', '--rm', '--network', 'none', '--user', '10001:10001', '--entrypoint', 'sh',
                           '-v', volume+':/run/test', self.images['broker']['id'], '-c', ': > /run/test/payload.bin')
                    deleted = self.job(server, volume, pub_kind, 'pub', 1, retain=True)
                    if deleted.returncode:
                        raise AssertionError('retained deletion failed')
                    invoke('run', '--rm', '--network', 'none', '--user', '10001:10001', '--entrypoint', 'sh',
                           '-v', volume+':/run/test', self.images['broker']['id'], '-c',
                           'printf "phase2\\000\\377\\r\\n" > /run/test/payload.bin')
                    subscriber = self.job(server, volume, sub_kind, 'sub', qos, detached=True)
                    result = self.job(server, volume, pub_kind, 'pub', qos)
                    if result.returncode:
                        raise AssertionError('publication after retained deletion failed')
                    (validate_xmqr if sub_kind == 'xmqr' else validate_mosquitto)(self.finished(subscriber), qos)
                    self.results.append(dict(server=server_kind, publisher=pub_kind, subscriber=sub_kind,
                                             qos=qos, scenario='retained_delete_then_live', status='PASS'))
            for pub_kind, sub_kind in [('xmqr', 'mosquitto'), ('mosquitto', 'xmqr')]:
                recipient = self.job(server, volume, sub_kind, 'sub', 1, detached=True)
                will_source = self.job(server, volume, pub_kind, 'sub', 1, topic='phase2/#', will=True, detached=True)
                invoke('kill', '--signal', 'KILL', will_source)
                (validate_xmqr if sub_kind == 'xmqr' else validate_mosquitto)(
                    self.finished(recipient), 1, payload=b'phase2-will')
                self.results.append(dict(server=server_kind, publisher=pub_kind, subscriber=sub_kind,
                                         scenario='will_after_test_process_kill', status='PASS'))
                identifier = 'c43-'+uuid.uuid4().hex[:12]
                subscriber = self.job(server, volume, sub_kind, 'sub', 1, persistent=identifier, detached=True)
                invoke('stop', '--time', '5', subscriber)
                if json.loads(invoke('inspect', subscriber).stdout)[0]['State']['ExitCode'] != 0:
                    raise AssertionError('persistent subscriber DISCONNECT failed')
                published = self.job(server, volume, pub_kind, 'pub', 1)
                if published.returncode:
                    raise AssertionError('offline publication failed')
                resumed = self.job(server, volume, sub_kind, 'sub', 1, persistent=identifier)
                if resumed.returncode:
                    raise AssertionError('persistent subscriber resume failed')
                (validate_xmqr if sub_kind == 'xmqr' else validate_mosquitto)(resumed.stdout, 1)
                self.results.append(dict(server=server_kind, publisher=pub_kind, subscriber=sub_kind,
                                         scenario='persistent_offline_resume', status='PASS'))
            for kind in ('xmqr', 'mosquitto'):
                for negative in ('ca', 'password'):
                    result = self.job(server, volume, kind, 'pub', 1, negative=negative)
                    if result.returncode == 0 or b'publish_complete' in result.stdout:
                        raise AssertionError('invalid credentials accepted')
                    self.results.append(dict(server=server_kind, client=kind, scenario='reject_'+negative, status='PASS'))
            if server_kind == 'xmqr':
                for kind in ('xmqr', 'mosquitto'):
                    result = self.job(server, volume, kind, 'pub', 1, topic='phase2/denied')
                    if result.returncode == 0:
                        raise AssertionError('XMQR publish ACL bypass')
                    self.results.append(dict(server=server_kind, client=kind, scenario='reject_publish_acl', status='PASS'))
                result = self.job(server, volume, 'xmqr', 'sub', 1, topic='other/#')
                if result.returncode == 0:
                    raise AssertionError('XMQR subscribe ACL bypass')
                self.results.append(dict(server=server_kind, client='xmqr', scenario='reject_subscribe_acl', status='PASS'))
            invoke('stop', '--time', '20', server)
            if json.loads(invoke('inspect', server).stdout)[0]['State']['ExitCode'] != 0:
                raise AssertionError('test broker graceful shutdown failed')
            wrong = self.start_server(server_kind, volume, wrong_name=True)
            for kind in ('xmqr', 'mosquitto'):
                result = self.job(wrong, volume, kind, 'pub', 1)
                if result.returncode == 0 or b'publish_complete' in result.stdout:
                    raise AssertionError('wrong server SAN accepted')
                self.results.append(dict(server=server_kind, client=kind, scenario='reject_server_san', status='PASS'))
            invoke('stop', '--time', '20', wrong)

    def close(self):
        for container in reversed(self.containers):
            invoke('rm', '-f', '-v', container, check=False)
        for volume in reversed(self.volumes):
            invoke('volume', 'rm', volume, check=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--broker-image', default='lepinheiro100/xmqr-broker:0.11.3')
    parser.add_argument('--client-image', default='lepinheiro100/xmqr-client:0.8.1')
    parser.add_argument('--mosquitto-image', default='eclipse-mosquitto:2.0.22')
    parser.add_argument('--hash-file', type=Path, required=True, help='public test fixture hash only')
    parser.add_argument('--openssl', default='openssl')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists() or args.report.is_symlink():
        parser.error('report exists; choose another output')
    images = {name:metadata(reference) for name, reference in [('broker', args.broker_image),
              ('client', args.client_image), ('mosquitto', args.mosquitto_image)]}
    version = invoke('run', '--rm', '--network', 'none', '--entrypoint', 'mosquitto', images['mosquitto']['id'], '-h').stdout
    if b'mosquitto version 2.0.22' not in version:
        parser.error('Mosquitto 2.0.22 required')
    c27spec = importlib.util.spec_from_file_location('c27', ROOT/'validation/C27-integration.py')
    c27 = importlib.util.module_from_spec(c27spec)
    c27spec.loader.exec_module(c27)
    c27.openssl = lambda root, *options: subprocess.run([args.openssl, *options], cwd=root, check=True, capture_output=True, timeout=30)
    campaign = Campaign(args, images)
    try:
        with tempfile.TemporaryDirectory(prefix=campaign.prefix) as temporary:
            root = Path(temporary)
            c27.certificates(root)
            campaign.execute(root)
        report = dict(status='PASS', completed_utc=datetime.now(timezone.utc).isoformat(), images=images,
                      client_source_head=subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip(),
                      harness_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), scenarios=campaign.results,
                      network='test-owned loopback namespaces, no published host ports',
                      limitations=['Not exhaustive MQTT3.1.1 conformance', 'No MQTT5', 'Physical ESP32 NOT_RUN',
                                   'Power-loss, reconnect faults and certificate rotation belong to later campaigns'])
        args.report.parent.mkdir(parents=True, exist_ok=True)
        with args.report.open('x', encoding='utf-8') as stream:
            json.dump(report, stream, indent=2)
            stream.write('\n')
        print('EXTERNAL_MTLS_PHASE2_PASS', len(campaign.results))
    finally:
        campaign.close()


if __name__ == '__main__':
    main()
