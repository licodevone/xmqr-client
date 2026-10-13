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
