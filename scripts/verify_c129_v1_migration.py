#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C129: migrate an actual draft-v1 inbound exchange produced by the old ELF."""
import argparse
import contextlib
import hashlib
import json
from pathlib import Path
import select
import subprocess
import tempfile
import time

from verify_subscription_inbound_journal_mqtt5 import activate, arguments, cleanup, line, listener, packet, ready


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--legacy-client', type=Path, required=True)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error('new report required; previous evidence is preserved')
    legacy = args.legacy_client.resolve(strict=True)
    current = args.client.resolve(strict=True)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    report = dict(status='RUNNING', legacy_client_sha256=digest(legacy),
                  client_sha256=digest(current), harness_sha256=digest(Path(__file__)),
                  helper_sha256=digest(Path(__file__).with_name('verify_subscription_inbound_journal_mqtt5.py')),
                  scope='actual draft-v1 ELF; owned TCP fixture; SIGKILL is not power loss')
    began = time.monotonic()
    child = None
    try:
        with tempfile.TemporaryDirectory(prefix='xmqr-c129-v1-', dir='/var/tmp') as temporary, listener() as server, contextlib.ExitStack() as resources:
            # Stop the producer before listener/directory teardown, including
            # assertion failures. It can still commit cleanup after peer EOF.
            resources.callback(lambda: cleanup(child))
            root = Path(temporary)
            child, peer = activate(legacy, root, server)
            try:
                properties = b'\x0b\x01\x0b\x02'
                body = b'\0\x08c129/two\0\x07' + bytes([len(properties)]) + properties + b'owned-legacy'
                peer.sendall(bytes([0x34, len(body)]) + body)
                assert line(child)['metadata']['subscription_identifiers'] == [1, 2]
                assert packet(peer) == (0x50, b'\0\x07')
                child.kill()
                child.communicate(timeout=5)
            finally:
                peer.close()
            path = root / 'subscription-inbound.json'
            before = path.read_bytes()
            document = json.loads(before)
            assert document['version'] == 1, 'legacy ELF did not produce a v1 journal'
            original_wire = document['entries'][0]['publication']['wire']
            child = subprocess.Popen(arguments(current, root, server), stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE, bufsize=0)
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b'\x20\x03\x01\0\0')
                ready(child)
                assert path.read_bytes() == before, 'loading v1 rewrote the draft journal'
                body = b'\0\x08c129/two\0\x07\0owned-legacy'
                peer.sendall(bytes([0x3c, len(body)]) + body)
                assert packet(peer) == (0x50, b'\0\x07')
                assert not select.select([child.stdout], [], [], .1)[0], 'v1 replay duplicated output'
                assert path.read_bytes() == before, 'replay changed original v1 bytes before durable transition'
                peer.sendall(b'\x62\x02\0\x07')
                assert packet(peer) == (0x70, b'\0\x07')
                document = json.loads(path.read_bytes())
                assert document['version'] == 2, 'completion did not commit v2'
                if document['entries']:
                    assert document['entries'][0]['publication']['wire'] == original_wire
                assert child.poll() is None
            report.update(status='PASS', original_metadata_preserved=True,
                          read_and_replay_preserved_v1_bytes=True, completed_document_version=2,
                          duplicate_output=False)
    except Exception as failure:
        report.update(status='FAIL', error_type=type(failure).__name__,
                      reason=str(failure) if isinstance(failure, AssertionError) else 'bounded private fixture failed')
    finally:
        try:
            cleanup(child)
            report['cleanup'] = 'PASS'
        except Exception as failure:
            report.update(status='FAIL', cleanup='FAIL', cleanup_error_type=type(failure).__name__)
    report['elapsed_seconds'] = round(time.monotonic() - began, 4)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open('x', encoding='utf-8') as output:
        json.dump(report, output, indent=2)
        output.write('\n')
    print(json.dumps(report), flush=True)
    if report['status'] != 'PASS':
        raise SystemExit(1)
    print('C129_V1_MIGRATION_PASS', flush=True)


if __name__ == '__main__':
    main()
