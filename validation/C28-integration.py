# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""C28 isolated broker and native Linux stdin cancellation contracts.
Reuses only declared C27 test certificate/packet helpers and broker fixtures.
"""
import argparse, importlib.util, json, secrets, signal, socket, ssl, subprocess
import os, select, tempfile, threading, time, unittest
from pathlib import Path

def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module

def cleanup_process(process):
    if process.poll() is None:
        process.kill()
    process.wait(timeout=3)
    for pipe in (process.stdin, process.stdout, process.stderr):
        if pipe is not None and not pipe.closed:
            pipe.close()

def bounded_line(pipe):
    deadline = time.monotonic()+4
    data = bytearray()
    while len(data) < 4096:
        if not select.select([pipe], [], [], max(0, deadline-time.monotonic()))[0]:
            raise AssertionError("client stdout deadline")
        byte = os.read(pipe.fileno(), 1)
        if not byte:
            raise AssertionError("client stdout closed before complete record")
        data.extend(byte)
        if byte == b"\n":
            return bytes(data)
    raise AssertionError("client stdout record limit")

class InputContracts(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="c28-isolated-")
        self.broker = fixtures.Broker(self.directory.name)
        self.addCleanup(self.directory.cleanup); self.addCleanup(self.broker.stop)
        self.broker.start()

    def cli(self, *args):
        return [str(CLIENT), "pub", "--open-lab", "--host", "127.0.0.1",
                "--port", str(self.broker.port), "--topic", "c28/input", "--output", "jsonl", *args]

    def test_real_broker_binary_and_lines_all_qos(self):
        for qos in (0, 1, 2):
            observer = fixtures.WireClient(self.broker, "c28-observe-"+str(qos))
            try:
                observer.subscribe([("c28/input", qos)])
                for lines, payloads, input_bytes in (
                    (False, [bytes(range(256))], bytes(range(256))),
                    (True, [b"\xff\0", b"", b"last\r"], b"\xff\0\r\n\nlast\r")):
                    process = subprocess.Popen(self.cli("--stdin", "--qos", str(qos),
                        *(["--line-mode"] if lines else [])), stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                    self.addCleanup(cleanup_process, process)
                    process.stdin.write(input_bytes); process.stdin.close()
                    for payload in payloads:
                        header, body = observer.receive()
                        self.assertEqual(header >> 4, 3)
                        topic_size = int.from_bytes(body[:2], "big")
                        offset = 2+topic_size
                        received_qos = (header >> 1) & 3
                        identifier = body[offset:offset+2] if received_qos else b""
                        self.assertEqual(body[offset+(2 if received_qos else 0):], payload)
                        if received_qos == 1: observer.send(0x40, identifier)
                        if received_qos == 2:
                            observer.send(0x50, identifier)
                            self.assertEqual(observer.receive(), (0x62, identifier))
                            observer.send(0x70, identifier)
                    process.wait(timeout=8)
                    self.assertEqual(process.returncode, 0, process.stderr.read().decode())
                    self.assertEqual(len(process.stdout.read().splitlines()), len(payloads))
            finally:
                observer.close()

    def test_sigint_blocked_stdin_before_connect_and_between_lines(self):
        for lines in (False, True):
            observer = fixtures.WireClient(self.broker, "c28-cancel-observer-"+str(lines))
            try:
                observer.subscribe([("c28/will", 1)])
                process = subprocess.Popen(self.cli("--stdin", "--qos", "1",
                    "--will-topic", "c28/will", "--will-message", "unexpected",
                    *(["--line-mode"] if lines else [])), stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                self.addCleanup(cleanup_process, process)
                try:
                    if lines:
                        process.stdin.write(b"first\n"); process.stdin.flush()
                        self.assertEqual(json.loads(bounded_line(process.stdout))["event"], "publish_complete")
                    else:
                        time.sleep(.2)
                    started = time.monotonic(); process.send_signal(signal.SIGINT)
                    self.assertEqual(process.wait(timeout=2), 0)
                    self.assertLess(time.monotonic()-started, 2)
                    self.assertIn(b"Cancelado", process.stderr.read())
                    observer.ping()  # Any unwanted Will would precede PINGRESP and fail this.
                finally:
                    process.stdin.close()
                    if process.poll() is None: process.kill(); process.wait(timeout=2)
            finally: observer.close()

    def test_disconnect_between_lines_never_reconnects(self):
        listener = socket.socket(); listener.bind(("127.0.0.1", 0)); listener.listen(2)
        listener.settimeout(3); port = listener.getsockname()[1]
        process = subprocess.Popen([str(CLIENT), "pub", "--open-lab", "--port", str(port),
            "--topic", "t", "--stdin", "--line-mode", "--qos", "1",
            "--output", "jsonl", "--reconnect-attempts", "10"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.addCleanup(cleanup_process, process)
        try:
            process.stdin.write(b"first\n"); process.stdin.flush()
            stream, _ = listener.accept(); stream.settimeout(3)
            self.assertEqual(c27.read_packet(stream)[0], 0x10); stream.sendall(b"\x20\x02\0\0")
            header, body = c27.read_packet(stream); self.assertEqual(header, 0x32)
            stream.sendall(b"\x40\x02"+body[3:5])
            self.assertEqual(json.loads(bounded_line(process.stdout))["event"], "publish_complete")
            stream.close(); self.assertEqual(process.wait(timeout=2), 1)
            listener.settimeout(.2)
            with self.assertRaises(socket.timeout): listener.accept()
            self.assertNotIn(b"Reconexao", process.stderr.read())
        finally:
            process.stdin.close(); listener.close()
            if process.poll() is None: process.kill(); process.wait(timeout=2)

    def test_non_regular_files_fail_without_connect_or_echo(self):
        native = tempfile.TemporaryDirectory(prefix="c28-file-policy-", dir="/tmp")
        self.addCleanup(native.cleanup)
        root = Path(native.name)
        fifo = root/"sensitive-fifo-path"
        os.mkfifo(fifo)
        for path in (fifo, root, Path("/dev/null"), root/"private-missing"):
            result = subprocess.run(self.cli("--message-file", str(path)),
                capture_output=True, timeout=2)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stdout, b"")
            self.assertNotIn(str(path).encode(), result.stderr)

    def test_secure_stdin_keeps_secret_separate_and_binary_exact(self):
        with tempfile.TemporaryDirectory(prefix="c28-tls-", dir="/tmp") as directory:
            root = Path(directory); root.chmod(0o700); c27.certificates(root)
            secret = secrets.token_hex(16); password = root/"password"
            password.write_text(secret); password.chmod(0o600)
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.load_cert_chain(root/"server.crt", root/"server.key")
            context.load_verify_locations(root/"ca.crt"); context.verify_mode = ssl.CERT_REQUIRED
            listener = socket.socket(); listener.bind(("127.0.0.1", 0)); listener.listen(1)
            listener.settimeout(3); state = []
            def peer():
                try:
                    raw, _ = listener.accept(); raw.settimeout(3)
                    with context.wrap_socket(raw, server_side=True) as stream:
                        header, body = c27.read_packet(stream)
                        state.append(header == 0x10 and secret.encode() in body)
                        stream.sendall(b"\x20\x02\0\0")
                        header, body = c27.read_packet(stream)
                        state.append(header == 0x30 and body[3:] == b"\xff\0\r\n")
                        state.append(c27.read_packet(stream) == (0xe0, b""))
                except Exception as error: state.append(type(error).__name__)
                finally: listener.close()
            server = threading.Thread(target=peer, daemon=True); server.start()
            result = subprocess.run([str(CLIENT), "pub", "--topic", "t", "--stdin",
                "--host", "127.0.0.1", "--port", str(listener.getsockname()[1]),
                "--ca", str(root/"ca.crt"), "--cert", str(root/"client.crt"),
                "--key", str(root/"client.key"), "--username", "fixture",
                "--password-file", str(password), "--output", "jsonl"],
                input=b"\xff\0\r\n", capture_output=True, timeout=8)
            server.join(timeout=4); self.assertFalse(server.is_alive()); self.assertEqual(state, [True]*3)
            self.assertEqual(result.returncode, 0)
            self.assertTrue(secret.encode() not in result.stdout+result.stderr, "secret leak (redacted)")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(); parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--broker", type=Path, required=True); parser.add_argument("--fixtures", type=Path, required=True)
    args = parser.parse_args(); CLIENT = args.client.resolve()
    fixtures = load("broker_fixtures", args.fixtures.resolve()); fixtures.BROKER = args.broker.resolve()
    c27 = load("c27_helpers", Path(__file__).with_name("C27-integration.py"))
    unittest.main(argv=["C28-integration"], verbosity=2)
