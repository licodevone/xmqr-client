# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""C27 isolated real-broker + TLS-peer contracts. Explicit binaries; no installs.

All ports/processes/state belong to this test. Does not edit/build the broker.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import queue
import secrets
import signal
import socket
import ssl
import struct
import subprocess
import sys
import tempfile
import threading
import time
import unittest

sys.dont_write_bytecode = True
CLIENT = None
fixtures = None
LIMIT = 8


class Captured:
    def __init__(self, args):
        self.process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                        text=True, encoding="utf-8")
        self.out, self.err = [], []
        self.stdout, self.stderr = queue.Queue(), queue.Queue()
        self.threads = []
        for pipe, lines, mailbox in ((self.process.stdout, self.out, self.stdout),
                                     (self.process.stderr, self.err, self.stderr)):
            def read(stream=pipe, collected=lines, box=mailbox):
                with stream:
                    for line in stream:
                        collected.append(line.rstrip("\n"))
                        box.put(line.rstrip("\n"))
            reader = threading.Thread(target=read, daemon=True)
            reader.start()
            self.threads.append(reader)

    def wait_line(self, mailbox, marker):
        until = time.monotonic() + LIMIT
        while time.monotonic() < until:
            line = mailbox.get(timeout=max(.01, until-time.monotonic()))
            if marker in line:
                return line
        raise AssertionError("bounded client readiness failed")

    def finish(self):
        code = self.process.wait(timeout=LIMIT)
        for reader in self.threads:
            reader.join(timeout=1)
            assert not reader.is_alive()
        return code

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait(timeout=LIMIT)
        for reader in self.threads:
            reader.join(timeout=1)


class Contracts(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="xmqr-c27-isolated-")
        self.broker = fixtures.Broker(self.directory.name)
        self.addCleanup(self.directory.cleanup)
        self.addCleanup(self.broker.stop)
        self.broker.start()

    def cli(self, command, *tail):
        return [str(CLIENT), command, "--open-lab", "--host", "127.0.0.1",
                "--port", str(self.broker.port), *tail]

    def publish(self, topic, message, qos="1"):
        result = subprocess.run(self.cli("pub", "--topic", topic, "--message", message,
                                        "--qos", qos, "--output", "jsonl"),
                                capture_output=True, text=True, timeout=LIMIT)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {
            "event": "publish_complete", "qos": int(qos),
            "confirmation": "sent" if qos == "0" else "broker_protocol_ack"})

    def test_json_binary_1024_utf8_and_retained_qos_0_1_2(self):
        topic = "é" * 512
        payload = bytes([0, 255, 10, 34])
        for qos in (0, 1, 2):
            with fixtures.WireClient(self.broker, f"binary-source-{qos}") as source:
                identifier = b"\0\x07" if qos else b""
                source.send(0x31 | (qos << 1), fixtures.text(topic) + identifier + payload)
                if qos:
                    self.assertEqual(source.receive(), (0x40 if qos == 1 else 0x50, identifier))
                    if qos == 2:
                        source.send(0x62, identifier)
                        self.assertEqual(source.receive(), (0x70, identifier))
                source.send(0xe0)
            result = subprocess.run(self.cli("sub", "--topic", topic, "--count", "1",
                                            "--qos", str(qos), "--output", "jsonl"),
                                    capture_output=True, text=True, timeout=LIMIT)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout), {
                "event": "publish", "topic": topic, "qos": qos,
                "retain": True, "payload_bytes": list(payload)})
            self.assertIn("Assinatura ativa", result.stderr)
            self.publish("completion", "ok", str(qos))

    def test_broker_restart_preserves_count_id_and_both_session_modes(self):
        for clean in (False, True):
            client = Captured(self.cli("sub", "--topic", "restart", "--count", "2",
                "--qos", "1", "--output", "jsonl", "--client-id", f"restart-{int(clean)}",
                "--clean-session", str(clean).lower(), "--reconnect-attempts", "10"))
            self.addCleanup(client.close)
            client.wait_line(client.stderr, "Assinatura ativa")
            self.publish("restart", "before")
            first = json.loads(client.stdout.get(timeout=LIMIT))
            self.assertEqual(first["payload_bytes"], list(b"before"))
            self.broker.stop()
            client.wait_line(client.stderr, "Reconexao")
            self.broker.start()
            if clean:
                client.wait_line(client.stderr, "Assinatura ativa")
            # Persistent session can queue this publication before reconnect completes.
            self.publish("restart", "after")
            second = json.loads(client.stdout.get(timeout=LIMIT))
            self.assertEqual(second["payload_bytes"], list(b"after"))
            self.assertEqual(client.finish(), 0, client.err)
            self.assertEqual(len(client.out), 2)

    def test_sigint_live_cancels_will_and_backoff_sigint_stays_bounded(self):
        with fixtures.WireClient(self.broker, "will-cancel-observer") as observer:
            observer.subscribe([("status/c27", 2)])
            client = Captured(self.cli("sub", "--topic", "unused", "--output", "jsonl",
                "--will-topic", "status/c27", "--will-message", "cancelled",
                "--will-qos", "2", "--will-retain", "true"))
            self.addCleanup(client.close)
            client.wait_line(client.stderr, "Assinatura ativa")
            client.process.send_signal(signal.SIGINT)
            self.assertEqual(client.finish(), 0, client.err)
            self.assertEqual(client.out, [])
            observer.socket.settimeout(.3)
            with self.assertRaises(socket.timeout):
                observer.receive()
            observer.socket.settimeout(fixtures.TIMEOUT)
            observer.ping()
            observer.send(0xe0)
        self.broker.stop()
        client = Captured(self.cli("pub", "--topic", "unused", "--message", "never-sent",
                                   "--output", "jsonl", "--reconnect-attempts", "10"))
        self.addCleanup(client.close)
        client.wait_line(client.stderr, "Reconexao")
        begin = time.monotonic()
        client.process.send_signal(signal.SIGINT)
        self.assertEqual(client.finish(), 0, client.err)
        self.assertLess(time.monotonic() - begin, 2)
        self.assertEqual(client.out, [])


