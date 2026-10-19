#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C140-C142: independent bounded wire regressions for five P232 client defects."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def frame(header, body):
    length = len(body)
    encoded = bytearray([header])
    while True:
        byte = length % 128
        length //= 128
        encoded.append(byte | (128 if length else 0))
        if not length:
            return bytes(encoded) + body


def packet(stream):
    def take(size):
        data = bytearray()
        while len(data) < size:
            chunk = stream.recv(size - len(data))
            if not chunk:
                raise EOFError('private peer closed')
            data.extend(chunk)
        return bytes(data)
    header = take(1)[0]
    length = 0
    for index in range(4):
        byte = take(1)[0]
        length += (byte & 127) << (7 * index)
        if not byte & 128:
            assert length <= 65536, 'fixture packet bound'
            return header, take(length)
    raise AssertionError('invalid fixture VBI')


def text(value):
    return len(value).to_bytes(2, 'big') + value


def listener():
    server = socket.socket()
    server.bind(('127.0.0.1', 0))
    server.listen(1)
    server.settimeout(4)
    return server


class Peer:
    def __init__(self, binary, folder, mode, extra=(), server=None):
        self.own_server = server is None
        self.server = server or listener()
        self.stream = None
        self.output = tempfile.TemporaryFile(dir=folder)
        self.errors = tempfile.TemporaryFile(dir=folder)
        arguments = [str(binary), mode, '--open-lab', '--protocol-version', '5',
                     '--host', '127.0.0.1', '--port', str(self.server.getsockname()[1]),
                     '--client-id', 'c140-audit', '--topic', 'audit/topic',
                     '--timeout-seconds', '2', '--total-timeout-seconds', '8',
                     '--output', 'jsonl']
        if mode == 'pub':
            arguments += ['--message', 'synthetic']
        environment = {key: value for key, value in os.environ.items()
                       if not key.startswith('MQTT_')}
        self.process = subprocess.Popen(arguments + list(extra), env=environment,
                                        stdout=self.output, stderr=self.errors)

    def __enter__(self):
        return self

    def __exit__(self, *_):
        if self.stream:
            self.stream.close()
        if self.own_server:
            self.server.close()
        if self.process.poll() is None:
            self.process.send_signal(signal.SIGINT)
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=3)
        self.output.close()
        self.errors.close()

    def handshake(self, properties=b'', present=False):
        self.stream, _ = self.server.accept()
        self.stream.settimeout(3)
        assert packet(self.stream)[0] == 0x10, 'expected CONNECT'
        self.stream.sendall(frame(0x20, bytes([int(present), 0, len(properties)]) + properties))

    def receive(self):
        try:
            return packet(self.stream)
        except (EOFError, ConnectionResetError):
            return None

    def wait(self, success):
        code = self.process.wait(timeout=4)
        assert (code == 0) == success, f'unexpected exit {code}'
        return code

    def rows(self):
        self.output.seek(0, 2)
        assert self.output.tell() < 65536, 'output bound'
        self.output.seek(0)
        return [json.loads(line) for line in self.output.read().splitlines()]

    def subscribe(self, qos):
        request = self.receive()
        assert request and request[0] == 0x82, 'expected SUBSCRIBE'
        assert request[1][-1] & 3 == qos
        self.stream.sendall(frame(0x90, request[1][:2] + bytes([0, qos])))

    def finish_sub(self):
        request = self.receive()
        assert request and request[0] == 0xa2, 'expected UNSUBSCRIBE'
        self.stream.sendall(frame(0xb0, request[1][:2] + b'\0\0'))
        assert self.receive() == (0xe0, b'')
        self.wait(True)


def acknowledge_publish(peer, publication, qos, retain):
    assert publication and publication[0] >> 4 == 3, 'expected PUBLISH'
    assert (publication[0] >> 1) & 3 == qos, 'QoS silently changed'
    assert bool(publication[0] & 1) == retain, 'retain silently changed'
    if qos:
        size = int.from_bytes(publication[1][:2], 'big')
        identifier = publication[1][2 + size:4 + size]
        peer.stream.sendall(frame(0x40 if qos == 1 else 0x50, identifier))
        if qos == 2:
            assert peer.receive() == (0x62, identifier)
            peer.stream.sendall(frame(0x70, identifier))


def publish_case(binary, folder, qos, retain, properties, allowed, extra=(), present=False, repeat=1):
    with Peer(binary, folder, 'pub', ['--qos', str(qos), '--retain', str(retain).lower(),
                                     *extra]) as peer:
        peer.handshake(properties, present)
        first = peer.receive()
        if not allowed:
            assert first is None, 'forbidden PUBLISH or other I/O sent'
            peer.wait(False)
            assert not peer.rows(), 'rejected publication claimed success'
        else:
            for index in range(repeat):
                acknowledge_publish(peer, first if index == 0 else peer.receive(), qos, retain)
            assert peer.receive() == (0xe0, b'')
            peer.wait(True)
            assert len(peer.rows()) == repeat


