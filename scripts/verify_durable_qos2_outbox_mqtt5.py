#!/usr/bin/env python3
"""C107: QoS 2 journal resends PUBLISH or PUBREL from the durable stage."""
import json
import os
import socket
import subprocess
import tempfile

from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet
from verify_durable_qos1_outbox_mqtt5 import command as qos1_command, listener


def command(port, directory):
    args = qos1_command(port, directory)
    args[args.index("--qos") + 1] = "2"
    return args


def pending_path(directory):
    return os.path.join(directory, "pending-qos2.json")


def publish_fields(header, body):
    assert header & 0xf0 == 0x30 and header & 0x06 == 0x04
    size = int.from_bytes(body[:2], "big")
    topic = body[2:2 + size]
    packet_id = int.from_bytes(body[2 + size:4 + size], "big")
    return topic, packet_id, body[4 + size:]


def stage(directory):
    with open(pending_path(directory), encoding="utf-8") as stream:
        return json.load(stream)["pending"]["stage"]


def first_publish(directory, port, send_pubrec=False, refusal=False):
    with listener(port) as server:
        child = subprocess.Popen(command(port, directory), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x00\x00\x00")
                header, body = packet(stream)
                assert header == 0x34, hex(header)
                topic, pkid, tail = publish_fields(header, body)
                assert topic == b"c105/outbox" and pkid == 1 and tail == b"\x00first"
                if send_pubrec:
                    stream.sendall(b"\x50\x04\x00\x01\x87\x00" if refusal
                                   else b"\x50\x02\x00\x01")
                    if not refusal:
                        assert packet(stream) == (0x62, b"\x00\x01")
                        assert stage(directory) == "PubRelReady"
                    else:
                        _, stderr = child.communicate(timeout=5)
                        assert child.returncode != 0 and stderr
            if not refusal:
                _, stderr = child.communicate(timeout=5)
                assert child.returncode != 0 and b"outbox" in stderr.lower()
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
    assert os.path.isfile(pending_path(directory)) is not refusal
    if not refusal:
        assert stage(directory) == ("PubRelReady" if send_pubrec else "PublishMayHaveSent")


def resume(directory, port, expected_header, session_present=True, negative_pubcomp=False):
    with listener(port) as server:
        child = subprocess.Popen(command(port, directory), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03" + bytes([int(session_present)]) + b"\x00\x00")
                if not session_present:
                    assert stream.recv(1) == b""
                elif expected_header == 0x3c:
                    header, body = packet(stream)
                    assert header == 0x3c, hex(header)
                    topic, pkid, tail = publish_fields(header, body)
                    assert topic == b"c105/outbox" and pkid == 1 and tail == b"\x00first"
                    stream.sendall(b"\x50\x02\x00\x01")
                    assert packet(stream) == (0x62, b"\x00\x01")
                    assert stage(directory) == "PubRelReady"
                    stream.sendall(b"\x70\x02\x00\x01")
                    assert packet(stream) == (0xe0, b"")
                else:
                    assert packet(stream) == (0x62, b"\x00\x01")
                    stream.sendall(b"\x70\x04\x00\x01\x92\x00" if negative_pubcomp
                                   else b"\x70\x02\x00\x01")
                    if not negative_pubcomp:
                        assert packet(stream) == (0xe0, b"")
                _, stderr = child.communicate(timeout=5)
                assert (child.returncode == 0) is (session_present and not negative_pubcomp), stderr
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
    assert os.path.isfile(pending_path(directory)) is not session_present


def main():
    assert CLIENT.is_file(), "build the Linux client first"
    with listener() as server:
        port = server.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="xmqr-c107-publish-") as directory:
        first_publish(directory, port)
        resume(directory, port, 0x3c, False)
        resume(directory, port, 0x3c)
    with tempfile.TemporaryDirectory(prefix="xmqr-c107-pubrel-") as directory:
        first_publish(directory, port, send_pubrec=True)
        resume(directory, port, 0x62)
    with tempfile.TemporaryDirectory(prefix="xmqr-c107-refusal-") as directory:
        first_publish(directory, port, send_pubrec=True, refusal=True)
    with tempfile.TemporaryDirectory(prefix="xmqr-c107-pubcomp-refusal-") as directory:
        first_publish(directory, port, send_pubrec=True)
        resume(directory, port, 0x62, negative_pubcomp=True)
    print("C107_QOS2_OUTBOX_WIRE_PASS PUBLISH DUP, PUBREL resume, no session, negative PUBREC/PUBCOMP")


if __name__ == "__main__":
    main()
