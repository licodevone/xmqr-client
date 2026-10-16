#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""P119/P120/C60/C61: isolated broker/client MQTT 5 ACL reason-code contract."""

import argparse
import os
from pathlib import Path
import re
import shlex
import socket
import subprocess
import tempfile
import time


def hash_password(admin: Path) -> str:
    command = f"{shlex.quote(str(admin))} hash-password"
    result = subprocess.run(
        ["script", "-q", "-e", "-c", command, "/dev/null"],
        input="fixture-pass\nfixture-pass\n", text=True, capture_output=True,
        check=True, timeout=20,
    )
    match = re.search(r'password_hash = "(\$argon2id\$[^"\r\n]+)"', result.stdout)
    assert match, "fixture password hash unavailable"
    return match.group(1)


def private_file(path: Path, content: str) -> None:
    path.write_text(content)
    path.chmod(0o600)


def run_client(client: Path, port: int, password: Path, mode: str, topic: str):
    command = [str(client), mode, "--plain-auth-lab", "--username", "fixture",
               "--password-file", str(password), "--protocol-version", "5",
               "--host", "127.0.0.1", "--port", str(port), "--client-id",
               f"fx-{mode}-{topic.split('/')[0]}", "--topic", topic,
               "--qos", "1", "--timeout-seconds", "3"]
    if mode == "pub":
        command += ["--message", "fixture-message"]
    else:
        command += ["--count", "1"]
    return subprocess.run(command, capture_output=True, text=True, timeout=8)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker", type=Path, required=True)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--admin", type=Path, required=True)
    args = parser.parse_args()
    broker, client, admin = (path.resolve() for path in
                             (args.broker, args.client, args.admin))
    with tempfile.TemporaryDirectory(prefix="xmqr-mqtt5-acl-") as temporary:
        root = Path(temporary)
        state = root / "state"
        state.mkdir()
        users, acl, password, wrong_password = (root / name for name in
                                               ("users.toml", "acl.toml",
                                                "password", "wrong-password"))
        private_file(users, "[[users]]\nusername='fixture'\npassword_hash='" +
                     hash_password(admin) + "'\n")
        private_file(acl, "[[rules]]\nusername='fixture'\n"
                     "publish=['allowed/topic']\nsubscribe=['allowed/topic']\n")
        private_file(password, "fixture-pass\n")
        private_file(wrong_password, "wrong-pass\n")
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        env = {key: value for key, value in os.environ.items()
               if not key.startswith("MQTT_")}
        env.update(MQTT_MODE="acl-lab", MQTT_BIND=f"127.0.0.1:{port}",
                   MQTT_STATE_DIR=str(state), MQTT_USERS_FILE=str(users),
                   MQTT_ACL_FILE=str(acl), RUST_LOG="error")
        process = subprocess.Popen([str(broker)], env=env,
                                   stdout=subprocess.DEVNULL,
                                   stderr=subprocess.PIPE)
        try:
            until = time.monotonic() + 5
            while time.monotonic() < until:
                if process.poll() is not None:
                    raise AssertionError("broker exited before readiness")
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=.1):
                        break
                except OSError:
                    time.sleep(.02)
            else:
                raise AssertionError("broker readiness timeout")
            denied = run_client(client, port, password, "pub", "denied/topic")
            assert denied.returncode != 0, "denied publish succeeded"
            assert "NotAuthorized (0x87)" in denied.stderr, denied.stderr
            assert not denied.stdout, "denied publish emitted success output"
            allowed = run_client(client, port, password, "pub", "allowed/topic")
            assert allowed.returncode == 0, allowed.stderr
            refused_connect = run_client(client, port, wrong_password,
                                         "pub", "allowed/topic")
            assert refused_connect.returncode != 0, "bad password connected"
            assert "BadUserNamePassword (0x86)" in refused_connect.stderr, refused_connect.stderr
            assert not refused_connect.stdout, "bad password emitted success output"
            sub = run_client(client, port, password, "sub", "denied/topic")
            assert sub.returncode != 0, "denied subscribe succeeded"
            assert "SUBACK MQTT 5 recusou filtro 1" in sub.stderr, sub.stderr
            assert "NotAuthorized (0x87)" in sub.stderr, sub.stderr
            print("PASS: denied PUBACK/SUBACK 0x87, CONNACK 0x86 and allowed publish")
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            process.stderr.close()


if __name__ == "__main__":
    main()
