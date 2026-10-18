#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Real migration regression: no child may outlive its private directory."""
import argparse
from pathlib import Path
import sys
import tempfile
from unittest.mock import patch

import verify_c129_v1_migration as migration


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--legacy-client', type=Path, required=True)
    parser.add_argument('--client', type=Path, required=True)
    args = parser.parse_args()
    children = []
    popen = migration.subprocess.Popen
    directory = tempfile.TemporaryDirectory

    def tracked_popen(*values, **options):
        child = popen(*values, **options)
        children.append(child)
        return child

    class CheckedDirectory:
        def __init__(self, *values, **options):
            self.inner = directory(*values, **options)

        def __enter__(self):
            return self.inner.__enter__()

        def __exit__(self, *failure):
            alive = any(child.poll() is None for child in children)
            try:
                assert not alive, 'private directory cleanup started with a live child'
            finally:
                # Deterministic RED must not itself leak a process or directory.
                for child in children:
                    if child.poll() is None:
                        child.kill()
                        child.communicate(timeout=5)
                self.inner.__exit__(*failure)

    with directory(prefix='c131-order-', dir='/var/tmp') as temp:
        report = Path(temp) / 'report.json'
        argv = ['migration', '--legacy-client', str(args.legacy_client),
                '--client', str(args.client), '--report', str(report)]
        with patch.object(migration.subprocess, 'Popen', tracked_popen), \
             patch.object(migration.tempfile, 'TemporaryDirectory', CheckedDirectory), \
             patch.object(sys, 'argv', argv):
            migration.main()
    assert children and all(child.poll() is not None for child in children)
    print('C131_MIGRATION_CLEANUP_PASS', flush=True)


if __name__ == '__main__':
    main()
