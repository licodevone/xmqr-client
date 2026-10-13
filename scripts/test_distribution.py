#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""P78/C42: packaging contracts, without Docker/network/publication."""
import copy
import importlib.util
from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('release_assets', ROOT / 'scripts/release_assets.py')
RELEASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RELEASE)


class DistributionContract(unittest.TestCase):
    def setUp(self):
        self.manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']
        self.dockerfile = (ROOT / 'Dockerfile').read_text(encoding='utf-8')

    def test_default_version_matches_cargo_and_runtime_is_checked(self):
        self.assertEqual(re.search(r'^ARG VERSION=(.+)$', self.dockerfile, re.M)[1], self.manifest['version'])
        name, components = RELEASE.specifications(self.manifest['name'])
        self.assertEqual(name, 'xmqr-broker' if self.manifest['name'] == 'mqtt-broker' else 'xmqr-client')
        for _, binary in components:
            self.assertIn(f'"$(/usr/local/bin/{binary} --version)" = "{binary} ${{VERSION}}"', self.dockerfile)

    def test_last_stage_and_default_entrypoint_are_preserved(self):
        self.assertEqual(re.findall(r'^FROM .+ AS (\S+)$', self.dockerfile, re.M)[-1], 'runtime')
        binary = 'mqtt-broker' if self.manifest['name'] == 'mqtt-broker' else 'mqtt-client'
        self.assertTrue(self.dockerfile.rstrip().endswith(f'ENTRYPOINT ["/usr/local/bin/{binary}"]'))
        self.assertIn('USER 10001:10001', self.dockerfile)
        self.assertIn('STOPSIGNAL SIGTERM', self.dockerfile)

    def test_broker_admin_target_precedes_default_broker(self):
        if self.manifest['name'] != 'mqtt-broker':
            return
        admin = self.dockerfile.split('FROM runtime-base AS admin\n')[1].split('FROM runtime-base AS runtime\n')[0]
        self.assertIn('ENTRYPOINT ["/usr/local/bin/mqtt-admin"]', admin)
        self.assertNotIn('/src/target/release/mqtt-broker', admin)
        self.assertNotIn('VOLUME', admin)

    def test_context_is_allowlist_with_final_secret_exclusions(self):
        rules = [r for r in (ROOT / '.dockerignore').read_text().splitlines() if r and not r.startswith('#')]
        self.assertEqual(rules[0], '**')
        self.assertEqual({r for r in rules if r.startswith('!')}, {
            '!Cargo.toml', '!Cargo.lock', '!Dockerfile', '!.dockerignore', '!LICENSE',
            '!src/', '!src/**', '!docs/', '!docs/third-party-licenses.md'})
        for pattern in ['**/.env*', '**/*.key', '**/*.pem', '**/*.crt', '**/*.wal', '**/*.db']:
            self.assertIn(pattern, rules)
            self.assertGreater(rules.index(pattern), rules.index('!src/**'))

    def test_provenance_accepts_only_expected_version_revision_and_runtime(self):
        metadata = {'Id': 'sha256:fixture', 'Os': 'linux', 'Architecture': 'amd64',
                    'Config': {'User': '10001:10001', 'StopSignal': 'SIGTERM',
                               'Entrypoint': ['/usr/local/bin/mqtt-admin'],
                               'Labels': {'org.opencontainers.image.version': '0.11.3',
                                          'org.opencontainers.image.revision': 'commit-fixture'}}}
        self.assertEqual(RELEASE.validate_image(metadata, 'mqtt-admin', '0.11.3', 'commit-fixture')['binary'], 'mqtt-admin')
        for field, value in [('User', 'root'), ('StopSignal', 'SIGKILL'), ('Entrypoint', ['/bin/sh'])]:
            wrong = copy.deepcopy(metadata)
            wrong['Config'][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                RELEASE.validate_image(wrong, 'mqtt-admin', '0.11.3', 'commit-fixture')
        with self.assertRaises(ValueError):
            RELEASE.validate_image(metadata, 'mqtt-admin', '0.11.4', 'commit-fixture')
        with self.assertRaises(ValueError):
            RELEASE.validate_image(metadata, 'mqtt-admin', '0.11.3', 'another-commit')
        wrong = copy.deepcopy(metadata)
        wrong['Architecture'] = 'arm64'
        with self.assertRaises(ValueError):
            RELEASE.validate_image(wrong, 'mqtt-admin', '0.11.3', 'commit-fixture')


if __name__ == '__main__':
    unittest.main()
