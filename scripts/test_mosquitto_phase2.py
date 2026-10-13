# SPDX-License-Identifier: MIT
"""C43 independent output/CLI contracts, without Docker or network."""
from pathlib import Path
import json
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
import verify_mosquitto_phase2 as b


class Contracts(unittest.TestCase):
    def test_binary_output_not_lossily_decoded(self):
        event = dict(event='publish', topic='phase2/value', qos=2, retain=False, payload_bytes=list(b.PAYLOAD))
        b.validate_xmqr(json.dumps(event).encode(), 2)
        event['payload_bytes'] = list(b'wrong')
        with self.assertRaisesRegex(AssertionError, 'mismatch'):
            b.validate_xmqr(json.dumps(event).encode(), 2)

    def test_qos_and_retained_are_checked_not_just_payload(self):
        output = f'DATA|phase2/value|1|1|{b.PAYLOAD.hex()}\r\n'.encode()
        b.validate_mosquitto(output, 1, True)
        for qos, retained in [(0, True), (1, False)]:
            with self.assertRaisesRegex(AssertionError, 'mismatch'):
                b.validate_mosquitto(output, qos, retained)

    def test_duplicate_delivery_cannot_pass(self):
        output = f'DATA|phase2/value|1|0|{b.PAYLOAD.hex()}\n'.encode()
        with self.assertRaisesRegex(AssertionError, 'mismatch'):
            b.validate_mosquitto(output*2, 1)

    def test_mosquitto_uses_explicit_clientid_and_no_insecure(self):
        campaign = b.Campaign(None, dict(mosquitto=dict(id='fixture'), client=dict(id='fixture')))
        for kind in ('xmqr', 'mosquitto'):
            _, _, args = campaign.client_args(kind, 'sub', 1)
            self.assertNotIn('--insecure', args)
            if kind == 'mosquitto':
                self.assertIn('-i', args)
                self.assertTrue(args[args.index('-i')+1])
            _, _, persistent_args = campaign.client_args(kind, 'sub', 1, persistent='c43-abcdef012345')
            field = '--client-id' if kind == 'xmqr' else '-i'
            self.assertLessEqual(len(persistent_args[persistent_args.index(field)+1]), 23)

    def test_existing_report_refused_before_docker(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary)/'report.json'
            report.write_text('preserve')
            result = subprocess.run([sys.executable, '-B', str(Path(b.__file__)), '--hash-file',
                                     'not-read', '--report', str(report)], capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(report.read_text(), 'preserve')


if __name__ == '__main__':
    unittest.main()