def openssl(root, *args):
    subprocess.run(["openssl", *args], cwd=root, capture_output=True, check=True, timeout=15)


def certificates(root):
    openssl(root, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
            "-keyout", "ca.key", "-out", "ca.crt", "-subj", "/CN=C27-test-CA",
            "-addext", "basicConstraints=critical,CA:TRUE",
            "-addext", "keyUsage=critical,keyCertSign,cRLSign")
    for serial, name, san, usage in ((2, "server", "IP:127.0.0.1", "serverAuth"),
                                    (3, "wrong-name", "IP:192.0.2.1", "serverAuth"),
                                    (4, "client", "DNS:client.fixture", "clientAuth")):
        openssl(root, "req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout",
                name+".key", "-out", name+".csr", "-subj", "/CN="+name)
        (root/(name+".ext")).write_text("basicConstraints=critical,CA:FALSE\n"
            "keyUsage=critical,digitalSignature,keyEncipherment\n"
            f"extendedKeyUsage={usage}\nsubjectAltName={san}\n")
        openssl(root, "x509", "-req", "-in", name+".csr", "-CA", "ca.crt",
                "-CAkey", "ca.key", "-set_serial", str(serial), "-days", "1",
                "-extfile", name+".ext", "-out", name+".crt")
    openssl(root, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
            "-keyout", "other.key", "-out", "other.crt", "-subj", "/CN=C27-other-CA",
            "-addext", "basicConstraints=critical,CA:TRUE")


def read_packet(stream):
    def exact(n):
        data = b""
        while len(data) < n:
            part = stream.recv(n-len(data))
            if not part:
                raise EOFError()
            data += part
        return data
    header = exact(1)[0]
    size, multiplier = 0, 1
    for _ in range(4):
        digit = exact(1)[0]
        size += (digit & 127)*multiplier
        assert size <= 65536
        if digit & 128 == 0:
            return header, exact(size)
        multiplier *= 128
    raise AssertionError("remaining length")


class TLSContracts(unittest.TestCase):
    def test_mtls_valid_ca_and_san_then_negative_no_retry(self):
        with tempfile.TemporaryDirectory(prefix="xmqr-c27-tls-", dir="/tmp") as directory:
            root = Path(directory)
            root.chmod(0o700)
            certificates(root)
            secret = secrets.token_hex(16)
            password = root/"password"
            password.write_text(secret)
            password.chmod(0o600)
            for server_name, trust, good in (("server", "ca", True),
                                           ("wrong-name", "ca", False),
                                           ("server", "other", False)):
                context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
                context.load_cert_chain(root/(server_name+".crt"), root/(server_name+".key"))
                context.load_verify_locations(root/"ca.crt")
                context.verify_mode = ssl.CERT_REQUIRED
                listener = socket.socket()
                listener.bind(("127.0.0.1", 0))
                listener.listen(2)
                listener.settimeout(3)
                port = listener.getsockname()[1]
                state = {"mqtt": False, "retry": False, "error": None}
                def peer():
                    try:
                        raw, _ = listener.accept()
                        raw.settimeout(3)
                        try:
                            with context.wrap_socket(raw, server_side=True) as secure:
                                header, _ = read_packet(secure)
                                assert header == 0x10
                                state["mqtt"] = True
                                secure.sendall(b"\x20\x02\0\0")
                                assert read_packet(secure)[0] == 0x30
                                assert read_packet(secure) == (0xe0, b"")
                        except ssl.SSLError:
                            raw.close()
                        listener.settimeout(.25)
                        try:
                            extra, _ = listener.accept()
                            state["retry"] = True
                            extra.close()
                        except socket.timeout:
                            pass
                    except Exception as error:
                        state["error"] = type(error).__name__
                    finally:
                        listener.close()
                server = threading.Thread(target=peer, daemon=True)
                server.start()
                result = subprocess.run([str(CLIENT), "pub", "--topic", "tls/fixture",
                    "--message", "payload", "--host", "127.0.0.1", "--port", str(port),
                    "--ca", str(root/(trust+".crt")), "--cert", str(root/"client.crt"),
                    "--key", str(root/"client.key"), "--username", "fixture-client",
                    "--password-file", str(password), "--output", "jsonl",
                    "--reconnect-attempts", "3"], capture_output=True, text=True, timeout=LIMIT)
                server.join(timeout=LIMIT)
                self.assertFalse(server.is_alive())
                self.assertIsNone(state["error"], state)
                self.assertEqual(state["mqtt"], good, state)
                self.assertFalse(state["retry"], state)
                self.assertEqual(result.returncode == 0, good, result.stderr.replace(secret, "<redacted>"))
                self.assertTrue(secret not in result.stderr+result.stdout, "sensitive fixture value leaked (redacted)")
                if good:
                    self.assertEqual(json.loads(result.stdout), {
                        "event": "publish_complete", "qos": 0, "confirmation": "sent"})
                else:
                    self.assertEqual(result.stdout, "")
                    self.assertNotIn("Reconexao", result.stderr)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker", type=Path, required=True)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    args = parser.parse_args()
    CLIENT = args.client.resolve()
    spec = importlib.util.spec_from_file_location("broker_fixtures", args.fixtures.resolve())
    fixtures = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(fixtures)
    fixtures.BROKER = args.broker.resolve()
    assert CLIENT.is_file() and fixtures.BROKER.is_file()
    unittest.main(argv=["C27-contract-integration"], verbosity=2)
