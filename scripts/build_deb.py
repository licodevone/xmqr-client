#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Build a native Ubuntu 26.04 package; never install or enable services."""
import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--bin-dir', type=Path)
    parser.add_argument('--output', type=Path, default=Path('dist'))
    parser.add_argument('--revision', default='1')
    args = parser.parse_args()
    if not args.revision.isdecimal():
        parser.error('revision must be numeric')
    release = dict(line.split('=', 1) for line in Path('/etc/os-release').read_text().splitlines() if '=' in line)
    if release.get('ID', '').strip('"') != 'ubuntu' or release.get('VERSION_ID', '').strip('"') != '26.04':
        parser.error('build requires Ubuntu 26.04; no compatibility claim for other releases')
    arch = subprocess.check_output(['dpkg', '--print-architecture'], text=True).strip()
    if arch != 'amd64':
        parser.error('only native amd64 is validated')
    root = Path(__file__).resolve().parents[1]
    manifest = tomllib.loads((root / 'Cargo.toml').read_text())['package']
    broker = manifest['name'] == 'mqtt-broker'
    name = 'xmqr-broker' if broker else 'xmqr-client'
    account = 'xmqr' if broker else 'xmqr-client'
    version = manifest['version'] + '-' + args.revision
    if not args.skip_build:
        run('cargo', 'build', '--locked', '--release', '--bins', cwd=root)
    bindir = args.bin_dir or Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')) / 'release'
    bindir = bindir.resolve()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='xmqr-deb-') as temp:
        stage = Path(temp) / 'package'
        def put(source, destination, mode=0o644):
            target = stage / destination
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            target.chmod(mode)
        binaries = ['mqtt-broker', 'mqtt-admin'] if broker else ['mqtt-client']
        for binary in binaries:
            put(bindir / binary, 'usr/bin/' + binary, 0o755)
        for unit in (root / 'packaging/systemd').glob('*.service'):
            put(unit, 'usr/lib/systemd/system/' + unit.name)
        for file in ['LICENSE', 'docs/third-party-licenses.md', 'docs/ubuntu-systemd.md']:
            put(root / file, 'usr/share/doc/' + name + '/' + Path(file).name)
        control = stage / 'DEBIAN'
        control.mkdir()
        # Let dpkg derive ABI dependencies from the actual ELF files.
        (Path(temp) / 'debian').mkdir()
        (Path(temp) / 'debian/control').write_text(f'Source: {name}\n\nPackage: {name}\nArchitecture: any\nDescription: XMQR\n')
        deps = subprocess.check_output(['dpkg-shlibdeps', '-O', *['-e' + str(stage / 'usr/bin' / b) for b in binaries]], cwd=temp, text=True)
        deps = next(line.split('=', 1)[1] for line in deps.splitlines() if line.startswith('shlibs:Depends='))
        (control / 'control').write_text(f'Package: {name}\nVersion: {version}\nArchitecture: {arch}\nMaintainer: XMQR maintainers <xmqr@localhost>\nDepends: {deps}, adduser, init-system-helpers\nSection: net\nPriority: optional\nHomepage: https://github.com/licodevone/{"xmqr" if broker else "xmqr-client"}\nDescription: Experimental MQTT 3.1.1 {"broker" if broker else "client"}\n Built and validated for Ubuntu 26.04 amd64.\n')
        if broker:
            put(root / 'packaging/broker.env', 'etc/xmqr/broker.env')
            (control / 'conffiles').write_text('/etc/xmqr/broker.env\n')
        else:
            put(root / 'packaging/client.conf.example', 'usr/share/doc/xmqr-client/client.conf.example')
        postinst = f'''#!/bin/sh
set -eu
if [ "$1" = configure ]; then
    if ! getent passwd {account} >/dev/null; then
        adduser --system --group --no-create-home --home /nonexistent {account}
    fi
    install -d -o root -g {account} -m 0750 /etc/{account}
'''
        if broker:
            postinst += '    install -d -o xmqr -g xmqr -m 0700 /var/lib/xmqr\n'
        postinst += '''    if [ -d /run/systemd/system ]; then systemctl daemon-reload; fi
    echo 'XMQR installed; configure credentials, then enable/start explicitly.'
fi
'''
        units = 'xmqr-broker.service' if broker else ''
        stop = f'    deb-systemd-invoke stop {units}\n' if broker else '''    if [ -d /run/systemd/system ]; then
        for unit in $(systemctl list-units --all --plain --no-legend 'xmqr-client-sub@*.service' 'xmqr-client-pub@*.service' | awk '{print $1}'); do
            deb-systemd-invoke stop "$unit"
        done
    fi
'''
        prerm = '#!/bin/sh\nset -eu\ncase "$1" in remove|upgrade|deconfigure)\n' + stop + ';;\nesac\n'
        postrm = '''#!/bin/sh
set -eu
if [ -d /run/systemd/system ]; then systemctl daemon-reload; fi
# Deliberately preserve credentials, service account and MQTT data, even on purge.
'''
        if broker:
            postrm += 'if [ "$1" = remove ] || [ "$1" = purge ]; then deb-systemd-helper disable xmqr-broker.service; fi\n'
        for filename, content in [('postinst', postinst), ('prerm', prerm), ('postrm', postrm)]:
            path = control / filename
            path.write_text(content)
            path.chmod(0o755)
            run('sh', '-n', str(path))
        deb = out / f'{name}_{version}_{arch}.deb'
        run('dpkg-deb', '--root-owner-group', '--build', str(stage), str(deb))
    digest = hashlib.sha256(deb.read_bytes()).hexdigest()
    deb.with_suffix('.deb.sha256').write_text(f'{digest}  {deb.name}\n')
    print(deb)


if __name__ == '__main__':
    main()
