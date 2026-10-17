#!/usr/bin/env python3
"""C85: MQTT 5 subscription options against an external disposable broker."""
import argparse
import json
from pathlib import Path
import select
import socket
import subprocess
import time


CLIENT = Path(__file__).resolve().parents[1] / "target/release/mqtt-client"
TOPIC = b"c85/options"


def exact(stream, count):
    data = bytearray()
    while len(data) < count:
        part = stream.recv(count - len(data))
        if not part:
            raise AssertionError("broker closed packet early")
        data.extend(part)
    return bytes(data)


def packet(stream):
    header = exact(stream, 1)[0]
    length = 0
    multiplier = 1
    for _ in range(4):
        part = exact(stream, 1)[0]
        length += (part & 0x7f) * multiplier
        if part < 0x80:
            assert length <= 4096
            return header, exact(stream, length)
        multiplier *= 128
    raise AssertionError("invalid MQTT length")


def connect(port, identity):
    stream = socket.create_connection(("127.0.0.1", port), timeout=5)
    stream.settimeout(5)
    body = b"\x00\x04MQTT\x05\x02\x00\x1e\x00" + len(identity).to_bytes(2, "big") + identity
    stream.sendall(bytes([0x10, len(body)]) + body)
    header, ack = packet(stream)
    assert header == 0x20 and ack[1] == 0, ack
    return stream


def publish(stream, payload, retain):
    body = len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00" + payload
    stream.sendall(bytes([0x31 if retain else 0x30, len(body)]) + body)


def raw_no_local(port):
    with connect(port, b"c85-raw-self") as same:
        body = b"\x00\x01\x00" + len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x0c"
        same.sendall(bytes([0x82, len(body)]) + body)
        header, ack = packet(same)
        assert header == 0x90 and ack[-1] == 0, ack
        publish(same, b"self", retain=True)
        assert not select.select([same], [], [], 0.3)[0], "No Local forwarded self publication"
        with connect(port, b"c85-raw-other") as other:
            publish(other, b"other", retain=True)
        header, body = packet(same)
        assert header == 0x31 and body.endswith(b"\x00other"), (header, body)


def client_retain(port, preserve):
    command = [str(CLIENT), "sub", "--open-lab", "--protocol-version", "5",
               "--port", str(port), "--topic", TOPIC.decode(), "--count", "1",
               "--output", "jsonl", "--retain-handling", "2",
               "--client-id", f"c85-client-{port}-{int(preserve)}"]
    if preserve:
        command.extend(["--no-local", "--retain-as-published"])
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
        payload = b"preserved" if preserve else b"cleared"
        with connect(port, b"c85-client-pub") as publisher:
            publish(publisher, payload, retain=True)
        output, errors = child.communicate(timeout=8)
        assert child.returncode == 0, errors
        message = json.loads(output.splitlines()[0])
        assert message["event"] == "publish" and message["topic"] == TOPIC.decode()
        assert message["retain"] is preserve and bytes(message["payload_bytes"]) == payload, message
    finally:
        if child.poll() is None:
            child.kill()
            child.communicate()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, required=True)
    args = parser.parse_args()
    assert CLIENT.is_file(), "build Linux client release first"
    raw_no_local(args.port)
    client_retain(args.port, preserve=False)
    client_retain(args.port, preserve=True)
    print("C85_MOSQUITTO_PASS raw No Local/RAP and client retained flag")


if __name__ == "__main__":
    main()
