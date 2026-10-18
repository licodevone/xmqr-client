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


def serve(root, listener, *, mqtt5, subscribe, no_protocol, wrong_san, durable,
          drop_ack, errors, session_present=False, expect_dup=False,
          no_session=False, close_listener=True, qos2=False,
          resume_pubrel=False, drop_comp=False, refusal=None):
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
            send_frame(stream, b"\x20\x03" + bytes([int(session_present)]) + b"\x00\x00"
                       if mqtt5 else b"\x20\x02\x00\x00")
            if no_session:
                try:
                    opcode, packet = read_frame(stream)
                    assert opcode == 8 or packet[:1] == b"\xe0", packet
                except (EOFError, OSError):
                    pass
                return
            opcode, request = read_frame(stream)
            assert opcode == 2
            if subscribe:
                assert request[0] == 0x82
                packet_id = request[2:4]
                send_frame(stream, b"\x90\x04" + packet_id + b"\x00\x00" if mqtt5
                           else b"\x90\x03" + packet_id + b"\x00")
                body = b"\x00\x0atest/value" + (b"\x00" if mqtt5 else b"") + b"hello"
                send_frame(stream, b"\x30" + bytes([len(body)]) + body)
            elif resume_pubrel:
                assert request == b"\x62\x02\x00\x01", request
                if drop_comp:
                    return
                send_frame(stream, b"\x70\x04\x00\x01\x92\x00" if refusal == "pubcomp"
                           else b"\x70\x02\x00\x01")
                if refusal:
                    try:
                        read_frame(stream)
                    except (EOFError, OSError):
                        pass
                    return
            else:
                assert request[0] & 0xf0 == 0x30 and request.endswith(b"ok"), request
                if durable:
                    topic_length = int.from_bytes(request[2:4], "big")
                    packet_id = request[4 + topic_length:6 + topic_length]
                    expected_header = (0x3c if expect_dup else 0x34) if qos2 else (
                        0x3a if expect_dup else 0x32)
                    assert request[0] == expected_header, request
                    assert packet_id == b"\x00\x01", request
                    if drop_ack:
                        return
                    if qos2:
                        send_frame(stream, b"\x50\x04" + packet_id + b"\x87\x00"
                                   if refusal == "pubrec" else b"\x50\x02" + packet_id)
                        if refusal == "pubrec":
                            try:
                                read_frame(stream)
                            except (EOFError, OSError):
                                pass
                            return
                        opcode, pubrel = read_frame(stream)
                        assert opcode == 2 and pubrel == b"\x62\x02" + packet_id, pubrel
                        if drop_comp:
                            return
                        send_frame(stream, b"\x70\x04" + packet_id + b"\x92\x00"
                                   if refusal == "pubcomp" else b"\x70\x02" + packet_id)
                        if refusal:
                            try:
                                read_frame(stream)
                            except (EOFError, OSError):
                                pass
                            return
                    else:
                        send_frame(stream, b"\x40\x04" + packet_id + b"\x00\x00")
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
        if close_listener:
            listener.close()


def scenario(root, *, mqtt5=False, subscribe=False, no_protocol=False,
             wrong_san=False, wrong_ca=False, profile=False, durable=False,
             drop_ack=False, qos2=False, refusal=None):
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    port = listener.getsockname()[1]
    errors = []
    worker = threading.Thread(
        target=serve,
        args=(root, listener),
        kwargs=dict(mqtt5=mqtt5, subscribe=subscribe, no_protocol=no_protocol,
                    wrong_san=wrong_san, durable=durable, drop_ack=drop_ack,
                    qos2=qos2, refusal=refusal, errors=errors),
        daemon=True)
    worker.start()
    url = f"wss://127.0.0.1:{port}/mqtt"
    connection = [
        "--ws-url", url, "--username", "device",
        "--ca", str(root / ("other.crt" if wrong_ca else "ca.crt")),
        "--cert", str(root / "client.crt"), "--key", str(root / "client.key"),
        "--password-file", str(root / "password"),
    ]
    if profile:
        profile_path = root / "c116-connection.toml"
        profile_path.write_text(
            f'version=1\n[connection]\nws_url="{url}"\nusername="device"\n'
            'ca="ca.crt"\ncert="client.crt"\nkey="client.key"\n'
            'password_file="password"\n')
        connection = ["--profile", str(profile_path)]
    command = [str(CLIENT), "sub" if subscribe else "pub", *connection,
               "--topic", "test/value", "--protocol-version",
               "5" if mqtt5 else "3.1.1", "--client-id", "c116-fixture",
               "--timeout-seconds", "3"]
    command += ["--count", "1"] if subscribe else ["--message", "ok"]
    if durable:
        assert mqtt5 and not subscribe
        outbox = root / f"outbox-{port}"
        outbox.mkdir(mode=0o700)
        command.extend(["--qos", "2" if qos2 else "1", "--clean-start", "false",
                        "--session-expiry-seconds", "60", "--outbox-dir", str(outbox)])
    result = subprocess.run(command, capture_output=True, timeout=9)
    worker.join(timeout=6)
    assert not worker.is_alive(), "WSS fixture did not finish"
    pending = outbox / ("pending-qos2.json" if qos2 else "pending.json") if durable else None
    if drop_ack:
        assert result.returncode != 0 and not errors, (result.stderr, errors)
        before = pending.read_bytes()
        assert not profile
        wrong = [f"wss://127.0.0.1:{port + 1}/mqtt" if arg == url else arg
                 for arg in command]
        rejected = subprocess.run(wrong, capture_output=True, timeout=9)
        assert rejected.returncode != 0 and b"outbox" in rejected.stderr.lower()
        assert pending.read_bytes() == before, "URL mismatch modified journal"
        return
    if refusal:
        assert result.returncode != 0 and not errors, (result.stderr, errors)
        assert not pending.exists(), "broker refusal must complete QoS2 journal"
        return
    if no_protocol or wrong_san or wrong_ca:
        assert result.returncode != 0, result.stdout
    else:
        assert result.returncode == 0, result.stderr.decode(errors="replace")
        assert not errors, errors
        if subscribe:
            assert b"hello" in result.stdout, result.stdout
        if durable:
            assert not pending.exists(), "ACK must complete outbox"


