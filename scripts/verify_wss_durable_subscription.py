#!/usr/bin/env python3
"""C123: WSS mTLS persistent subscription manifest and fail-closed recovery."""
import base64
import hashlib
import json
from pathlib import Path
import select
import socket
import ssl
import subprocess
import tempfile
import threading
import time

from verify_wss_client import CLIENT, certificates, exact, read_frame, send_frame


def command(root, url, directory):
    return [str(CLIENT), "sub", "--ws-url", url, "--topic", "test/#",
            "--username", "device", "--password-file", str(root / "password"),
            "--ca", str(root / "ca.crt"), "--cert", str(root / "client.crt"),
            "--key", str(root / "client.key"), "--protocol-version", "5",
            "--qos", "1", "--client-id", "c123-wss-sub", "--clean-start", "false",
            "--session-expiry-seconds", "60", "--subscription-state-dir", str(directory),
            "--output", "jsonl", "--timeout-seconds", "3"]


def pending(directory):
    return json.loads((directory / "subscription.json").read_text())["pending"]


def serve(root, listener, child, mode, directory, errors):
    try:
        raw, _ = listener.accept()
        raw.settimeout(4)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(str(root / "server.crt"), str(root / "server.key"))
        context.load_verify_locations(str(root / "ca.crt"))
        context.verify_mode = ssl.CERT_REQUIRED
        with context.wrap_socket(raw, server_side=True) as stream:
            request = bytearray()
            while not request.endswith(b"\r\n\r\n"):
                request.extend(exact(stream, 1))
                assert len(request) < 4096
            assert request.startswith(b"GET /mqtt HTTP/1.1\r\n")
            assert b"sec-websocket-protocol: mqtt" in request.lower()
            key = next(line.split(b":", 1)[1].strip() for line in request.split(b"\r\n")
                       if line.lower().startswith(b"sec-websocket-key:"))
            accept = base64.b64encode(hashlib.sha1(
                key + b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest())
            stream.sendall(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n"
                           b"Connection: Upgrade\r\nSec-WebSocket-Protocol: mqtt\r\n"
                           b"Sec-WebSocket-Accept: " + accept + b"\r\n\r\n")
            opcode, connect = read_frame(stream)
            assert opcode == 2 and connect[0] == 0x10 and b"MQTT\x05" in connect
            assert b"c123-wss-sub" in connect
            send_frame(stream, b"\x20\x03" + bytes([int(mode in ("resume", "prepared"))])
                       + b"\x00\x00")
            if mode == "no-session":
                try:
                    opcode, data = read_frame(stream)
                    assert opcode == 8 or data[:1] == b"\xe0", data
                except (OSError, EOFError):
                    pass
                return
            if mode in ("first", "uncertain", "prepared"):
                opcode, subscribe = read_frame(stream)
                assert opcode == 2 and subscribe[0] == 0x82, subscribe
                assert b"test/#" in subscribe
                if mode == "uncertain":
                    return
                send_frame(stream, b"\x90\x04" + subscribe[2:4] + b"\x00\x01")
                if mode == "prepared":
                    for _ in range(100):
                        if pending(directory)["stage"] == "Active":
                            break
                        time.sleep(.01)
                    assert pending(directory)["stage"] == "Active"
            if mode in ("first", "resume"):
                topic = b"test/value"
                body = len(topic).to_bytes(2, "big") + topic + b"\x00\x07\x00hello"
                send_frame(stream, b"\x32" + bytes([len(body)]) + body)
                ready, _, _ = select.select([child.stdout], [], [], 4)
                assert ready, "subscriber did not flush output"
                line = child.stdout.readline()
                assert b'"event":"publish"' in line, line
                opcode, ack = read_frame(stream)
                assert opcode == 2 and ack[0] == 0x40 and ack[2:4] == b"\x00\x07", ack
    except BaseException as error:
        errors.append(error)


def exchange(root, listener, directory, mode):
    url = f"wss://127.0.0.1:{listener.getsockname()[1]}/mqtt"
    child = subprocess.Popen(command(root, url, directory), stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE)
    errors = []
    worker = threading.Thread(target=serve,
                              args=(root, listener, child, mode, directory, errors), daemon=True)
    worker.start()
    worker.join(timeout=9)
    assert not worker.is_alive() and not errors, errors
    if mode == "no-session":
        _, stderr = child.communicate(timeout=5)
        assert child.returncode != 0 and b"manifesto" in stderr.lower(), stderr
    else:
        if child.poll() is None:
            child.kill()
        child.communicate(timeout=5)
    return url


def main():
    assert CLIENT.is_file(), "build Linux client first"
    with tempfile.TemporaryDirectory(prefix="xmqr-c123-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        certificates(root)
        (root / "password").write_text("temporary-secret\n")
        (root / "password").chmod(0o600)
        listener = socket.socket()
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(8)
        try:
            directory = root / "active"
            directory.mkdir(mode=0o700)
            url = exchange(root, listener, directory, "first")
            manifest = pending(directory)
            assert manifest["stage"] == "Active"
            assert manifest["spec"]["host"] == url and manifest["spec"]["port"] == 0
            before = (directory / "subscription.json").read_bytes()
            mismatched = subprocess.run(command(root, url.replace("/mqtt", "/other"), directory),
                                        capture_output=True, timeout=6)
            assert mismatched.returncode != 0 and b"manifesto" in mismatched.stderr.lower()
            assert (directory / "subscription.json").read_bytes() == before
            exchange(root, listener, directory, "no-session")
            assert (directory / "subscription.json").read_bytes() == before
            exchange(root, listener, directory, "resume")
            uncertain = root / "uncertain"
            uncertain.mkdir(mode=0o700)
            exchange(root, listener, uncertain, "uncertain")
            assert pending(uncertain)["stage"] == "Prepared"
            exchange(root, listener, uncertain, "prepared")
            assert pending(uncertain)["stage"] == "Active"
        finally:
            listener.close()
    print("C123_WSS_DURABLE_SUBSCRIPTION_PASS")


if __name__ == "__main__":
    main()
