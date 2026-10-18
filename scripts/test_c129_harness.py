# SPDX-License-Identifier: MIT
import unittest

from verify_c129_compatibility import accepted


class CompatibilityGateTests(unittest.TestCase):
    def test_requires_every_full_marker_token(self):
        self.assertTrue(accepted(0, b"FIRST_PASS\nSECOND_PASS ", ["FIRST_PASS", "SECOND_PASS"]))
        self.assertFalse(accepted(0, b"FIRST_PASS", ["FIRST_PASS", "SECOND_PASS"]))

    def test_nonzero_exit_cannot_pass_with_marker(self):
        self.assertFalse(accepted(1, b"FIRST_PASS", ["FIRST_PASS"]))

    def test_lookalike_and_negative_markers_do_not_pass(self):
        for value in [b"NOT_FIRST_PASS", b"FIRST_PASS_FAIL", b"FIRST_PASS:FAILED", b""]:
            self.assertFalse(accepted(0, value, ["FIRST_PASS"]))

    def test_empty_requirement_does_not_imply_pass(self):
        self.assertFalse(accepted(0, b"", []))


if __name__ == "__main__":
    unittest.main()
