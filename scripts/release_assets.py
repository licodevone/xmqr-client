#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Export tested Docker images and a .deb with hashes/provenance; never publish.

Call only from an accepted clean commit, after native/Docker/package gates.
The caller supplies a .deb built from that commit. No credentials are read.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def specifications(package):
    if package == 'mqtt-broker':
        return 'xmqr-broker', [('xmqr-broker', 'mqtt-broker'), ('xmqr-admin', 'mqtt-admin')]
    if package == 'xmqr-client':
        return 'xmqr-client', [('xmqr-client', 'mqtt-client')]
    raise ValueError('unsupported package')


def validate_image(data, binary, version, revision):
    config = data['Config']
    labels = config.get('Labels') or {}
    if data['Os'] != 'linux' or data['Architecture'] != 'amd64':
        raise ValueError('only Linux/amd64 is validated')
    if config['User'] != '10001:10001' or config['StopSignal'] != 'SIGTERM':
        raise ValueError('invalid runtime user/signal')
    if config['Entrypoint'] != ['/usr/local/bin/' + binary]:
        raise ValueError('wrong binary entrypoint')
    if labels.get('org.opencontainers.image.version') != version:
        raise ValueError('image version differs from Cargo.toml')
    if labels.get('org.opencontainers.image.revision') != revision:
        raise ValueError('image was not built from this commit')
    return {'id': data['Id'], 'platform': 'linux/amd64', 'binary': binary,
            'user': config['User'], 'source_revision': revision}


def export_image(reference, destination):
    # Save only this exact image. Never export all tags or credentials.
    with tempfile.TemporaryFile() as errors:
        process = subprocess.Popen(['docker', 'image', 'save', reference],
                                   stdout=subprocess.PIPE, stderr=errors)
        try:
            with destination.open('xb') as output:
                with gzip.GzipFile(fileobj=output, mode='wb', filename='', mtime=0) as compressed:
                    shutil.copyfileobj(process.stdout, compressed)
            if process.wait() != 0:
                raise RuntimeError('docker save failed')
        finally:
            process.stdout.close()
            if process.poll() is None:
                process.kill()
                process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--deb', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=Path('dist'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    package = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['package']
    name, images = specifications(package['name'])
    version = package['version']
    if subprocess.run(['git', 'diff', '--quiet', 'HEAD'], cwd=root).returncode:
        parser.error('tracked changes present: commit the accepted source first')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    if args.deb.name != f'{name}_{version}-1_amd64.deb':
        parser.error('deb filename/version/architecture do not match source')
    with args.deb.open('rb') as stream:
        if stream.read(8) != b'!<arch>\n':
            parser.error('not a Debian package')
    args.output.mkdir(parents=True, exist_ok=True)
    manifest = {'schema_version': 1, 'package': name, 'version': version,
                'source_revision': revision, 'git_tag': 'v' + version,
                'deb_target': 'Ubuntu 26.04 amd64', 'docker_target': 'Debian Bookworm linux/amd64',
                'images': {}, 'artifacts': [], 'experimental': True}
    deb_destination = args.output / args.deb.name
    if deb_destination.resolve() != args.deb.resolve():
        with args.deb.open('rb') as source, deb_destination.open('xb') as output:
            shutil.copyfileobj(source, output)
    artifacts = [deb_destination]
    for repository, binary in images:
        reference = f'lepinheiro100/{repository}:{version}'
        data = json.loads(subprocess.check_output(['docker', 'image', 'inspect', reference]))[0]
        metadata = validate_image(data, binary, version, revision)
        actual = subprocess.check_output(['docker', 'run', '--rm', '--network', 'none',
                                         '--read-only', '--cap-drop=ALL',
                                         '--security-opt', 'no-new-privileges', reference, '--version'], text=True).strip()
        if actual != binary + ' ' + version:
            raise ValueError('binary version does not match image/source')
        destination = args.output / f'{repository}-{version}-linux-amd64.tar.gz'
        export_image(reference, destination)
        metadata['version_output'] = actual
        manifest['images'][reference] = metadata
        artifacts.append(destination)
    for artifact in artifacts:
        checksum = digest(artifact)
        (args.output / (artifact.name + '.sha256')).write_text(
            checksum + '  ' + artifact.name + '\n', encoding='utf-8')
        manifest['artifacts'].append({'file': artifact.name, 'sha256': checksum,
                                     'size': artifact.stat().st_size})
    destination = args.output / f'{name}-{version}-release-manifest.json'
    with destination.open('x', encoding='utf-8') as output:
        json.dump(manifest, output, indent=2)
        output.write('\n')
    (args.output / (destination.name + '.sha256')).write_text(
        digest(destination) + '  ' + destination.name + '\n', encoding='utf-8')
    print(destination)


if __name__ == '__main__':
    main()
