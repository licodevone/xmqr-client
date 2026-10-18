#!/usr/bin/env python3
"""C104: observe MQTT 5 DISCONNECT expiry on an independent TCP fixture."""
import socket
import subprocess
import os
from pathlib import Path


CLIENT = Path(__file__).resolve().parents[1] / "target/debug" / (
    "mqtt-client.exe" if os.name == "nt" else "mqtt-client"
)


def exact(stream, count):
    data = bytearray()
    while len(data) < count:
        chunk = stream.recv(count - len(data))
        if not chunk:
            raise AssertionError("client closed before completing packet")
        data.extend(chunk)
    return bytes(data)


def packet(stream):
    header = exact(stream, 1)[0]
    remaining = 0
    factor = 1
    for _ in range(4):
        digit = exact(stream, 1)[0]
        remaining += (digit & 0x7f) * factor
        assert remaining <= 65536
        if digit & 0x80 == 0:
            return header, exact(stream, remaining)
        factor *= 128
    raise AssertionError("invalid Remaining Length")


def run_case(expiry, expected):
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(8)
        args = [str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
                "--host", "127.0.0.1", "--port", str(listener.getsockname()[1]),
                "--topic", "c104/wire", "--message", "hello", "--client-id", "c104-wire",
                "--session-expiry-seconds", "60", "--timeout-seconds", "4"]
        if expiry is not None:
            args += ["--disconnect-session-expiry-seconds", expiry]
        child = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with listener.accept()[0] as stream:
                stream.settimeout(5)
                header, body = packet(stream)
                assert header == 0x10 and body.startswith(b"\x00\x04MQTT\x05")
                stream.sendall(b"\x20\x03\x00\x00\x00")
                header, body = packet(stream)
                assert header == 0x30 and body.endswith(b"hello")
                assert packet(stream) == (0xe0, expected)
            stdout, stderr = child.communicate(timeout=8)
            assert child.returncode == 0, (stdout, stderr)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def main():
    assert CLIENT.is_file(), "build the Linux client first"
    run_case(None, b"")
    run_case("0", b"\x00\x05\x11\x00\x00\x00\x00")
    run_case("30", b"\x00\x05\x11\x00\x00\x00\x1e")
    run_case("never", b"\x00\x05\x11\xff\xff\xff\xff")
    print("C104_DISCONNECT_SESSION_EXPIRY_WIRE_PASS omitted, zero, finite, never")


if __name__ == "__main__":
    main()
