#!/usr/bin/env python3
"""C102/P180: real MQTT 5 subscriber against disposable XMQR broker."""
import os
import json
from pathlib import Path
import select
import socket
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]
BROKER = ROOT.parent / "xmqr/target/debug/mqtt-broker"
CLIENT = ROOT / "target/debug/mqtt-client"
TOPIC = "c102/integration"


def main():
    assert BROKER.is_file() and CLIENT.is_file(), "build Linux broker and client first"
    with tempfile.TemporaryDirectory(prefix="xmqr-c102-integration-") as state:
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        environment = dict(os.environ, MQTT_MODE="open-lab", MQTT_STATE_DIR=state,
                           MQTT_BIND=f"127.0.0.1:{port}")
        broker = subprocess.Popen([BROKER], env=environment, stdout=subprocess.DEVNULL,
                                  stderr=subprocess.DEVNULL)
        subscriber = None
        try:
            for _ in range(100):
                if broker.poll() is not None:
                    raise AssertionError("broker stopped before listening")
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                        break
                except OSError:
                    time.sleep(0.05)
            else:
                raise AssertionError("broker did not listen")
            subscriber = subprocess.Popen(
                [CLIENT, "sub", "--open-lab", "--protocol-version", "5",
                 "--port", str(port), "--topic", TOPIC, "--count", "1", "--output", "jsonl",
                 "--subscribe-user-property", "trace=one",
                 "--subscribe-user-property", "trace=two",
                 "--unsubscribe-user-property", "done=yes",
                 "--subscription-identifier", "3"],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
            deadline = time.monotonic() + 8
            while time.monotonic() < deadline:
                ready, _, _ = select.select([subscriber.stderr], [], [], deadline - time.monotonic())
                assert ready, f"client SUBACK timeout (exit={subscriber.poll()})"
                line = subscriber.stderr.readline()
                if b"Assinatura ativa" in line:
                    break
            else:
                raise AssertionError("client did not acknowledge subscription")
            publication = subprocess.run(
                [CLIENT, "pub", "--open-lab", "--port", str(port), "--topic", TOPIC,
                 "--message", "c102-real", "--qos", "1"],
                capture_output=True, timeout=8)
            assert publication.returncode == 0, publication.stderr
            output, errors = subscriber.communicate(timeout=8)
            assert subscriber.returncode == 0, errors
            event = json.loads(output)
            assert event["event"] == "publish", event
            assert event["topic"] == TOPIC, event
            assert bytes(event["payload_bytes"]) == b"c102-real", event
            assert event["metadata"]["subscription_identifiers"] == [3], event
        finally:
            if subscriber is not None and subscriber.poll() is None:
                subscriber.kill()
                subscriber.communicate()
            if broker.poll() is None:
                broker.terminate()
                broker.wait(timeout=8)
    print("C102_P180_INTEGRATION_PASS MQTT5 sub properties, MQTT311 pub, unsuback")


if __name__ == "__main__":
    main()
