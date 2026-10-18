#!/usr/bin/env python3
"""C114: isolated WSS mTLS, MQTT subprotocol, pub/sub and TLS negatives."""
import base64
import hashlib
import os
from pathlib import Path
import socket
import ssl
import subprocess
import tempfile
import threading


ROOT = Path(__file__).resolve().parents[1]
CLIENT = ROOT / "target/debug/mqtt-client"


def openssl(root, *args):
    subprocess.run(["openssl", *map(str, args)], cwd=root, check=True,
                   capture_output=True, timeout=15)


def certificates(root):
    openssl(root, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
            "-keyout", "ca.key", "-out", "ca.crt", "-subj", "/CN=C114-CA",
            "-addext", "basicConstraints=critical,CA:TRUE",
            "-addext", "keyUsage=critical,keyCertSign,cRLSign")
    for serial, name, san, usage in (
        (2, "server", "IP:127.0.0.1", "serverAuth"),
        (3, "wrong-name", "IP:192.0.2.1", "serverAuth"),
        (4, "client", "DNS:device.fixture", "clientAuth"),
    ):
        openssl(root, "req", "-new", "-newkey", "rsa:2048", "-nodes",
                "-keyout", f"{name}.key", "-out", f"{name}.csr",
                "-subj", f"/CN={name}")
        (root / f"{name}.ext").write_text(
            "basicConstraints=critical,CA:FALSE\n"
            "keyUsage=critical,digitalSignature,keyEncipherment\n"
            f"extendedKeyUsage={usage}\nsubjectAltName={san}\n")
        openssl(root, "x509", "-req", "-in", f"{name}.csr",
                "-CA", "ca.crt", "-CAkey", "ca.key", "-set_serial", str(serial),
                "-days", "1", "-extfile", f"{name}.ext", "-out", f"{name}.crt")
    openssl(root, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
            "-keyout", "other.key", "-out", "other.crt", "-subj", "/CN=C114-other",
            "-addext", "basicConstraints=critical,CA:TRUE")


def exact(stream, length):
    data = bytearray()
    while len(data) < length:
        part = stream.recv(length - len(data))
        if not part:
            raise EOFError("WSS closed")
        data.extend(part)
    return bytes(data)


def read_frame(stream):
    first, second = exact(stream, 2)
    size = second & 127
    if size == 126:
        size = int.from_bytes(exact(stream, 2), "big")
    elif size == 127:
        size = int.from_bytes(exact(stream, 8), "big")
    assert second & 0x80 and size < 65536
    mask = exact(stream, 4)
    body = exact(stream, size)
    return first & 15, bytes(byte ^ mask[index % 4]
                             for index, byte in enumerate(body))


def send_frame(stream, body):
    length = len(body)
    assert length < 126
    stream.sendall(bytes([0x82, length]) + body)


