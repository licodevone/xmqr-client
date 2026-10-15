#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C56: bounded real broker/client checks for the phase-5 CLI slice."""
import argparse
import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time

from verify_phase1 import Lab, TIMEOUT


def formatted_subscriber(lab, name):
    process = subprocess.Popen([
        str(lab.client), 'sub', '--open-lab', '--port', str(lab.port),
        '--topic', 'c56/binary', '--count', '1', '--output', name,
    ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    lab.children.append(process)
    # The status line is emitted on stderr after SUBACK, before publishing.
    if not select.select([process.stderr], [], [], TIMEOUT)[0]:
        raise AssertionError('formatted subscriber readiness timeout')
    line = process.stderr.readline()
    if b'Assinatura ativa' not in line:
        raise AssertionError('formatted subscriber did not become ready')
    return process


def verify(broker, client):
    with tempfile.TemporaryDirectory(prefix='xmqr-c56-') as temporary:
        lab = Lab(broker, client, Path(temporary) / 'state')
        try:
            subscriber = lab.subscriber('c56/#', [
                '--exclude-topic', 'c56/skip', '--count', '1', '--qos', '1'])
            assert lab.pub('c56/skip', b'private-discarded', ['--qos', '1']).returncode == 0
            assert lab.pub('c56/show', b'visible', ['--qos', '1']).returncode == 0
            subscriber.wait(timeout=TIMEOUT)
            assert subscriber.returncode == 0
            lines = lab.output(subscriber).splitlines()
            assert len(lines) == 1 and json.loads(lines[0])['topic'] == 'c56/show'

            subscriber = lab.subscriber('c56/repeat', ['--count', '3', '--qos', '1'])
            started = time.monotonic()
            publisher = subprocess.run(lab.args('pub', 'c56/repeat', [
                '--message', 'R', '--qos', '1', '--repeat', '3',
                '--repeat-interval-ms', '40']), capture_output=True, timeout=TIMEOUT)
            assert publisher.returncode == 0
            assert time.monotonic() - started >= .06
            assert publisher.stdout.count(b'publish_complete') == 3
            subscriber.wait(timeout=TIMEOUT)
            assert subscriber.returncode == 0
            assert len(lab.output(subscriber).splitlines()) == 3

            for name, expected in [
                ('raw', bytes([0, 255, 10, 34])),
                ('hex', b'topic_hex=6335362f62696e617279 payload_hex=00ff0a22 qos=0 retain=false\n'),
            ]:
                subscriber = formatted_subscriber(lab, name)
                assert lab.pub('c56/binary', bytes([0, 255, 10, 34]), []).returncode == 0
                stdout, _ = subscriber.communicate(timeout=TIMEOUT)
                assert subscriber.returncode == 0 and stdout == expected
        finally:
            lab.close()
    return ['exclude/ACK/count:PASS', 'repeat/QoS1/interval:PASS',
            'raw/binary:PASS', 'hex/binary:PASS']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--broker', required=True, type=Path)
    parser.add_argument('--client', required=True, type=Path)
    args = parser.parse_args()
    if os.name != 'posix':
        parser.error('requires isolated Unix broker state')
    for binary in (args.broker, args.client):
        if not binary.is_file() or not os.access(binary, os.X_OK):
            parser.error('explicit executable unavailable')
    for result in verify(args.broker.resolve(), args.client.resolve()):
        print(result)


if __name__ == '__main__':
    main()
