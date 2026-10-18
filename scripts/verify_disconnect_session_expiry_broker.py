#!/usr/bin/env python3
"""C104: confirm DISCONNECT Session Expiry against a disposable XMQR broker."""
import argparse
import os
import socket
import subprocess
import tempfile
import time
from pathlib import Path

from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet


def start(broker, port, state):
    env = dict(os.environ, MQTT_MODE="open-lab", MQTT_STATE_DIR=state,
               MQTT_BIND=f"127.0.0.1:{port}")
    child = subprocess.Popen([broker], env=env, stdout=subprocess.DEVNULL,
                             stderr=subprocess.DEVNULL)
    for _ in range(100):
        if child.poll() is not None:
            raise AssertionError("broker stopped before listening")
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                return child
        except OSError:
            time.sleep(0.03)
    raise AssertionError("broker did not listen")


def resume(port, identity):
    with socket.create_connection(("127.0.0.1", port), timeout=4) as stream:
        stream.settimeout(4)
        body = (b"\x00\x04MQTT\x05\x00\x00\x1e\x05\x11\x00\x00\x00\x00"
                + len(identity).to_bytes(2, "big") + identity)
        stream.sendall(bytes([0x10, len(body)]) + body)
        header, ack = packet(stream)
        assert header == 0x20 and ack[1] == 0, (header, ack)
        stream.sendall(b"\xe0\x00")
        return bool(ack[0] & 1)


def publish(port, identity, expiry):
    command = [str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
               "--host", "127.0.0.1", "--port", str(port), "--topic", "c104/live",
               "--message", "value", "--client-id", identity.decode(),
               "--session-expiry-seconds", "60", "--disconnect-session-expiry-seconds",
               expiry, "--timeout-seconds", "4"]
    result = subprocess.run(command, capture_output=True, timeout=8)
    assert result.returncode == 0, result.stderr


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker", type=Path, required=True)
    args = parser.parse_args()
    assert args.broker.is_file() and CLIENT.is_file(), "build Linux broker and client"
    with tempfile.TemporaryDirectory(prefix="xmqr-c104-broker-") as state:
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        broker = start(args.broker, port, state)
        try:
            for identity, expiry, expected in [(b"c104-zero", "0", False),
                                               (b"c104-keep", "30", True)]:
                publish(port, identity, expiry)
                assert resume(port, identity) is expected, (identity, expiry)
        finally:
            broker.terminate()
            try:
                broker.wait(timeout=5)
            except subprocess.TimeoutExpired:
                broker.kill()
                broker.wait(timeout=5)
    print("C104_DISCONNECT_SESSION_EXPIRY_BROKER_PASS zero removes, finite preserves")


if __name__ == "__main__":
    main()