def fresh_subscriber(binary, folder, present, durable=False):
    extra = ['--qos', '0', '--count', '1', '--clean-start', 'false',
             '--session-expiry-seconds', '60']
    if durable:
        extra.remove('--count')
        extra.remove('1')
        directory = Path(folder) / 'fresh-sub'
        directory.mkdir(mode=0o700)
        extra += ['--subscription-state-dir', str(directory)]
    with Peer(binary, folder, 'sub', extra) as peer:
        peer.handshake(present=present)
        if present:
            assert peer.receive() is None, 'fresh subscriber accepted previous session'
            peer.wait(False)
        else:
            peer.subscribe(0)
            peer.stream.sendall(frame(0x30, text(b'audit/topic') + b'\0last'))
            peer.finish_sub()


def qos2_case(binary, folder, duplicate=False, unknown=False, reuse=False):
    count = 3 if reuse else 2
    with Peer(binary, folder, 'sub', ['--qos', '2', '--count', str(count)]) as peer:
        peer.handshake()
        peer.subscribe(2)
        identifier = b'\xff\xff'
        if unknown:
            peer.stream.sendall(frame(0x62, b'\0\x49'))
            assert peer.receive() == (0x70, b'\0\x49\x92\0')
        body = text(b'audit/topic') + identifier + b'\0first'
        peer.stream.sendall(frame(0x34, body))
        assert peer.receive() == (0x50, identifier)
        if duplicate:
            # Synthetic input tests receiver obligation, not sender retry permission.
            peer.stream.sendall(frame(0x3c, body))
            assert peer.receive() == (0x50, identifier)
        peer.stream.sendall(frame(0x62, identifier))
        assert peer.receive() == (0x70, identifier)
        peer.stream.sendall(frame(0x62, identifier))
        assert peer.receive() == (0x70, identifier + b'\x92\0')
        if reuse:
            peer.stream.sendall(frame(0x34, text(b'audit/topic') + identifier + b'\0new'))
            assert peer.receive() == (0x50, identifier)
            peer.stream.sendall(frame(0x62, identifier))
            assert peer.receive() == (0x70, identifier)
        peer.stream.sendall(frame(0x30, text(b'audit/topic') + b'\0last'))
        peer.finish_sub()
        payloads = [bytes(row['payload_bytes']) for row in peer.rows() if row.get('event') == 'publish']
        assert payloads == ([b'first', b'new', b'last'] if reuse else [b'first', b'last'])


def outbox_capability_rejection(binary, folder, qos, retain=False):
    directory = Path(folder) / f'outbox-{qos}-{retain}'
    directory.mkdir(mode=0o700)
    extra = ['--outbox-dir', str(directory), '--clean-start', 'false', '--session-expiry-seconds', '60']
    filename = directory / ('pending.json' if qos == 1 else 'pending-qos2.json')
    with listener() as server:
        before = None
        for _ in range(2):
            with Peer(binary, folder, 'pub', ['--qos', str(qos), '--retain', str(retain).lower(), *extra], server) as peer:
                peer.handshake(b'\x25\0' if retain else b'\x24\0')
                assert peer.receive() is None, 'outbox sent unsupported PUBLISH'
                peer.wait(False)
            current = digest(filename)
            if before is not None:
                assert current == before, 'refusal changed existing pending publication'
            before = current
            assert json.loads(filename.read_text())['pending']['stage'] == 'Prepared'


