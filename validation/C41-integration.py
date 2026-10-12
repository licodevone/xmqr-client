# SPDX-License-Identifier: MIT
"""C41: retained bootstrap versus live output; explicit separate broker/client."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
CLIENT = None
f = c27 = None


class Contracts(unittest.TestCase):
    def test_filters_preserve_bootstrap_live_count_and_qos(self):
        for qos in (0, 1, 2):
            with self.subTest(qos=qos), tempfile.TemporaryDirectory(prefix='xmqr-c41-') as temporary:
                broker = f.Broker(temporary)
                subscriber = None
                try:
                    broker.start()
                    def cli(command, *tail):
                        return [str(CLIENT), command, '--open-lab', '--host', '127.0.0.1',
                                '--port', str(broker.port), '--topic', 'sensor/value',
                                '--qos', str(qos), '--output', 'jsonl', *tail]
                    def publish(message):
                        result = subprocess.run(cli('pub', '--message', message, '--retain', 'true'),
                                                capture_output=True, text=True, timeout=8)
                        self.assertEqual(result.returncode, 0, result.stderr)
                    publish('bootstrap')
                    # Connection readiness is observed via CONNACK/SUBACK in the CLI.
                    subscriber = c27.Captured(cli('sub', '--ignore-retained', '--count', '1'))
                    subscriber.wait_line(subscriber.stderr, 'Assinatura ativa')
                    publish('live')
                    self.assertEqual(subscriber.finish(), 0, '\n'.join(subscriber.err))
                    rows = [json.loads(line) for line in subscriber.out]
                    self.assertEqual(len(rows), 1)
                    self.assertFalse(rows[0]['retain'])
                    self.assertEqual(bytes(rows[0]['payload_bytes']), b'live')
                    subscriber.close()
                    subscriber = None
                    result = subprocess.run(cli('sub', '--retained-only', '--count', '1'),
                                            capture_output=True, text=True, timeout=8)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    rows = [json.loads(line) for line in result.stdout.splitlines()]
                    self.assertEqual(len(rows), 1)
                    self.assertTrue(rows[0]['retain'])
                    self.assertEqual(bytes(rows[0]['payload_bytes']), b'live')
                finally:
                    if subscriber:
                        subscriber.close()
                    if broker.process is not None:
                        broker.process.terminate()
                        try:
                            self.assertEqual(broker.process.wait(timeout=20), 0)
                        finally:
                            broker.stop()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--broker', type=Path, required=True)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--fixtures', type=Path, required=True)
    args = parser.parse_args()
    CLIENT = args.client.resolve(strict=True)
    for name, path in [('f', args.fixtures), ('c27', Path(__file__).with_name('C27-integration.py'))]:
        spec = importlib.util.spec_from_file_location(name, path.resolve(strict=True))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        globals()[name] = module
    f.BROKER = args.broker.resolve(strict=True)
    unittest.main(argv=[sys.argv[0]])
