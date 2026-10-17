#!/usr/bin/env python3
"""C102: inspect real MQTT 5 SUBSCRIBE/UNSUBSCRIBE wire properties."""
import socket
import subprocess
from pathlib import Path


CLIENT = Path(__file__).resolve().parents[1] / "target/debug/mqtt-client"
TOPIC = b"c102/properties"
PAIR_ONE = b"\x26\x00\x05trace\x00\x03one"
PAIR_TWO = b"\x26\x00\x05trace\x00\x03two"
PAIR_UNSUB = b"\x26\x00\x04done\x00\x03yes"


def exact(stream, count):
    data = bytearray()
    while len(data) < count:
        chunk = stream.recv(count - len(data))
        if not chunk:
            raise AssertionError("broker socket closed early")
        data.extend(chunk)
    return bytes(data)


def packet(stream):
    header = exact(stream, 1)[0]
    size = 0
    factor = 1
    for _ in range(4):
        digit = exact(stream, 1)[0]
        size += (digit & 0x7f) * factor
        assert size <= 65536
        if digit & 0x80 == 0:
            return header, exact(stream, size)
        factor *= 128
    raise AssertionError("invalid remaining length")


def body_properties(body):
    assert len(body) >= 3
    size = body[2]
    assert size < 128
    return body[:2], body[3:3 + size], body[3 + size:]


def main():
    assert CLIENT.is_file(), "build the Linux client first"
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(8)
        port = listener.getsockname()[1]
        command = [str(CLIENT), "sub", "--open-lab", "--protocol-version", "5",
                   "--host", "127.0.0.1", "--port", str(port), "--topic", TOPIC.decode(),
                   "--count", "1", "--client-id", "c102-wire", "--timeout-seconds", "4",
                   "--subscription-identifier", "3",
                   "--subscribe-user-property", "trace=one",
                   "--subscribe-user-property", "trace=two",
                   "--unsubscribe-user-property", "done=yes"]
        child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with listener.accept()[0] as stream:
                stream.settimeout(5)
                header, connect = packet(stream)
                assert header == 0x10 and connect.startswith(b"\x00\x04MQTT\x05")
                stream.sendall(b"\x20\x03\x00\x00\x00")

                header, subscribe = packet(stream)
                assert header == 0x82
                subscribe_id, properties, filters = body_properties(subscribe)
                assert properties == b"\x0b\x03" + PAIR_ONE + PAIR_TWO, properties
                assert filters == len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00"
                stream.sendall(b"\x90\x04" + subscribe_id + b"\x00\x00")

                publication = len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00payload"
                stream.sendall(bytes([0x30, len(publication)]) + publication)
                header, unsubscribe = packet(stream)
                assert header == 0xa2
                unsubscribe_id, properties, filters = body_properties(unsubscribe)
                assert properties == PAIR_UNSUB, properties
                assert filters == len(TOPIC).to_bytes(2, "big") + TOPIC
                stream.sendall(b"\xb0\x04" + unsubscribe_id + b"\x00\x00")
                assert packet(stream) == (0xe0, b"")
            output, errors = child.communicate(timeout=8)
            assert child.returncode == 0, errors
            assert b"payload" in output, output
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
    print("C102_SUBSCRIPTION_USER_PROPERTIES_PASS ordered wire, identifier, unsubscribe, count")


if __name__ == "__main__":
    main()
