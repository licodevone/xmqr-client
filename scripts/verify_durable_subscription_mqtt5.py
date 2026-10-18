#!/usr/bin/env python3
"""C108: durable subscription declaration resumes without another SUBSCRIBE."""
import json
import os
import socket
import subprocess
import tempfile
import time

from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet
from verify_durable_qos1_outbox_mqtt5 import listener


def command(port, directory, topic="c108/#"):
    return [str(CLIENT), "sub", "--open-lab", "--protocol-version", "5",
            "--host", "127.0.0.1", "--port", str(port), "--topic", topic,
            "--qos", "1", "--client-id", "c108-sub", "--clean-start", "false",
            "--session-expiry-seconds", "60", "--subscription-state-dir", directory,
            "--timeout-seconds", "2"]


def stage(directory):
    with open(os.path.join(directory, "subscription.json"), encoding="utf-8") as stream:
        return json.load(stream)["pending"]["stage"]


def first(port, directory, acknowledge=True):
    with listener(port) as server:
        child = subprocess.Popen(command(port, directory), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(4)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03\x00\x00\x00")
                header, body = packet(stream)
                assert header == 0x82 and body[:3] == b"\x00\x01\x00"
                assert body[5:11] == b"c108/#" and body[-1] == 1, body
                if acknowledge:
                    stream.sendall(b"\x90\x04\x00\x01\x00\x01")
                    for _ in range(100):
                        if stage(directory) == "Active":
                            break
                        time.sleep(0.01)
                    assert stage(directory) == "Active"
                else:
                    assert stage(directory) == "Prepared"
                child.kill()
            child.communicate(timeout=5)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def resume(port, directory, present=True, expect_subscribe=False):
    with listener(port) as server:
        child = subprocess.Popen(command(port, directory), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as stream:
                stream.settimeout(3)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x03" + bytes([int(present)]) + b"\x00\x00")
                if not present:
                    assert stream.recv(1) == b""
                elif expect_subscribe:
                    assert packet(stream)[0] == 0x82
                    stream.sendall(b"\x90\x04\x00\x01\x00\x01")
                    for _ in range(100):
                        if stage(directory) == "Active":
                            break
                        time.sleep(0.01)
                    assert stage(directory) == "Active"
                    child.kill()
                else:
                    stream.sendall(b"\x30\x11\x00\x09c108/data\x00hello")
                    stream.settimeout(0.4)
                    try:
                        unexpected = packet(stream)
                    except socket.timeout:
                        unexpected = None
                    assert unexpected is None, unexpected
                    child.kill()
            stdout, stderr = child.communicate(timeout=5)
            if present and not expect_subscribe:
                assert b"hello" in stdout, (stdout, stderr)
            if not present:
                assert child.returncode != 0 and b"manifesto" in stderr.lower(), stderr
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()



def foreign_session(port, directory):
    with listener(port) as server:
        child = subprocess.Popen(command(port, directory), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE)
        with server.accept()[0] as stream:
            stream.settimeout(3)
            assert packet(stream)[0] == 0x10
            stream.sendall(b"\x20\x03\x01\x00\x00")
            assert stream.recv(1) == b""
        _, stderr = child.communicate(timeout=5)
        assert child.returncode != 0 and b"sessao existente" in stderr.lower(), stderr
    assert not os.path.exists(os.path.join(directory, "subscription.json"))


def main():
    assert CLIENT.is_file(), "build Linux client first"
    with listener() as probe:
        port = probe.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="xmqr-c108-active-") as directory:
        first(port, directory)
        resume(port, directory, present=False)
        assert stage(directory) == "Active"
        mismatched = subprocess.run(command(port, directory, "different/#"),
                                    capture_output=True, timeout=4)
        assert mismatched.returncode != 0 and b"manifesto" in mismatched.stderr.lower()
        resume(port, directory)
    with tempfile.TemporaryDirectory(prefix="xmqr-c108-prepared-") as directory:
        first(port, directory, acknowledge=False)
        resume(port, directory, expect_subscribe=True)
    with tempfile.TemporaryDirectory(prefix="xmqr-c108-foreign-") as directory:
        foreign_session(port, directory)
    print("C108_SUBSCRIPTION_WIRE_PASS active resume, no session, mismatch, uncertain SUBACK, foreign session")


if __name__ == "__main__":
    main()
