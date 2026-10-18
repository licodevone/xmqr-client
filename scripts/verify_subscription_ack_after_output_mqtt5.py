#!/usr/bin/env python3
"""C110: persistent subscriber acknowledges QoS only after successful output."""
import socket
import subprocess
import tempfile
import time

from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet
from verify_durable_qos1_outbox_mqtt5 import listener
from verify_durable_subscription_mqtt5 import command, stage


def args(port, directory, qos):
    result = command(port, directory) + ["--output", "jsonl"]
    result[result.index("--qos") + 1] = str(qos)
    return result


def publish(qos, identifier=7):
    topic = b"c108/data"
    body = len(topic).to_bytes(2, "big") + topic + identifier.to_bytes(2, "big") + b"\x00hello"
    return bytes([0x30 | (qos << 1), len(body)]) + body


def activate(port, directory, qos):
    with listener(port) as server:
        child = subprocess.Popen(args(port, directory, qos), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x00\x00\x00")
                header, body = packet(stream)
                assert header == 0x82 and body[-1] == qos
                stream.sendall(b"\x90\x04\x00\x01\x00" + bytes([qos]))
                for _ in range(100):
                    if stage(directory) == "Active":
                        break
                    time.sleep(0.01)
                assert stage(directory) == "Active"
                child.kill()
            child.communicate(timeout=5)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def success(port, directory, qos):
    with listener(port) as server:
        child = subprocess.Popen(args(port, directory, qos), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x01\x00\x00")
                stream.sendall(publish(qos))
                line = child.stdout.readline()
                assert b'"event":"publish"' in line and b"hello" not in line, line
                header, body = packet(stream)
                assert (header, body[:2]) == ((0x40 if qos == 1 else 0x50), b"\x00\x07")
                if qos == 2:
                    stream.sendall(b"\x62\x02\x00\x07")
                    assert packet(stream) == (0x70, b"\x00\x07")
                child.kill()
            child.communicate(timeout=5)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def failed_output(port, directory, qos):
    with listener(port) as server, open("/dev/full", "wb") as full:
        child = subprocess.Popen(args(port, directory, qos), stdout=full,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x01\x00\x00")
                stream.sendall(publish(qos))
                _, stderr = child.communicate(timeout=5)
                assert child.returncode != 0, stderr
                assert stream.recv(1) == b"", "ACK sent despite failed stdout"
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def filtered(port, directory, qos):
    with listener(port) as server:
        child = subprocess.Popen(args(port, directory, qos) + ["--exclude-topic", "c108/#"],
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x01\x00\x00")
                stream.sendall(publish(qos))
                header, body = packet(stream)
                assert (header, body[:2]) == ((0x40 if qos == 1 else 0x50), b"\x00\x07")
                if qos == 2:
                    stream.sendall(b"\x62\x02\x00\x07")
                    assert packet(stream) == (0x70, b"\x00\x07")
                child.kill()
            stdout, _ = child.communicate(timeout=5)
            assert b'"event":"publish"' not in stdout
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def main():
    assert CLIENT.is_file(), "build Linux client first"
    with listener() as probe:
        port = probe.getsockname()[1]
    for qos in (1, 2):
        with tempfile.TemporaryDirectory(prefix=f"xmqr-c110-qos{qos}-") as directory:
            activate(port, directory, qos)
            success(port, directory, qos)
            failed_output(port, directory, qos)
            filtered(port, directory, qos)
    print("C110_ACK_AFTER_OUTPUT_PASS QoS1/QoS2, failed stdout without ACK, filtered ACK")


if __name__ == "__main__":
    main()
