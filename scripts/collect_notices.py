#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Copy original Cargo dependency notices, including vendored subdirectories.

Only dependency roots returned by locked Cargo metadata are read. Paths and
hashes in the output are relative; no workstation paths or credentials appear.
This deliberately includes development/other-platform dependencies as a superset.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def notice_name(name):
    upper = name.upper()
    return any(upper == word or upper.startswith(word + suffix)
               for word in ('LICENSE', 'LICENCE', 'COPYING', 'NOTICE', 'COPYRIGHT', 'AUTHORS', 'CONTRIBUTORS')
               for suffix in ('.', '-', '_'))


def notice_paths(package):
    manifest = Path(package['manifest_path'])
    if manifest.is_symlink():
        raise ValueError('symlink dependency manifest is not allowed')
    base = manifest.parent.resolve(strict=True)
    selected = set()
    declared = package.get('license_file')
    if declared:
        candidate = Path(declared)
        candidate = candidate if candidate.is_absolute() else base / candidate
        if candidate.is_symlink() or not candidate.is_file():
            raise ValueError('invalid declared dependency license_file')
        candidate.resolve(strict=True).relative_to(base)
        selected.add(candidate.resolve())
    for directory, folders, files in os.walk(base, followlinks=False):
        parent = Path(directory)
        folders[:] = [name for name in folders if name not in ('.git', 'target')]
        if any((parent / name).is_symlink() for name in folders):
            raise ValueError('symlink in dependency source tree')
        for name in files:
            if not notice_name(name):
                continue
            candidate = parent / name
            if candidate.is_symlink():
                raise ValueError('symlink dependency notice is not allowed')
            candidate.resolve(strict=True).relative_to(base)
            selected.add(candidate.resolve())
    if not selected:
        raise ValueError('dependency has no original license/notice text: ' + package['name'])
    return base, sorted(selected)


