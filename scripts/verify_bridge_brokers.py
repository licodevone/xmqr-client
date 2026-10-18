#!/usr/bin/env python3
"""C125: disposable one-way TLS bridge against two real XMQR brokers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import select
import signal
import ssl
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
CLIENT_ROOT = Path(__file__).resolve().parents[1]
CLIENT = CLIENT_ROOT / "target/debug/mqtt-client"
BRIDGE = CLIENT_ROOT / "target/debug/mqtt-bridge"


def private(path, contents):
    path.write_text(contents)
    path.chmod(0o600)


def wait_markers(child, markers, seconds=12):
    deadline = time.monotonic() + seconds
    seen = bytearray()
    while time.monotonic() < deadline:
        assert child.poll() is None, ("bridge exited", seen)
        ready, _, _ = select.select([child.stderr], [], [], .2)
        if ready:
            seen.extend(os.read(child.stderr.fileno(), 4096))
            assert len(seen) < 16384, "excess bridge diagnostics"
            if all(marker.encode() in seen for marker in markers):
                return
    raise AssertionError(("bridge status timeout", markers, seen))


def command(root, port, mode, topic, payload=None):
    common = [str(CLIENT), mode, "--port", str(port), "--topic", topic,
              "--username", "device", "--password-file", str(root / "password"),
              "--ca", str(root / "ca.crt"), "--cert", str(root / "client.crt"),
              "--key", str(root / "client.key"), "--client-id", f"c125-{mode}",
              "--timeout-seconds", "5"]
    if mode == "pub":
        common += ["--message", payload, "--qos", "0"]
    else:
        common += ["--count", "1"]
    return common


def exchange(root, source_port, target_port, suffix, payload, *, expected=True):
    subscriber = subprocess.Popen(command(root, target_port, "sub", f"mirror/{suffix}"),
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        ready, _, _ = select.select([subscriber.stdout], [], [], 8)
        assert ready, "subscriber did not report SUBACK"
        assert "Assinatura ativa" in subscriber.stdout.readline()
        published = subprocess.run(command(root, source_port, "pub", f"sensors/{suffix}",
                                           payload), capture_output=True, timeout=10)
        assert published.returncode == 0, published.stderr.decode(errors="replace")
        if expected:
            stdout, stderr = subscriber.communicate(timeout=8)
            assert subscriber.returncode == 0 and payload in stdout, (stdout, stderr)
        else:
            try:
                stdout, stderr = subscriber.communicate(timeout=4)
                assert payload not in stdout, (stdout, stderr)
            except subprocess.TimeoutExpired:
                subscriber.kill()
                stdout, _ = subscriber.communicate(timeout=5)
                assert payload not in stdout
    finally:
        if subscriber.poll() is None:
            subscriber.kill()
            subscriber.communicate(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--broker-root", type=Path, required=True)
    args = parser.parse_args()
    broker_root = args.broker_root.resolve()
    sys.path.insert(0, str(broker_root / "scripts"))
    from verify_multi_endpoint import free_port, hash_fixture, wait_port
    from verify_wss_mtls import certificates

    broker = broker_root / "target/debug/mqtt-broker"
    admin = broker_root / "target/debug/mqtt-admin"
    assert all(path.is_file() for path in (broker, admin, CLIENT, BRIDGE))
    with tempfile.TemporaryDirectory(prefix="xmqr-c125-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        certificates(root)
        secret = secrets.token_urlsafe(18)
        private(root / "password", secret + "\n")
        fingerprint = hashlib.sha256(
            ssl.PEM_cert_to_DER_cert((root / "client.crt").read_text())).hexdigest()
        digest = hash_fixture(admin, secret)
        private(root / "users.toml",
                f'[[users]]\nusername="device"\npassword_hash="{digest}"\n'
                f'cert_sha256="sha256:{fingerprint}"\n')
        private(root / "source-acl.toml",
                '[[rules]]\nusername="device"\n'
                'publish=["sensors/allowed","sensors/denied"]\n'
                'subscribe=["sensors/#"]\n')
        private(root / "target-acl.toml",
                '[[rules]]\nusername="device"\npublish=["mirror/allowed"]\n'
                'subscribe=["mirror/allowed","mirror/denied"]\n')
        source_port, target_port = free_port(), free_port()
        assert source_port != target_port
        base = {key: value for key, value in os.environ.items() if not key.startswith("MQTT_")}
        base.update(MQTT_MODE="secure-mtls", MQTT_SERVER_CERT=str(root / "server.crt"),
                    MQTT_SERVER_KEY=str(root / "server.key"),
                    MQTT_CLIENT_CA=str(root / "ca.crt"),
                    MQTT_USERS_FILE=str(root / "users.toml"))
        source_env = dict(base, MQTT_BIND=f"127.0.0.1:{source_port}",
                          MQTT_STATE_DIR=str(root / "source-state"),
                          MQTT_ACL_FILE=str(root / "source-acl.toml"))
        target_env = dict(base, MQTT_BIND=f"127.0.0.1:{target_port}",
                          MQTT_STATE_DIR=str(root / "target-state"),
                          MQTT_ACL_FILE=str(root / "target-acl.toml"))
        def start(environment, port):
            log_path = root / f"broker-{port}.log"
            with log_path.open("ab") as log:
                child = subprocess.Popen([str(broker)], env=environment,
                                         stdout=log, stderr=log)
            try:
                wait_port(child, port)
            except AssertionError as error:
                raise AssertionError(log_path.read_text(errors="replace")[-1200:]) from error
            return child
        def stop(child):
            if child.poll() is None:
                child.send_signal(signal.SIGTERM)
                assert child.wait(timeout=8) == 0
        def endpoint(port, client_id, ca="ca.crt"):
            return (f'host="127.0.0.1"\nport={port}\nclient_id="{client_id}"\n'
                    'username="device"\n'
                    f'password_file={json.dumps(str(root / "password"))}\n'
                    f'ca={json.dumps(str(root / ca))}\n'
                    f'cert={json.dumps(str(root / "client.crt"))}\n'
                    f'key={json.dumps(str(root / "client.key"))}\n')
        def configuration(path, *, destination_port=target_port, dest_ca="ca.crt",
                          retries=8):
            private(path, f'version=1\nmax_retries={retries}\n[source]\n'
                    + endpoint(source_port, "c125-bridge-source")
                    + '[destination]\n'
                    + endpoint(destination_port, "c125-bridge-target", dest_ca)
                    + '[route]\nsource_prefix="sensors/"\ntarget_prefix="mirror/"\n')
        config = root / "bridge.toml"
        configuration(config)
        same = root / "same.toml"
        configuration(same, destination_port=source_port)
        rejected = subprocess.run([str(BRIDGE), "--config", str(same)],
                                  capture_output=True, timeout=5)
        assert rejected.returncode == 2 and b"invalida" in rejected.stderr.lower()
        assert subprocess.run([str(BRIDGE), "--version"], capture_output=True,
                              timeout=5).stdout.startswith(b"mqtt-bridge 0.1.0-dev")
        source = start(source_env, source_port)
        target = start(target_env, target_port)
        bridge_process = None
        try:
            wrong = root / "wrong-ca.toml"
            configuration(wrong, dest_ca="other.crt", retries=1)
            negative = subprocess.run([str(BRIDGE), "--config", str(wrong)],
                                      capture_output=True, timeout=15)
            assert negative.returncode != 0
            assert secret.encode() not in negative.stderr
            bridge_process = subprocess.Popen([str(BRIDGE), "--config", str(config)],
                                              stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                              bufsize=0)
            wait_markers(bridge_process, ("destino conectado", "origem assinada"))
            exchange(root, source_port, target_port, "allowed", "c125-first")
            exchange(root, source_port, target_port, "denied", "c125-denied",
                     expected=False)
            stop(target)
            wait_markers(bridge_process, ("destino desconectado",), seconds=10)
            target = start(target_env, target_port)
            wait_markers(bridge_process, ("destino conectado",), seconds=20)
            exchange(root, source_port, target_port, "allowed", "c125-after-restart")
        except BaseException:
            if bridge_process is not None and bridge_process.poll() is None:
                ready, _, _ = select.select([bridge_process.stderr], [], [], .1)
                if ready:
                    print(os.read(bridge_process.stderr.fileno(), 4096).decode(errors="replace"),
                          file=sys.stderr)
            print((root / f"broker-{target_port}.log").read_text(errors="replace")[-1800:],
                  file=sys.stderr)
            raise
        finally:
            if bridge_process is not None:
                if bridge_process.poll() is None:
                    bridge_process.terminate()
                bridge_process.communicate(timeout=8)
            stop(source)
            stop(target)
    print("C125_BRIDGE_XMQR_PASS mapping, ACL, TLS-CA, reconnect, same-endpoint rejection")


if __name__ == "__main__":
    main()
