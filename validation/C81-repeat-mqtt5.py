#!/usr/bin/env python3
"""C81: repeated MQTT 5 publication against XMQR and pinned Mosquitto."""
import os
import socket
import subprocess
import sys
import tempfile
from pathlib import Path

BROKER_ROOT = Path(__file__).resolve().parents[2] / "xmqr"
sys.path.insert(0, str(BROKER_ROOT / "scripts"))
from verify_mqtt5_sessions import BROKER, CLIENT, packet, start, stop  # noqa: E402

TOPIC = b"c81/repeat"


def free_port():
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def subscriber(port, identity):
    stream = socket.create_connection(("127.0.0.1", port), timeout=5)
    stream.settimeout(6)
    body = (b"\x00\x04MQTT\x05\x02\x00\x1e\x00"
            + len(identity).to_bytes(2, "big") + identity)
    stream.sendall(bytes([0x10, len(body)]) + body)
    header, ack = packet(stream)
    assert header == 0x20 and ack[1] == 0, ack
    subscribe = b"\x00\x01\x00" + len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x01"
    stream.sendall(bytes([0x82, len(subscribe)]) + subscribe)
    header, ack = packet(stream)
    assert header == 0x90 and ack[-1] == 1, ack
    return stream


def publication(stream, qos):
    header, body = packet(stream)
    assert header >> 4 == 3 and (header >> 1) & 3 == min(qos, 1), (header, body)
    size = int.from_bytes(body[:2], "big")
    assert body[2:2 + size] == TOPIC, body
    offset = 2 + size
    identifier = None
    if qos:
        identifier = body[offset:offset + 2]
        offset += 2
    property_size = body[offset]
    properties = body[offset + 1:offset + 1 + property_size]
    assert b"\x01\x01" in properties, properties
    assert b"\x03\x00\x0atext/plain" in properties, properties
    assert b"\x26\x00\x05trace\x00\x03one" in properties, properties
    assert body[offset + 1 + property_size:] == b"hello", body
    if identifier:
        stream.sendall(b"\x40\x02" + identifier)


def run_case(port, label):
    stream = subscriber(port, f"c81-{label}".encode())
    try:
        for qos in (0, 1, 2):
            command = [str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
                       "--port", str(port), "--topic", TOPIC.decode(), "--message", "hello",
                       "--qos", str(qos), "--repeat", "3", "--repeat-interval-ms", "20",
                       "--payload-format", "1", "--content-type", "text/plain",
                       "--publish-user-property", "trace=one", "--output", "jsonl"]
            publisher = subprocess.Popen(command, stdout=subprocess.PIPE,
                                         stderr=subprocess.PIPE)
            for _ in range(3):
                publication(stream, qos)
            output, errors = publisher.communicate(timeout=8)
            assert publisher.returncode == 0, errors
            assert output.count(b'"event":"publish_complete"') == 3, output
    finally:
        stream.close()


def main():
    assert BROKER.is_file() and CLIENT.is_file(), "build Linux broker/client first"
    with tempfile.TemporaryDirectory(prefix="xmqr-c81-repeat-") as state:
        port = free_port()
        broker = start(port, state)
        try:
            run_case(port, "xmqr")
        finally:
            stop(broker)
    external = os.environ.get("C81_MOSQUITTO_PORT")
    if external:
        run_case(int(external), "mosquitto")
    print("PASS: C81 repeated MQTT 5 QoS0/1/2 with ordered ACK and properties against XMQR"
          + (" and external Mosquitto" if external else ""))


if __name__ == "__main__":
    main()
