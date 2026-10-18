#!/usr/bin/env python3
"""C105: durable QoS 1 outbox retains uncertainty and resends only with Session Present."""
import os
import socket
import subprocess
import tempfile

from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet


def command(port, directory, message="first"):
    return [str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
            "--host", "127.0.0.1", "--port", str(port), "--topic", "c105/outbox",
            "--message", message, "--qos", "1", "--clean-start", "false",
            "--session-expiry-seconds", "60", "--client-id", "c105-device",
            "--outbox-dir", directory, "--timeout-seconds", "2"]


def listener(port=0):
    server = socket.socket()
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", port))
    server.listen(1)
    server.settimeout(5)
    return server


def publish_fields(header, body):
    assert header & 0xf0 == 0x30 and header & 0x06 == 0x02
    size = int.from_bytes(body[:2], "big")
    topic = body[2:2 + size]
    packet_id = int.from_bytes(body[2 + size:4 + size], "big")
    return topic, packet_id, body[4 + size:]


def interrupted(directory, port):
    with listener(port) as server:
        child = subprocess.Popen(command(server.getsockname()[1], directory),
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x00\x00\x00")
                header, body = packet(stream)
                assert header == 0x32
                topic, packet_id, tail = publish_fields(header, body)
                assert topic == b"c105/outbox" and packet_id == 1 and tail == b"\x00first"
            _, stderr = child.communicate(timeout=5)
            assert child.returncode != 0 and b"outbox" in stderr.lower()
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
    assert os.path.isfile(os.path.join(directory, "pending.json"))


def resumed(directory, port, session_present):
    with listener(port) as server:
        child = subprocess.Popen(command(server.getsockname()[1], directory),
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03" + bytes([int(session_present)]) + b"\x00\x00")
                if session_present:
                    header, body = packet(stream)
                    assert header == 0x3a, hex(header)
                    topic, packet_id, tail = publish_fields(header, body)
                    assert topic == b"c105/outbox" and packet_id == 1 and tail == b"\x00first"
                    stream.sendall(b"\x40\x02\x00\x01")
                    assert packet(stream) == (0xe0, b"")
                else:
                    assert stream.recv(1) == b"", "must not replay without Session Present"
            _, stderr = child.communicate(timeout=5)
            assert (child.returncode == 0) is session_present, stderr
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
    assert os.path.isfile(os.path.join(directory, "pending.json")) is not session_present


def negative_puback(directory):
    with listener() as server:
        child = subprocess.Popen(command(server.getsockname()[1], directory),
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x00\x00\x00")
                assert packet(stream)[0] == 0x32
                stream.sendall(b"\x40\x04\x00\x01\x87\x00")
                _, stderr = child.communicate(timeout=5)
                assert child.returncode != 0 and stderr
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
    assert not os.path.exists(os.path.join(directory, "pending.json")), stderr


def no_disk_commit_no_network(directory):
    os.chmod(directory, 0o755)
    with listener() as server:
        result = subprocess.run(command(server.getsockname()[1], directory),
                                capture_output=True, timeout=4)
        assert result.returncode != 0 and b"outbox" in result.stderr.lower()
        server.settimeout(0.3)
        try:
            server.accept()
        except socket.timeout:
            pass
        else:
            raise AssertionError("connected before committing the outbox")


def main():
    assert CLIENT.is_file(), "build the Linux client first"
    with tempfile.TemporaryDirectory(prefix="xmqr-c105-outbox-") as directory:
        with listener() as server:
            port = server.getsockname()[1]
        interrupted(directory, port)
        mismatch = subprocess.run(command(9, directory, "different"), capture_output=True,
                                  timeout=4)
        assert mismatch.returncode != 0 and b"outbox" in mismatch.stderr.lower()
        resumed(directory, port, False)
        resumed(directory, port, True)
    with tempfile.TemporaryDirectory(prefix="xmqr-c105-refusal-") as directory:
        negative_puback(directory)
    with tempfile.TemporaryDirectory(prefix="xmqr-c105-disk-") as directory:
        no_disk_commit_no_network(directory)
    print("C105_QOS1_OUTBOX_WIRE_PASS uncertain, mismatch, no-session, DUP, refusal, disk gate")


if __name__ == "__main__":
    main()
