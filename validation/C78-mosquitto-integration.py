#!/usr/bin/env python3
"""C78: inspect request properties forwarded by a disposable Mosquitto 2 broker."""
import os
from pathlib import Path
import socket
import subprocess

CLIENT = Path(__file__).resolve().parents[1] / "target/debug/mqtt-client"
PORT = int(os.environ.get("C78_MOSQUITTO_PORT", "18884"))


def packet(stream):
    header = stream.recv(1)
    assert header, "connection closed"
    length = 0
    multiplier = 1
    while True:
        digit = stream.recv(1)
        assert digit, "truncated packet length"
        length += (digit[0] & 127) * multiplier
        if digit[0] & 128 == 0:
            break
        multiplier *= 128
    body = bytearray()
    while len(body) < length:
        chunk = stream.recv(length - len(body))
        assert chunk, "truncated packet body"
        body.extend(chunk)
    return header[0], bytes(body)


def string(body, offset):
    size = int.from_bytes(body[offset:offset + 2], "big")
    offset += 2
    return body[offset:offset + size], offset + size


def main():
    assert CLIENT.is_file(), "build Linux client first"
    with socket.create_connection(("127.0.0.1", PORT), timeout=5) as stream:
        stream.settimeout(5)
        identity = b"c78-external-sub"
        connect = (b"\x00\x04MQTT\x05\x02\x00\x1e\x00"
                   + len(identity).to_bytes(2, "big") + identity)
        stream.sendall(bytes([0x10, len(connect)]) + connect)
        connack_header, connack_body = packet(stream)
        assert connack_header == 0x20 and connack_body[:2] == b"\x00\x00", (connack_header, connack_body)
        topic = b"request/mosq"
        subscribe = b"\x00\x01\x00" + len(topic).to_bytes(2, "big") + topic + b"\x01"
        stream.sendall(bytes([0x82, len(subscribe)]) + subscribe)
        assert packet(stream) == (0x90, b"\x00\x01\x00\x01")

        result = subprocess.run([
            str(CLIENT), "pub", "--open-lab", "--port", str(PORT),
            "--protocol-version", "5", "--topic", topic.decode(),
            "--message", "external", "--qos", "1",
            "--response-topic", "reply/t", "--correlation-data-hex", "00ff07",
            "--publish-user-property", "trace=one",
            "--publish-user-property", "trace=two",
        ], capture_output=True, timeout=10)
        assert result.returncode == 0, result.stderr
        header, body = packet(stream)
        assert header >> 4 == 3 and (header >> 1) & 3 == 1, (header, body)
        received_topic, offset = string(body, 0)
        assert received_topic == topic
        packet_id = body[offset:offset + 2]
        offset += 2
        properties_length = body[offset]
        offset += 1
        end = offset + properties_length
        response = correlation = None
        users = []
        while offset < end:
            kind = body[offset]
            offset += 1
            if kind == 0x08:
                response, offset = string(body, offset)
            elif kind == 0x09:
                correlation, offset = string(body, offset)
            elif kind == 0x26:
                key, offset = string(body, offset)
                value, offset = string(body, offset)
                users.append((key, value))
            else:
                raise AssertionError(f"unexpected PUBLISH property {kind:#x}")
        assert response == b"reply/t" and correlation == b"\x00\xff\x07"
        assert users == [(b"trace", b"one"), (b"trace", b"two")]
        assert body[end:] == b"external"
        stream.sendall(b"\x40\x02" + packet_id)
        stream.sendall(b"\xe0\x00")
    print("PASS: C78 -> Mosquitto 2 MQTT 5 Response Topic, Correlation Data and ordered User Properties")


if __name__ == "__main__":
    main()
