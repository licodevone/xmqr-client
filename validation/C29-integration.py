# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Exercise repeated --topic subscriptions against an isolated XMQR broker."""
import argparse
import json
import os
import selectors
import socket
import subprocess
import tempfile
import time
import unittest
from pathlib import Path


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def wait_for_port(port, process, deadline):
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise AssertionError("isolated broker exited during startup")
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                return
        except OSError:
            time.sleep(0.03)
    raise AssertionError("isolated broker startup timeout")


def wait_for_text(stream, needle, timeout=5):
    selector = selectors.DefaultSelector()
    selector.register(stream, selectors.EVENT_READ)
    deadline = time.monotonic() + timeout
    content = bytearray()
    try:
        while time.monotonic() < deadline:
            if not selector.select(max(0, deadline - time.monotonic())):
                break
            byte = os.read(stream.fileno(), 1)
            if not byte:
                break
            content.extend(byte)
            if needle in content:
                return
        raise AssertionError("client did not confirm all subscriptions")
    finally:
        selector.close()


def read_json_lines(stream, count, timeout=8):
    selector = selectors.DefaultSelector()
    selector.register(stream, selectors.EVENT_READ)
    deadline = time.monotonic() + timeout
    lines = bytearray()
    records = []
    try:
        while len(records) < count and time.monotonic() < deadline:
            if not selector.select(max(0, deadline - time.monotonic())):
                break
            byte = os.read(stream.fileno(), 1)
            if not byte:
                break
            lines.extend(byte)
            if byte == b"\n":
                records.append(json.loads(lines))
                lines.clear()
        if len(records) != count:
            raise AssertionError(f"expected {count} message records, received {len(records)}")
        return records
    finally:
        selector.close()


def stop(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)
    for pipe in (process.stdin, process.stdout, process.stderr):
        if pipe is not None and not pipe.closed:
            pipe.close()


class MultipleTopicIntegration(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="c29-multi-topic-")
        self.addCleanup(self.temp.cleanup)
        self.state = Path(self.temp.name) / "state dir"
        self.state.mkdir(mode=0o700)
        self.port = free_port()
        env = os.environ.copy()
        env.update({
            "MQTT_MODE": "open-lab",
            "MQTT_BIND": f"127.0.0.1:{self.port}",
            "MQTT_STATE_DIR": str(self.state),
            "RUST_LOG": "warn",
        })
        self.broker = subprocess.Popen(
            [str(BROKER)], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE
        )
        self.addCleanup(stop, self.broker)
        wait_for_port(self.port, self.broker, time.monotonic() + 5)

    def client_base(self, command, client_id):
        return [str(CLIENT), command, "--open-lab", "--host", "127.0.0.1",
                "--port", str(self.port), "--qos", "1", "--client-id", client_id]

    def publish(self, topic, message, retain=False):
        result = subprocess.run(
            self.client_base("pub", "c29-pub-" + str(time.monotonic_ns()))
            + ["--topic", topic, "--message", message, "--retain", str(retain).lower(),
               "--output", "jsonl"],
            capture_output=True, timeout=5,
        )
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))

    def test_batched_filters_disjoint_overlap_retained_and_dollar_topics(self):
        self.publish("area/a/retained", "stored", retain=True)
        subscriber = subprocess.Popen(
            self.client_base("sub", "c29-sub")
            + ["--topic", "area/a/#", "--topic", "area/b/#",
               "--topic", "overlap/#", "--topic", "overlap/+/state",
               "--topic", "$custom/#", "--count", "5", "--output", "jsonl"],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        self.addCleanup(stop, subscriber)
        wait_for_text(subscriber.stderr, b"Assinatura ativa")
        self.publish("area/a/live", "a")
        self.publish("area/b/live", "b")
        self.publish("overlap/x/state", "once")
        self.publish("$custom/value", "explicit-dollar")

        records = read_json_lines(subscriber.stdout, 5)
        self.assertEqual(subscriber.wait(timeout=3), 0, subscriber.stderr.read().decode())
        by_topic = {record["topic"]: record for record in records}
        self.assertEqual(set(by_topic), {
            "area/a/retained", "area/a/live", "area/b/live",
            "overlap/x/state", "$custom/value",
        })
        self.assertEqual(len(records), len(by_topic), "overlapping filters duplicated a delivery")
        self.assertTrue(by_topic["area/a/retained"]["retain"])
        self.assertFalse(by_topic["$custom/value"]["retain"])
        self.assertEqual(by_topic["$custom/value"]["payload_bytes"], list(b"explicit-dollar"))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--broker", type=Path, required=True)
    args = parser.parse_args()
    CLIENT = args.client.resolve()
    BROKER = args.broker.resolve()
    unittest.main(argv=["C29-integration"], verbosity=2)
