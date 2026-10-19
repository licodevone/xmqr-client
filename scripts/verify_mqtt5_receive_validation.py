#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C144: bounded CLI regressions for four P232 MQTT 5 receive recommendations."""
import argparse
import json
from pathlib import Path
import tempfile
import time

from verify_mqtt5_audit_fixes import Peer, digest, frame, packet, text


def connect(peer, connack=b'\x20\x03\0\0\0'):
    peer.stream, _ = peer.server.accept()
    peer.stream.settimeout(3)
    assert packet(peer.stream)[0] == 0x10, 'expected CONNECT'
    peer.stream.sendall(connack)


def rejected(peer):
    started = time.monotonic()
    response = peer.receive()
    # Error DISCONNECT is allowed, but never a success ACK or application flow.
    if response is not None:
        assert response[0] == 0xe0, 'invalid packet was acknowledged or accepted'
        assert len(response[1]) >= 1 and response[1][0] >= 0x80, 'success DISCONNECT'
        assert peer.receive() is None, 'connection did not close after error'
    code = peer.process.wait(timeout=1)
    assert code > 0, f'expected handled error, got exit {code}'
    assert time.monotonic() - started < 1.5, 'rejection waited for operation timeout'
    assert not peer.rows(), 'invalid packet produced successful output'


def user_property(key, value):
    return b'\x26' + text(key) + text(value)


def invalid_connack(binary, folder, wire):
    with Peer(binary, folder, 'pub', ['--qos', '1']) as peer:
        connect(peer, wire)
        rejected(peer)


def invalid_publication(binary, folder, topic=b'audit/topic', properties=b''):
    with Peer(binary, folder, 'sub', ['--qos', '1', '--count', '1']) as peer:
        connect(peer)
        peer.subscribe(1)
        # QoS 1 makes an unwanted PUBACK visible before application output.
        peer.stream.sendall(frame(0x32, text(topic) + b'\0\1'
                                  + bytes([len(properties)]) + properties + b'data'))
        rejected(peer)


def invalid_puback(binary, folder, header=0x40, properties=None):
    with Peer(binary, folder, 'pub', ['--qos', '1']) as peer:
        connect(peer)
        publication = peer.receive()
        assert publication and publication[0] == 0x32, 'canonical publish failed'
        size = int.from_bytes(publication[1][:2], 'big')
        identifier = publication[1][2 + size:4 + size]
        body = identifier
        if properties is not None:
            body += b'\0' + bytes([len(properties)]) + properties
        peer.stream.sendall(frame(header, body))
        rejected(peer)


def canonical_binary_and_unicode(binary, folder):
    with Peer(binary, folder, 'sub', ['--qos', '1', '--count', '1']) as peer:
        connect(peer)
        peer.subscribe(1)
        topic = 'audit/ação/🤖\ufeff'.encode()
        payload = b'\0\xff\xc0\x80'
        properties = user_property(b'', 'ação/🤖\ufeff'.encode())
        properties += b'\x09' + text(payload)
        wire = frame(0x32, text(topic) + b'\0\1' + bytes([len(properties)])
                     + properties + payload)
        peer.stream.sendall(wire[:1])
        peer.stream.sendall(wire[1:])
        assert peer.receive() == (0x40, b'\0\1'), 'canonical PUBACK missing'
        peer.finish_sub()
        rows = peer.rows()
        assert len(rows) == 1 and rows[0]['event'] == 'publish'
        assert rows[0]['topic'].encode() == topic, 'Unicode topic changed'
        assert rows[0]['payload_bytes'] == list(payload), 'binary payload changed'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists() or args.report.is_symlink():
        parser.error('new report path required; previous failures are preserved')
    binary = args.client.resolve(strict=True)
    before = digest(binary)
    cases = [
        ('canonical-unicode-binary', canonical_binary_and_unicode),
        ('nul-topic', lambda b, f: invalid_publication(b, f, b'audit/\0topic')),
        ('redundant-remaining-length', lambda b, f: invalid_connack(b, f, b'\x20\x83\0\0\0\0')),
        ('redundant-property-length', lambda b, f: invalid_connack(b, f, b'\x20\x04\0\0\x80\0')),
        ('redundant-subscription-id', lambda b, f: invalid_publication(b, f, properties=b'\x0b\x81\0')),
    ]
    for header in [0x41, 0x42, 0x4f]:
        cases.append((f'puback-reserved-{header:02x}',
                      lambda b, f, h=header: invalid_puback(b, f, h)))
    for side, key, value in [('key', b'\0', b'value'), ('value', b'key', b'v\0')]:
        properties = user_property(key, value)
        cases += [
            (f'connack-nul-{side}', lambda b, f, p=properties:
             invalid_connack(b, f, frame(0x20, b'\0\0' + bytes([len(p)]) + p))),
            (f'publish-nul-{side}', lambda b, f, p=properties:
             invalid_publication(b, f, properties=p)),
            (f'puback-nul-{side}', lambda b, f, p=properties:
             invalid_puback(b, f, properties=p)),
        ]
    report = dict(schema_version=1, prompt='C144', binary_sha256=before,
                  harness_sha256=digest(Path(__file__)),
                  helper_sha256=digest(Path(__file__).with_name('verify_mqtt5_audit_fixes.py')),
                  scope='synthetic loopback TCP, bounded temporary state; no production broker',
                  scenarios=[])
    began = time.monotonic()
    with tempfile.TemporaryDirectory(prefix='xmqr-c144-receive-') as folder:
        for name, operation in cases:
            try:
                operation(binary, folder)
                row = dict(name=name, status='PASS')
            except Exception as error:
                row = dict(name=name, status='FAIL', error_type=type(error).__name__,
                           reason=str(error)[:180])
            report['scenarios'].append(row)
    report['binary_unchanged'] = digest(binary) == before
    report['counts'] = {s: sum(r['status'] == s for r in report['scenarios'])
                        for s in ('PASS', 'FAIL')}
    report['status'] = 'PASS' if not report['counts']['FAIL'] and report['binary_unchanged'] else 'FAIL'
    report['elapsed_seconds'] = round(time.monotonic() - began, 3)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open('x', encoding='utf-8') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: report[k] for k in ('status', 'counts', 'elapsed_seconds')}))
    return int(report['status'] != 'PASS')


if __name__ == '__main__':
    raise SystemExit(main())