def supplement_paths(root, package, packages):
    key = package['name'], package['version']
    if key == ('windows-permissions', '0.2.4'):
        repository = 'https://github.com/danieldulaney/windows-permissions-rs'
        if (package.get('license') != 'MIT' or package.get('repository') != repository
                or package.get('source') != 'registry+https://github.com/rust-lang/crates.io-index'):
            raise ValueError('windows-permissions source/license does not match reviewed crate')
        base = root / 'packaging/notices/windows-permissions-0.2.4'
        crate = Path(package['manifest_path']).parent
        expected = {
            'LICENSE-MIT': '508a77d2e7b51d98adeed32648ad124b7b30241a8e70b2e72c99f92d8e5874d1',
            'Cargo.toml': '7d7095e9b45322206e2e46466eafd90581f34dfab8a9c046ed4a5da1054f88a9',
            'Cargo.toml.orig': '6b94500e0e663637e2ea697d70b7910ab341c03b2074116d39169aa9c5deac2a',
        }
        selected = []
        for name, checksum in expected.items():
            path = base / name
            if path.is_symlink() or digest(path) != checksum:
                raise ValueError('windows-permissions reviewed notice hash mismatch: ' + name)
            if name != 'LICENSE-MIT' and digest(crate / name) != checksum:
                raise ValueError('windows-permissions crate attribution hash mismatch: ' + name)
            selected.append(path)
        return base, selected, {
            'repository': repository,
            'archive_sha256': '9e2ccdc3c6bf4d4a094e031b63fadd08d8e42abd259940eb8aa5fdc09d4bf9be',
            'scope': 'SPDX MIT text and unmodified author attribution from the crate manifests',
        }
    if key == ('rumqttc', '0.25.1'):
        vcs = json.loads((Path(package['manifest_path']).parent / '.cargo_vcs_info.json').read_text())
        commit = 'f1e9e8d558783f942993046679cdf3c8c3a3d36b'
        repository = 'https://github.com/bytebeamio/rumqtt'
        if (vcs['git']['sha1'] != commit or vcs.get('path_in_vcs') != 'rumqttc'
                or package.get('license') != 'Apache-2.0' or package.get('repository') != repository):
            raise ValueError('rumqttc supplement source/license does not match locked crate')
        base = root / 'packaging/notices/rumqttc-0.25.1'
        selected = [base / 'LICENSE']
        if selected[0].is_symlink() or digest(selected[0]) != 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30':
            raise ValueError('rumqttc original license hash mismatch')
        return base, selected, {'source_revision': commit, 'repository': repository,
                                'url': repository + '/blob/' + commit + '/LICENSE',
                                'license_git_blob': 'd645695673349e3947e8e5ae42332d0ac3164cd7',
                                'scope': 'original monorepo LICENSE; crate omits file; no NOTICE in reviewed tree'}
    if key == ('valuable', '0.1.1'):
        vcs = json.loads((Path(package['manifest_path']).parent / '.cargo_vcs_info.json').read_text())
        commit = '9efc29b6e58cef28f6566a47aa7e142a55fead77'
        if vcs['git']['sha1'] != commit or package.get('license') != 'MIT':
            raise ValueError('supplement source/license does not match locked crate')
        base = root / 'packaging/notices/valuable-0.1.1'
        selected = [base / 'LICENSE']
        if selected[0].is_symlink() or digest(selected[0]) != 'ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44':
            raise ValueError('supplement license hash mismatch')
        return base, selected, {'source_revision': commit,
                                'url': 'https://raw.githubusercontent.com/tokio-rs/valuable/' + commit + '/LICENSE'}
    if key == ('pharos', '0.5.3'):
        base = Path(package['manifest_path']).parent.resolve()
        vcs = json.loads((base / '.cargo_vcs_info.json').read_text())
        commit = 'f5b985a59cfeced5ae66325d477b6c982c9408bf'
        repository = 'https://github.com/najamelan/pharos'
        if (vcs['git']['sha1'] != commit or package.get('license') != 'Unlicense'
                or package.get('repository') != repository):
            raise ValueError('pharos supplement source/license does not match locked crate')
        readme = base / 'README.md'
        if (readme.is_symlink() or digest(readme) != '19578b95fc9fe905a8b1b2cdf27a32fd4df37c10f7002a18e6f2044209f5f331'
                or '## License\n\n[Unlicence](https://unlicense.org/)' not in readme.read_text()):
            raise ValueError('pharos original license declaration mismatch')
        return base, [readme], {'source_revision': commit, 'repository': repository,
                                'url': repository + '/blob/' + commit + '/README.md#license',
                                'scope': 'original README license declaration; source tree has no LICENSE file'}
    if key == ('ws_stream_tungstenite', '0.15.0'):
        base = Path(package['manifest_path']).parent.resolve()
        vcs = json.loads((base / '.cargo_vcs_info.json').read_text())
        commit = '508c0f28037b5decab5eb61ac4ab805f05c96cdf'
        repository = 'https://github.com/najamelan/ws_stream_tungstenite'
        if (vcs['git']['sha1'] != commit or vcs.get('path_in_vcs') != ''
                or package.get('license') != 'Unlicense'
                or package.get('repository') != repository):
            raise ValueError('ws_stream_tungstenite supplement source/license does not match locked crate')
        readme = base / 'README.md'
        if (readme.is_symlink() or digest(readme) != '3d73da6e62bbed9fb2f082398be3f178b16f5e163415199494c87ee3bd0962d6'
                or '## License\n\n[Unlicence](https://unlicense.org/)' not in readme.read_text()):
            raise ValueError('ws_stream_tungstenite original license declaration mismatch')
        return base, [readme], {'source_revision': commit, 'repository': repository,
                                'url': repository + '/blob/' + commit + '/README.md#license',
                                'scope': 'original README license declaration; source tree has no LICENSE file'}
    if key in {('winapi-i686-pc-windows-gnu', '0.4.0'), ('winapi-x86_64-pc-windows-gnu', '0.4.0')}:
        repository = 'https://github.com/retep998/winapi-rs'
        donor = next(p for p in packages if (p['name'], p['version']) == ('winapi', '0.3.9'))
        if donor.get('repository') != repository or package.get('repository') != repository:
            raise ValueError('shared notice repository does not match')
        if package.get('license') not in ('MIT/Apache-2.0', 'MIT OR Apache-2.0'):
            raise ValueError('shared notice declaration does not match')
        base = Path(donor['manifest_path']).parent.resolve()
        vcs = json.loads((base / '.cargo_vcs_info.json').read_text())
        commit = '796a8e6c2971dc2ff1bcff166e6671284f9b5b6b'
        if vcs['git']['sha1'] != commit:
            raise ValueError('shared notice donor source does not match')
        selected = [base / name for name in ('LICENSE-MIT', 'LICENSE-APACHE')]
        if any(p.is_symlink() or not p.is_file() for p in selected):
            raise ValueError('shared notice donor license is not regular')
        return base, selected, {'donor_package': 'winapi 0.3.9', 'donor_source_revision': commit,
                                'repository': repository, 'scope': 'same-repository license texts; Windows superset only'}
    raise ValueError('no reviewed notice supplement for dependency: ' + package['name'])


def collect(root, output):
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=root))
    if output.exists():
        raise ValueError('notice destination exists; use a new owned output directory')
    output.mkdir(parents=True)
    manifest = {'schema_version': 1, 'scope': 'Cargo.lock dependency superset; no relicensing',
                'packages': []}
    for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'], p['id'])):
        if package['id'] in metadata['workspace_members']:
            continue
        supplement = None
        try:
            base, selected = notice_paths(package)
        except ValueError as error:
            if not str(error).startswith('dependency has no original license/notice text:'):
                raise
            base, selected, supplement = supplement_paths(root, package, metadata['packages'])
        name = package['name'] + '-' + package['version']
        if '/' in name or '\\' in name or name in ('.', '..'):
            raise ValueError('unsafe package name')
        entry = {'name': package['name'], 'version': package['version'],
                 'declared_license': package.get('license'), 'source': package.get('source'),
                 'files': []}
        if supplement:
            entry['supplement_provenance'] = supplement
        for source in selected:
            relative = source.relative_to(base)
            target = output / name / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            with source.open('rb') as src, target.open('xb') as dst:
                shutil.copyfileobj(src, dst)
            target.chmod(0o644)
            checksum = digest(source)
            if digest(target) != checksum:
                raise ValueError('notice copy hash mismatch')
            entry['files'].append({'path': (Path(name) / relative).as_posix(),
                                   'sha256': checksum, 'size': target.stat().st_size})
        manifest['packages'].append(entry)
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = collect(Path(__file__).resolve().parents[1], args.output.resolve())
    print(json.dumps({'packages': len(report['packages']),
                      'notices': sum(len(p['files']) for p in report['packages'])}))


if __name__ == '__main__':
    main()
