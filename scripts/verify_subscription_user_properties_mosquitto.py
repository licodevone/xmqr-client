#!/usr/bin/env python3
"""C102: independent Mosquitto MQTT 5 interop; supply an ephemeral listener port."""
import argparse
import json
from pathlib import Path
import select
import subprocess
import time

from verify_subscribe_options_mqtt5 import connect


CLIENT = Path(__file__).resolve().parents[1] / "target/release/mqtt-client"
TOPIC = b"c102/external"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, required=True)
    args = parser.parse_args()
    assert CLIENT.is_file(), "build Linux client release first"
    subscriber = subprocess.Popen(
        [CLIENT, "sub", "--open-lab", "--protocol-version", "5",
         "--port", str(args.port), "--topic", TOPIC.decode(), "--count", "1",
         "--output", "jsonl", "--subscription-identifier", "7",
         "--subscribe-user-property", "trace=one",
         "--subscribe-user-property", "trace=two",
         "--unsubscribe-user-property", "done=yes"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    try:
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            ready, _, _ = select.select([subscriber.stderr], [], [], deadline - time.monotonic())
            assert ready, f"Mosquitto SUBACK timeout (exit={subscriber.poll()})"
            if b"Assinatura ativa" in subscriber.stderr.readline():
                break
        else:
            raise AssertionError("Mosquitto did not acknowledge subscription")
        with connect(args.port, b"c102-external-pub") as stream:
            body = len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00" + b"external"
            stream.sendall(bytes([0x30, len(body)]) + body)
        output, errors = subscriber.communicate(timeout=8)
        assert subscriber.returncode == 0, errors
        event = json.loads(output)
        assert event["topic"] == TOPIC.decode(), event
        assert bytes(event["payload_bytes"]) == b"external", event
        assert event["metadata"]["subscription_identifiers"] == [7], event
    finally:
        if subscriber.poll() is None:
            subscriber.kill()
            subscriber.communicate()
    print("C102_MOSQUITTO_PASS repeat, identifier, publish, unsubscribe")


if __name__ == "__main__":
    main()
