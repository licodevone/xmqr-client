#!/usr/bin/env python3
"""C92: real TLS wire contract for client-initiated MQTT 5 PLAIN re-authentication."""
import argparse
import importlib.util
from pathlib import Path
import secrets
import socket
import ssl
import subprocess
import tempfile
import threading
import time


ROOT = Path(__file__).resolve().parents[1]


def fixture_module():
    spec = importlib.util.spec_from_file_location("c27", ROOT / "validation/C27-integration.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def trial(client, root, fixture, secret, scenario):
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(8)
    port = listener.getsockname()[1]
    errors = []
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(root / "server.crt", root / "server.key")

    def serve():
        try:
            with listener:
                socket_stream, _ = listener.accept()
                with context.wrap_socket(socket_stream, server_side=True) as stream:
                    stream.settimeout(8)
                    header, connect = fixture.read_packet(stream)
                    assert header == 0x10 and connect[7] & 0xc0 == 0, connect
                    assert b"\x15\x00\x05PLAIN" in connect
                    stream.sendall(b"\x20\x0d\x00\x00\x0a\x2a\x01\x15\x00\x05PLAIN")
                    header, auth = fixture.read_packet(stream)
                    expected_data = b"\0fixture-device\0" + secret.encode()
                    assert header == 0xf0 and auth[:2] == bytes([0x19, len(auth)-2]), auth
                    assert b"\x15\x00\x05PLAIN" in auth and auth.endswith(expected_data), auth
                    if scenario == "success":
                        stream.sendall(b"\xf0\x0a\x00\x08\x15\x00\x05PLAIN")
                        header, publish = fixture.read_packet(stream)
                        assert header == 0x30 and b"c92/value" in publish
                        assert publish.endswith(b"through-auth"), publish
                        try:
                            stream.settimeout(2)
                            stream.recv(1)
                        except socket.timeout:
                            pass
                    elif scenario == "wrong-method":
                        stream.sendall(b"\xf0\x0a\x00\x08\x15\x00\x05OTHER")
                    elif scenario == "missing-method":
                        stream.sendall(b"\xf0\x02\x00\x00")
                    elif scenario == "denied":
                        stream.sendall(b"\xe0\x02\x86\x00")
                    elif scenario == "timeout":
                        time.sleep(2)
                    else:
                        raise AssertionError(scenario)
        except Exception as error:
            errors.append(error)

    worker = threading.Thread(target=serve, daemon=True)
    worker.start()
    command = [str(client), "pub", "--protocol-version", "5", "--auth-method", "plain",
               "--reauth-before-io", "--tls-password", "--host", "127.0.0.1", "--port",
               str(port), "--ca", str(root / "ca.crt"), "--username", "fixture-device",
               "--password-file", str(root / "password"), "--topic", "c92/value",
               "--message", "through-auth", "--timeout-seconds", "1"]
    result = subprocess.run(command, capture_output=True, text=True, timeout=9)
    worker.join(timeout=9)
    assert not worker.is_alive() and not errors, errors
    assert secret not in result.stdout + result.stderr
    assert (result.returncode == 0) == (scenario == "success"), (scenario, result)
    if scenario == "timeout":
        assert "tempo esgotado" in result.stderr, result.stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", type=Path, required=True)
    args = parser.parse_args()
    fixture = fixture_module()
    with tempfile.TemporaryDirectory(prefix="xmqr-c92-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        fixture.certificates(root)
        secret = secrets.token_urlsafe(24)
        (root / "password").write_text(secret + "\n")
        (root / "password").chmod(0o600)
        for scenario in ("success", "wrong-method", "missing-method", "denied", "timeout"):
            trial(args.client, root, fixture, secret, scenario)
    print("C92_AUTH_WIRE_PASS exact AUTH, invalid confirmations, timeout and no secret output")


if __name__ == "__main__":
    main()