def pubrel_resume_under_restricted_capabilities(binary, folder):
    directory = Path(folder) / 'pubrel-outbox'
    directory.mkdir(mode=0o700)
    extra = ['--qos', '2', '--retain', 'true', '--outbox-dir', str(directory),
             '--clean-start', 'false', '--session-expiry-seconds', '60']
    with listener() as server:
        with Peer(binary, folder, 'pub', extra, server) as peer:
            peer.handshake()
            publication = peer.receive()
            assert publication[0] == 0x35
            size = int.from_bytes(publication[1][:2], 'big')
            identifier = publication[1][2 + size:4 + size]
            peer.stream.sendall(frame(0x50, identifier))
            assert peer.receive() == (0x62, identifier)
            peer.stream.close()
            peer.wait(False)
        filename = directory / 'pending-qos2.json'
        assert json.loads(filename.read_text())['pending']['stage'] == 'PubRelReady'
        with Peer(binary, folder, 'pub', extra, server) as peer:
            peer.handshake(b'\x24\0\x25\0', present=True)
            assert peer.receive() == (0x62, identifier), 'existing PUBREL incorrectly treated as new PUBLISH'
            peer.stream.sendall(frame(0x70, identifier))
            assert peer.receive() == (0xe0, b'')
            peer.wait(True)
        assert not filename.exists()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists() or args.report.is_symlink():
        parser.error('report must be new')
    binary = args.client.resolve(strict=True)
    before = digest(binary)
    report = {'suite': 'C140-C142', 'binary_sha256': before,
              'version': subprocess.check_output([str(binary), '--version'], text=True, timeout=5).strip(),
              'harness_sha256': digest(__file__), 'scenarios': [],
              'limitations': ['synthetic TCP loopback, not TLS/certification or business exactly-once',
                              'four P232 receiving recommendations are not included']}
    cases = []
    for qos in range(3):
        for retain in [False, True]:
            cases.append((f'default-qos{qos}-retain{retain}', lambda f, q=qos, r=retain:
                          publish_case(binary, f, q, r, b'', True)))
            cases.append((f'maximum0-qos{qos}-retain{retain}', lambda f, q=qos, r=retain:
                          publish_case(binary, f, q, r, b'\x24\0', q == 0)))
            cases.append((f'maximum1-qos{qos}-retain{retain}', lambda f, q=qos, r=retain:
                          publish_case(binary, f, q, r, b'\x24\1', q <= 1)))
            cases.append((f'no-retain-qos{qos}-retain{retain}', lambda f, q=qos, r=retain:
                          publish_case(binary, f, q, r, b'\x25\0', not r)))
    cases += [
        ('invalid-maximum2', lambda f: publish_case(binary, f, 0, False, b'\x24\2', False)),
        ('invalid-retain2', lambda f: publish_case(binary, f, 0, False, b'\x25\2', False)),
        ('repeat-allowed', lambda f: publish_case(binary, f, 1, True, b'\x24\1\x25\1', True,
                                               ['--repeat', '3', '--repeat-interval-ms', '1'], repeat=3)),
        ('repeat-rejected', lambda f: publish_case(binary, f, 1, True, b'\x25\0', False,
                                                 ['--repeat', '3', '--repeat-interval-ms', '1'])),
        ('fresh-pub-present', lambda f: publish_case(binary, f, 1, False, b'', False,
            ['--clean-start', 'false', '--session-expiry-seconds', '60'], present=True)),
        ('fresh-pub-absent', lambda f: publish_case(binary, f, 1, False, b'', True,
            ['--clean-start', 'false', '--session-expiry-seconds', '60'])),
        ('fresh-sub-present', lambda f: fresh_subscriber(binary, f, True)),
        ('fresh-sub-absent', lambda f: fresh_subscriber(binary, f, False)),
        ('qos2-cycle-repeat-pubrel', lambda f: qos2_case(binary, f)),
        ('qos2-duplicate', lambda f: qos2_case(binary, f, duplicate=True)),
        ('qos2-unknown-pubrel', lambda f: qos2_case(binary, f, unknown=True)),
        ('qos2-pid-reuse', lambda f: qos2_case(binary, f, duplicate=True, reuse=True)),
    ]
    if os.name == 'posix':
        cases += [('first-sub-manifest-present', lambda f: fresh_subscriber(binary, f, True, True)),
                  ('outbox-qos1-capability', lambda f: outbox_capability_rejection(binary, f, 1)),
                  ('outbox-qos2-capability', lambda f: outbox_capability_rejection(binary, f, 2)),
                  ('outbox-qos1-retain', lambda f: outbox_capability_rejection(binary, f, 1, True)),
                  ('outbox-qos2-retain', lambda f: outbox_capability_rejection(binary, f, 2, True)),
                  ('pubrel-resume-restricted', lambda f: pubrel_resume_under_restricted_capabilities(binary, f))]
    began = time.monotonic()
    with tempfile.TemporaryDirectory(prefix='xmqr-c140-audit-') as folder:
        for name, operation in cases:
            try:
                operation(folder)
                row = {'name': name, 'status': 'PASS'}
            except Exception as error:
                row = {'name': name, 'status': 'FAIL', 'error_type': type(error).__name__,
                       'reason': str(error)[:180]}
            report['scenarios'].append(row)
    report['binary_unchanged'] = digest(binary) == before
    report['counts'] = {status: sum(row['status'] == status for row in report['scenarios'])
                        for status in ('PASS', 'FAIL')}
    report['elapsed_seconds'] = round(time.monotonic() - began, 3)
    report['status'] = 'PASS' if report['counts']['FAIL'] == 0 and report['binary_unchanged'] else 'FAIL'
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open('x', encoding='utf-8') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({key: report[key] for key in ('status', 'counts', 'elapsed_seconds')}))
    return int(report['status'] != 'PASS')


if __name__ == '__main__':
    raise SystemExit(main())