def recovery_scenario(root, *, resume_session, qos2=False, first_drop_comp=False):
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    port = listener.getsockname()[1]
    outbox = root / f"recovery-{port}"
    outbox.mkdir(mode=0o700)
    secret = root / "client-password"
    secret.write_text("temporary-secret\n")
    secret.chmod(0o600)
    command = [str(CLIENT), "pub", "--ws-url", f"wss://127.0.0.1:{port}/mqtt",
               "--topic", "test/value", "--message", "ok", "--username", "device",
               "--password-file", str(secret), "--ca", str(root / "ca.crt"),
               "--cert", str(root / "client.crt"), "--key", str(root / "client.key"),
               "--protocol-version", "5", "--qos", "2" if qos2 else "1",
               "--client-id", "c122-fixture" if qos2 else "c120-fixture",
               "--clean-start", "false", "--session-expiry-seconds", "60",
               "--outbox-dir", str(outbox), "--timeout-seconds", "3"]
    options = dict(mqtt5=True, subscribe=False, no_protocol=False, wrong_san=False,
                   durable=True, drop_ack=not first_drop_comp, qos2=qos2,
                   drop_comp=first_drop_comp)
    errors = []
    first = threading.Thread(target=serve, args=(root, listener),
                             kwargs=dict(options, errors=errors, close_listener=False), daemon=True)
    first.start()
    interrupted = subprocess.run(command, capture_output=True, timeout=9)
    first.join(timeout=6)
    assert not first.is_alive() and not errors, errors
    assert interrupted.returncode != 0, interrupted.stdout
    pending = outbox / ("pending-qos2.json" if qos2 else "pending.json")
    before = pending.read_bytes()
    expected_stage = b'PubRelReady' if first_drop_comp else (
        b'PublishMayHaveSent' if qos2 else b'MayHaveSent')
    assert expected_stage in before, before
    second = threading.Thread(target=serve, args=(root, listener),
                              kwargs=dict(options, errors=errors, drop_ack=False,
                                          drop_comp=False,
                                          session_present=resume_session,
                                          expect_dup=resume_session and not first_drop_comp,
                                          resume_pubrel=resume_session and first_drop_comp,
                                          no_session=not resume_session), daemon=True)
    second.start()
    retried = subprocess.run(command, capture_output=True, timeout=9)
    second.join(timeout=6)
    assert not second.is_alive() and not errors, (errors, retried.stderr)
    if resume_session:
        assert retried.returncode == 0, retried.stderr
        assert not pending.exists(), "PUBACK must clear resumed outbox"
    else:
        assert retried.returncode != 0 and b"sessao" in retried.stderr.lower()
        assert pending.read_bytes() == before, "Session Present=0 modified journal"


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
                scenario(root, mqtt5=mqtt5, subscribe=subscribe, profile=True)
            scenario(root, mqtt5=mqtt5, no_protocol=True)
        scenario(root, wrong_san=True)
        scenario(root, wrong_ca=True)
        scenario(root, mqtt5=True, durable=True)
        scenario(root, mqtt5=True, durable=True, profile=True)
        scenario(root, mqtt5=True, durable=True, drop_ack=True)
        recovery_scenario(root, resume_session=True)
        recovery_scenario(root, resume_session=False)
        recovery_scenario(root, resume_session=True, qos2=True)
        recovery_scenario(root, resume_session=False, qos2=True)
        recovery_scenario(root, resume_session=True, qos2=True, first_drop_comp=True)
        recovery_scenario(root, resume_session=False, qos2=True, first_drop_comp=True)
        scenario(root, mqtt5=True, durable=True, qos2=True)
        scenario(root, mqtt5=True, durable=True, qos2=True, refusal="pubrec")
        scenario(root, mqtt5=True, durable=True, qos2=True, refusal="pubcomp")
    print("C114_WSS_MTLS_FIXTURE_PASS C116_PROFILE_FIXTURE_PASS "
          "C119_WSS_QOS1_OUTBOX_PASS C120_WSS_QOS1_RECOVERY_PASS "
          "C122_WSS_QOS2_OUTBOX_PASS")


if __name__ == "__main__":
    main()
