#!/usr/bin/env python3
"""C89: disposable Mosquitto MQTT 5 shared filter and normal subscriber interop."""
import argparse
import json
from pathlib import Path
import select
import subprocess
import time

from verify_subscribe_options_mqtt5 import connect


CLIENT = Path(__file__).resolve().parents[1] / "target/release/mqtt-client"
TOPIC = b"c89/value"


def publish(stream, payload, retain=False):
    body = len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00" + payload
    assert len(body) < 128
    stream.sendall(bytes([0x31 if retain else 0x30, len(body)]) + body)


def subscriber(port, identity, topic, count):
    command = [str(CLIENT), "sub", "--open-lab", "--protocol-version", "5",
               "--port", str(port), "--client-id", identity, "--topic", topic,
               "--count", str(count), "--output", "jsonl"]
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        ready, _, _ = select.select([child.stderr], [], [], deadline - time.monotonic())
        assert ready, f"SUBACK timeout: {identity}"
        line = child.stderr.readline()
        assert line, f"client exited before SUBACK: {identity}"
        if b"Assinatura ativa" in line:
            return child
    raise AssertionError(f"SUBACK missing: {identity}")


def messages(child):
    output, errors = child.communicate(timeout=12)
    assert child.returncode == 0, errors
    return [json.loads(line) for line in output.splitlines()]


def payloads(items):
    for item in items:
        assert item["topic"] == TOPIC.decode(), item
    return [bytes(item["payload_bytes"]) for item in items]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, required=True)
    args = parser.parse_args()
    assert CLIENT.is_file(), "build Linux client release first"
    children = []
    with connect(args.port, b"c89-publisher") as publisher:
        publish(publisher, b"retained-seed", retain=True)
        try:
            shared = "$share/workers/" + TOPIC.decode()
            first = subscriber(args.port, "c89-worker-a", shared, 5)
            children.append(first)
            second = subscriber(args.port, "c89-worker-b", shared, 5)
            children.append(second)
            normal = subscriber(args.port, "c89-observer", TOPIC.decode(), 11)
            children.append(normal)
            assert not select.select([first.stdout, second.stdout], [], [], 0.3)[0], (
                "shared subscriber received initial retained publication"
            )
            expected = [f"message-{index}".encode() for index in range(10)]
            for payload in expected:
                publish(publisher, payload)
            first_payloads = payloads(messages(first))
            second_payloads = payloads(messages(second))
            normal_payloads = payloads(messages(normal))
            assert len(first_payloads) == len(second_payloads) == 5
            assert sorted(first_payloads + second_payloads) == sorted(expected)
            assert normal_payloads[0] == b"retained-seed"
            assert normal_payloads[1:] == expected
        finally:
            for child in children:
                if child.poll() is None:
                    child.kill()
                    child.communicate()
    print("C89_MOSQUITTO_PASS one delivery per group, normal copy, no initial retained")


if __name__ == "__main__":
    main()
