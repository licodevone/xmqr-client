#!/usr/bin/env python3
"""C87: Subscription Identifier client output with disposable external MQTT 5 broker."""
import argparse
import json
from pathlib import Path
import select
import subprocess
import time

from verify_subscribe_options_mqtt5 import connect


CLIENT = Path(__file__).resolve().parents[1] / "target/release/mqtt-client"
TOPIC = b"c87/value"


def publish(port, payload):
    with connect(port, b"c87-publisher") as stream:
        body = len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00" + payload
        stream.sendall(bytes([0x30, len(body)]) + body)


def exercise(port, filters, expected_ids):
    command = [str(CLIENT), "sub", "--open-lab", "--protocol-version", "5",
               "--port", str(port), "--client-id", f"c87-{port}-{len(filters)}",
               "--subscription-identifier", "17", "--count", str(len(expected_ids)), "--output", "jsonl"]
    for topic in filters:
        command.extend(["--topic", topic])
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            ready, _, _ = select.select([child.stderr], [], [], deadline - time.monotonic())
            assert ready, "client SUBACK timeout"
            if b"Assinatura ativa" in child.stderr.readline():
                break
        else:
            raise AssertionError("client did not acknowledge subscription")
        publish(port, b"id-17")
        output, errors = child.communicate(timeout=8)
        assert child.returncode == 0, errors
        messages = [json.loads(line) for line in output.splitlines()]
        assert len(messages) == len(expected_ids), messages
        for message, identifiers in zip(messages, expected_ids, strict=True):
            assert message["topic"] == TOPIC.decode() and bytes(message["payload_bytes"]) == b"id-17", message
            assert message["metadata"]["subscription_identifiers"] == identifiers, message
    finally:
        if child.poll() is None:
            child.kill()
            child.communicate()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, required=True)
    args = parser.parse_args()
    assert CLIENT.is_file(), "build Linux client release first"
    exercise(args.port, [TOPIC.decode()], [[17]])
    exercise(args.port, [TOPIC.decode(), "c87/#"], [[17], [17]])
    print("C87_MOSQUITTO_PASS Subscription Identifier and overlapping filters")


if __name__ == "__main__":
    main()
