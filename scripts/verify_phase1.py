# SPDX-License-Identifier: MIT
"""C38: real broker/client phase-1 checks; Unix, loopback, disposable state only."""
import argparse
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import sys
import tempfile
import threading
import uuid
import time

TIMEOUT = 8


def wait_line(lines, wanted):
    until = time.monotonic() + TIMEOUT
    while time.monotonic() < until:
        try:
            line = lines.get(timeout=max(.01, until - time.monotonic()))
        except queue.Empty:
            break
        if wanted in line:
            return
    raise AssertionError("subscriber readiness timeout")


class Lab:
    def __init__(self, broker, client, state, limits=None):
        self.client = client
        self.output_directory = state.parent
        self.output_files = {}
        state.mkdir(parents=True, exist_ok=True)
        with socket.socket() as reserve:
            reserve.bind(("127.0.0.1", 0))
            self.port = reserve.getsockname()[1]
        env = {key: value for key, value in os.environ.items() if not key.startswith("MQTT_")}
        env.update(MQTT_MODE="open-lab", MQTT_BIND=f"127.0.0.1:{self.port}",
                   MQTT_STATE_DIR=str(state), RUST_LOG="error")
        env.update(limits or {})
        self.children = []
        self.broker = subprocess.Popen([str(broker)], env=env, stdout=subprocess.DEVNULL,
                                       stderr=subprocess.PIPE)
        self.children.append(self.broker)
        try:
            until = time.monotonic() + TIMEOUT
            while time.monotonic() < until:
                if self.broker.poll() is not None:
                    raise AssertionError("isolated broker exited before readiness")
                try:
                    with socket.create_connection(("127.0.0.1", self.port), timeout=.1):
                        return
                except OSError:
                    time.sleep(.02)
            raise AssertionError("broker readiness timeout")
        except BaseException:
            self.close()
            raise

    def args(self, mode, topic, extra):
        return [str(self.client), mode, "--open-lab", "--port", str(self.port),
                "--topic", topic, "--timeout-seconds", "1", "--keepalive-seconds", "1",
                "--output", "jsonl", *extra]

    def subscriber(self, topic, extra):
        # JSON arrays for 4096 binary bytes exceed some Unix pipe capacities.
        # Write directly to a test-owned file so waiting cannot block on stdout.
        output_path = self.output_directory / f"subscriber-{uuid.uuid4().hex}.jsonl"
        output_file = output_path.open("wb")
        try:
            child = subprocess.Popen(self.args("sub", topic, extra), stdout=output_file,
                                     stderr=subprocess.PIPE, text=True)
        finally:
            output_file.close()
        self.output_files[child] = output_path
        self.children.append(child)
        lines = queue.Queue()
        def read():
            for line in child.stderr:
                lines.put(line)
        threading.Thread(target=read, daemon=True).start()
        wait_line(lines, "Assinatura ativa")
        return child

    def output(self, child):
        assert child.poll() is not None, "subscriber output read before exit"
        return self.output_files[child].read_text(encoding="utf-8")

    def pub(self, topic, data, extra):
        return subprocess.run(self.args("pub", topic, ["--stdin", *extra]), input=data,
                              capture_output=True, timeout=TIMEOUT)

    def close(self):
        for child in reversed(self.children):
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=TIMEOUT)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=TIMEOUT)
            for stream in (child.stdin, child.stdout, child.stderr):
                if stream is not None:
                    stream.close()


def verify(broker, client):
    results = []
    with tempfile.TemporaryDirectory(prefix="xmqr-c38-") as root:
        root = Path(root)
        lab = Lab(broker, client, root / "normal")
        try:
            for qos in range(3):
                topic = f"c38/binary/{qos}"
                subscriber = lab.subscriber(topic, ["--qos", str(qos), "--count", "1"])
                # With timeout1 this must keep waiting for application data while
                # keepalive1 services PINGREQ/PINGRESP beyond broker's 1.5x window.
                time.sleep(3)
                assert subscriber.poll() is None, "application idle wrongly used protocol timeout"
                data = bytes(index % 256 for index in range(4096))
                publication = lab.pub(topic, data, ["--qos", str(qos)])
                assert publication.returncode == 0, "4096-byte publication failed"
                completion = json.loads(publication.stdout)
                assert completion["event"] == "publish_complete"
                subscriber.wait(timeout=TIMEOUT)
                assert subscriber.returncode == 0, "subscriber failed"
                received = json.loads(lab.output(subscriber))
                assert received["topic"] == topic and received["qos"] == qos
                assert bytes(received["payload_bytes"]) == data
                rejected = lab.pub(topic, data + b"x", ["--qos", str(qos)])
                assert rejected.returncode == 2 and b"4096" in rejected.stderr
                assert not rejected.stdout
                results.append(f"payload4096/4097-qos{qos}-keepalive1-timeout1:PASS")
            filters = [f"{index:02}" + "é" * 511 for index in range(63)]
            tail = "a" * 826
            args = []
            for topic in filters:
                args.extend(["--topic", topic])
            subscriber = lab.subscriber(tail, args)
            subscriber.terminate()
            subscriber.wait(timeout=TIMEOUT)
            assert subscriber.returncode == 0, "boundary subscriber shutdown failed"
            rejected = subprocess.run(lab.args("sub", tail + "a", args),
                                      capture_output=True, timeout=TIMEOUT)
            assert rejected.returncode == 2 and b"64 KiB completos" in rejected.stderr
            results.append("SUBSCRIBE-complete65536/65537:PASS")
        finally:
            lab.close()
        limited = Lab(broker, client, root / "limited", {"MQTT_MAX_SUBSCRIPTIONS": "1"})
        try:
            refusal = subprocess.run(limited.args("sub", "c38/one", ["--topic", "c38/two"]),
                                     capture_output=True, timeout=TIMEOUT)
            assert refusal.returncode != 0
            assert b"conexao interrompida" in refusal.stderr
            assert b"quota" not in refusal.stderr
            assert b"Reconexao" not in refusal.stderr
            results.append("quota-subscription-real-transport-cause-unspecified:PASS")
        finally:
            limited.close()
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--broker", type=Path, required=True)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    if os.name != "posix":
        print("BLOCKED: real broker persistence/integration requires Unix", file=sys.stderr)
        return 2
    for binary in (args.broker, args.client):
        if not binary.is_file() or not os.access(binary, os.X_OK):
            parser.error("explicit broker/client executable unavailable")
    results = verify(args.broker.resolve(), args.client.resolve())
    report = {"scope": "C38 isolated Unix broker/client", "results": results,
              "broker": str(args.broker.resolve()), "client": str(args.client.resolve())}
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    for result in results:
        print(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