def serve(root, listener, *, mqtt5, subscribe, no_protocol, wrong_san, errors):
    try:
        raw, _ = listener.accept()
        raw.settimeout(5)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        name = "wrong-name" if wrong_san else "server"
        context.load_cert_chain(str(root / f"{name}.crt"), str(root / f"{name}.key"))
        context.load_verify_locations(str(root / "ca.crt"))
        context.verify_mode = ssl.CERT_REQUIRED
        with context.wrap_socket(raw, server_side=True) as stream:
            request = bytearray()
            while not request.endswith(b"\r\n\r\n"):
                request.extend(exact(stream, 1))
                assert len(request) < 4096
            assert request.startswith(b"GET /mqtt HTTP/1.1\r\n"), request
            assert b"sec-websocket-protocol: mqtt" in request.lower()
            key = next(line.split(b":", 1)[1].strip() for line in request.split(b"\r\n")
                       if line.lower().startswith(b"sec-websocket-key:"))
            accept = base64.b64encode(hashlib.sha1(
                key + b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest())
            response = (b"HTTP/1.1 101 Switching Protocols\r\n"
                        b"Upgrade: websocket\r\nConnection: Upgrade\r\n"
                        b"Sec-WebSocket-Accept: " + accept + b"\r\n")
            if not no_protocol:
                response += b"Sec-WebSocket-Protocol: mqtt\r\n"
            stream.sendall(response + b"\r\n")
            if no_protocol:
                return
            opcode, connect = read_frame(stream)
            assert opcode == 2 and connect[0] == 0x10
            assert b"MQTT" + bytes([5 if mqtt5 else 4]) in connect
            assert b"device" in connect and b"temporary-secret" in connect
            send_frame(stream, b"\x20\x03\x00\x00\x00" if mqtt5 else b"\x20\x02\x00\x00")
            opcode, request = read_frame(stream)
            assert opcode == 2
            if subscribe:
                assert request[0] == 0x82
                packet_id = request[2:4]
                send_frame(stream, b"\x90\x04" + packet_id + b"\x00\x00" if mqtt5
                           else b"\x90\x03" + packet_id + b"\x00")
                body = b"\x00\x0atest/value" + (b"\x00" if mqtt5 else b"") + b"hello"
                send_frame(stream, b"\x30" + bytes([len(body)]) + body)
            else:
                assert request[0] & 0xf0 == 0x30 and request.endswith(b"ok"), request
            for _ in range(4):
                opcode, packet = read_frame(stream)
                if opcode == 8 or packet[:1] == b"\xe0":
                    return
                if subscribe and packet[:1] == b"\xa2":
                    packet_id = packet[2:4]
                    send_frame(stream, b"\xb0\x04" + packet_id + b"\x00\x00" if mqtt5
                               else b"\xb0\x02" + packet_id)
                    continue
                raise AssertionError(f"unexpected MQTT packet: {packet!r}")
            raise AssertionError("MQTT DISCONNECT missing")
    except (ssl.SSLError, OSError, EOFError) as error:
        if not wrong_san:
            errors.append(error)
    except BaseException as error:
        errors.append(error)
    finally:
        listener.close()


def scenario(root, *, mqtt5=False, subscribe=False, no_protocol=False,
             wrong_san=False, wrong_ca=False):
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    port = listener.getsockname()[1]
    errors = []
    worker = threading.Thread(
        target=serve,
        args=(root, listener),
        kwargs=dict(mqtt5=mqtt5, subscribe=subscribe, no_protocol=no_protocol,
                    wrong_san=wrong_san, errors=errors),
        daemon=True)
    worker.start()
    command = [
        str(CLIENT), "sub" if subscribe else "pub",
        "--ws-url", f"wss://127.0.0.1:{port}/mqtt",
        "--topic", "test/value", "--username", "device",
        "--ca", str(root / ("other.crt" if wrong_ca else "ca.crt")),
        "--cert", str(root / "client.crt"), "--key", str(root / "client.key"),
        "--password-file", str(root / "password"),
        "--protocol-version", "5" if mqtt5 else "3.1.1",
        "--client-id", "c114-fixture", "--timeout-seconds", "3",
    ]
    command += ["--count", "1"] if subscribe else ["--message", "ok"]
    result = subprocess.run(command, capture_output=True, timeout=9)
    worker.join(timeout=6)
    assert not worker.is_alive(), "WSS fixture did not finish"
    if no_protocol or wrong_san or wrong_ca:
        assert result.returncode != 0, result.stdout
    else:
        assert result.returncode == 0, result.stderr.decode(errors="replace")
        assert not errors, errors
        if subscribe:
            assert b"hello" in result.stdout, result.stdout


def main():
    assert CLIENT.is_file(), "build the client before this fixture"
    with tempfile.TemporaryDirectory(prefix="xmqr-c114-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        certificates(root)
        (root / "password").write_text("temporary-secret\n")
        (root / "password").chmod(0o600)
        for mqtt5 in (False, True):
            for subscribe in (False, True):
                scenario(root, mqtt5=mqtt5, subscribe=subscribe)
            scenario(root, mqtt5=mqtt5, no_protocol=True)
        scenario(root, wrong_san=True)
        scenario(root, wrong_ca=True)
    print("C114_WSS_MTLS_FIXTURE_PASS")


if __name__ == "__main__":
    main()
