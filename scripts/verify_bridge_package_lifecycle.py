#!/usr/bin/env python3
"""C128: disposable Ubuntu 26.04 upgrade/rollback with retained local config."""
import argparse
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--old-deb", type=Path, required=True)
    parser.add_argument("--new-deb", type=Path, required=True)
    args = parser.parse_args()
    old = args.old_deb.resolve(strict=True)
    new = args.new_deb.resolve(strict=True)
    assert old.name == "xmqr-client_0.38.0-1_amd64.deb"
    assert new.name == "xmqr-client_0.39.0-1_amd64.deb"
    script = """set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends systemd >/tmp/apt-systemd.log
apt-get install -y --no-install-recommends /old/xmqr-client_0.38.0-1_amd64.deb >/tmp/apt-old.log
test "$(mqtt-client --version)" = 'mqtt-client 0.38.0'
test ! -e /usr/bin/mqtt-bridge
printf 'keep\n' >/etc/xmqr-client/operator.keep
chmod 0600 /etc/xmqr-client/operator.keep
apt-get install -y --allow-downgrades --no-install-recommends /new/xmqr-client_0.39.0-1_amd64.deb >/tmp/apt-new.log
test "$(mqtt-client --version)" = 'mqtt-client 0.39.0'
test "$(mqtt-bridge --version)" = 'mqtt-bridge 0.1.0 (xmqr-client 0.39.0)'
test -f /usr/lib/systemd/system/xmqr-client-bridge@.service
test "$(cat /etc/xmqr-client/operator.keep)" = keep
test ! -e /etc/systemd/system/multi-user.target.wants/xmqr-client-bridge@lab.service
systemd-analyze verify /usr/lib/systemd/system/xmqr-client-sub@.service /usr/lib/systemd/system/xmqr-client-pub@.service /usr/lib/systemd/system/xmqr-client-bridge@.service
apt-get install -y --allow-downgrades --no-install-recommends /old/xmqr-client_0.38.0-1_amd64.deb >/tmp/apt-rollback.log
test "$(mqtt-client --version)" = 'mqtt-client 0.38.0'
test ! -e /usr/bin/mqtt-bridge
test "$(cat /etc/xmqr-client/operator.keep)" = keep
apt-get install -y --no-install-recommends /new/xmqr-client_0.39.0-1_amd64.deb >/tmp/apt-upgrade-again.log
test "$(mqtt-bridge --version)" = 'mqtt-bridge 0.1.0 (xmqr-client 0.39.0)'
apt-get purge -y xmqr-client >/tmp/apt-purge.log
test "$(cat /etc/xmqr-client/operator.keep)" = keep
printf 'C133_PACKAGE_LIFECYCLE_PASS\n'
"""
    result = subprocess.run(
        ["docker", "run", "--rm", "--network", "bridge",
         "-v", f"{old.parent}:/old:ro", "-v", f"{new.parent}:/new:ro",
         "ubuntu:26.04", "sh", "-ec", script],
        check=False, capture_output=True, text=True, timeout=180,
    )
    if result.returncode:
        print(result.stdout[-3000:])
        print(result.stderr[-3000:])
        raise SystemExit(result.returncode)
    print(result.stdout.strip())


if __name__ == "__main__":
    main()
