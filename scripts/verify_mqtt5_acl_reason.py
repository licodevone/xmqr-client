#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""P119-P122/C60-C63: isolated broker/client MQTT 5 reason-code contract."""

import argparse
import json
import os
from pathlib import Path
import re
import select
import shlex
import shutil
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


def client_command(client: Path, port: int, password: Path, mode: str,
                   topic: str, qos: str = "1"):
    command = [str(client), mode, "--plain-auth-lab", "--username", "fixture",
               "--password-file", str(password), "--protocol-version", "5",
               "--host", "127.0.0.1", "--port", str(port), "--client-id",
               f"fx-{mode}-{topic.split('/')[0]}", "--topic", topic,
               "--qos", qos, "--timeout-seconds", "3"]
    if mode == "pub":
        command += ["--message", "fixture-message"]
    else:
        command += ["--count", "1", "--output", "jsonl"]
    return command


def run_client(client: Path, port: int, password: Path, mode: str,
               topic: str, qos: str = "1"):
    return subprocess.run(client_command(client, port, password, mode, topic, qos),
                          capture_output=True, text=True, timeout=8)


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
            denied_qos2 = run_client(client, port, password, "pub", "denied/topic", "2")
            assert denied_qos2.returncode != 0, "denied QoS2 publish succeeded"
            assert "PUBREC: NotAuthorized (0x87)" in denied_qos2.stderr, denied_qos2.stderr
            assert not denied_qos2.stdout, "denied QoS2 emitted success output"
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
            for qos in ["1", "2"]:
                subscriber = subprocess.Popen(
                    client_command(client, port, password, "sub", "allowed/topic", qos),
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                try:
                    ready, _, _ = select.select([subscriber.stderr], [], [], 4)
                    assert ready, "subscriber did not report active subscription"
                    status = subscriber.stderr.readline()
                    assert "Assinatura ativa" in status, status
                    delivered = run_client(client, port, password,
                                           "pub", "allowed/topic", qos)
                    assert delivered.returncode == 0, delivered.stderr
                    output, errors = subscriber.communicate(timeout=8)
                    assert subscriber.returncode == 0, errors
                    events = [json.loads(line) for line in output.splitlines()]
                    assert any(event.get("topic") == "allowed/topic" and
                               event.get("payload_bytes") == list(b"fixture-message") and
                               event.get("qos") == int(qos)
                               for event in events), output
                finally:
                    if subscriber.poll() is None:
                        subscriber.kill()
                        subscriber.wait(timeout=5)
            for program in ("mosquitto_pub", "mosquitto_sub"):
                assert shutil.which(program), f"{program} unavailable for external interop"
            mosquitto_args = ["-V", "mqttv5", "-h", "127.0.0.1", "-p", str(port),
                              "-u", "fixture", "-P", "fixture-pass", "-q", "2"]
            external_sub = subprocess.Popen(
                ["mosquitto_sub", *mosquitto_args, "-t", "allowed/topic",
                 "-i", "mosq-sub-qos2", "-C", "1", "-W", "5"],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                time.sleep(.35)
                assert external_sub.poll() is None, "Mosquitto subscriber exited early"
                published = run_client(client, port, password, "pub", "allowed/topic", "2")
                assert published.returncode == 0, published.stderr
                output, errors = external_sub.communicate(timeout=8)
                assert external_sub.returncode == 0, errors
                assert output.strip() == "fixture-message", output
            finally:
                if external_sub.poll() is None:
                    external_sub.kill()
                    external_sub.wait(timeout=5)
            subscriber = subprocess.Popen(
                client_command(client, port, password, "sub", "allowed/topic", "2"),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                ready, _, _ = select.select([subscriber.stderr], [], [], 4)
                assert ready, "XMQR subscriber did not become ready"
                assert "Assinatura ativa" in subscriber.stderr.readline()
                external_pub = subprocess.run(
                    ["mosquitto_pub", *mosquitto_args, "-t", "allowed/topic",
                     "-i", "mosq-pub-qos2", "-m", "external-qos2"],
                    capture_output=True, text=True, timeout=8)
                assert external_pub.returncode == 0, external_pub.stderr
                output, errors = subscriber.communicate(timeout=8)
                assert subscriber.returncode == 0, errors
                events = [json.loads(line) for line in output.splitlines()]
                assert any(event.get("payload_bytes") == list(b"external-qos2") and
                           event.get("qos") == 2 for event in events), output
            finally:
                if subscriber.poll() is None:
                    subscriber.kill()
                    subscriber.wait(timeout=5)
            print("PASS: denied PUBACK/PUBREC/SUBACK, CONNACK, QoS1/2 delivery and UNSUBACK")
            print("PASS: Mosquitto 2.0.22 MQTT5 QoS2 in both directions")
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
