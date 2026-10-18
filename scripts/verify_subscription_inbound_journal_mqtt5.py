#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C129: strict, private MQTT 5 receiver journal recovery over real TCP."""
import argparse
import hashlib
import json
from pathlib import Path
import select
import socket
import subprocess
import tempfile
import time


def packet(peer):
    def exact(size):
        value = b""
        while len(value) < size:
            data = peer.recv(size - len(value))
            if not data:
                raise AssertionError("unexpected EOF before protocol completion")
            value += data
        return value
    header = exact(1)[0]
    length, multiplier = 0, 1
    for _ in range(4):
        digit = exact(1)[0]
        length += (digit & 127) * multiplier
        if not digit & 128:
            if length > 65536:
                raise AssertionError("fixture packet exceeds bound")
            return header, exact(length)
        multiplier *= 128
    raise AssertionError("invalid fixture Remaining Length")


def ready(child):
    if not select.select([child.stderr], [], [], 5)[0]:
        raise AssertionError("bounded subscription readiness timeout")
    if b"Assinatura" not in child.stderr.readline():
        raise AssertionError("subscription did not become ready")


def publish(peer, identifier, value):
    body = b"\x00\x08c129/two" + identifier.to_bytes(2, "big") + b"\x00" + value
    peer.sendall(bytes([0x34, len(body)]) + body)


def line(child):
    if not select.select([child.stdout], [], [], 5)[0]:
        raise AssertionError("bounded application output timeout")
    result = json.loads(child.stdout.readline())
    if result.get("event") != "publish":
        raise AssertionError("application publication missing")
    return result


def cleanup(child):
    if child is not None:
        if child.poll() is None:
            child.kill()
        child.communicate(timeout=5)


def recovery(binary, root):
    with socket.socket() as server:
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(2)
        server.settimeout(5)
        port = server.getsockname()[1]
        args = [str(binary), "sub", "--open-lab", "--protocol-version", "5",
                "--host", "127.0.0.1", "--port", str(port), "--topic", "c129/#",
                "--qos", "2", "--client-id", "c129-receiver", "--clean-start", "false",
                "--session-expiry-seconds", "never", "--subscription-state-dir", str(root),
                "--output", "jsonl", "--timeout-seconds", "2"]
        child = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b"\x20\x03\x00\x00\x00")
                header, subscribe = packet(peer)
                assert header == 0x82
                peer.sendall(b"\x90\x04" + subscribe[:2] + b"\x00\x02")
                ready(child)
                publish(peer, 7, b"owned-first")
                first = line(child)
                header, ack = packet(peer)
                assert header == 0x50 and ack[:2] == b"\x00\x07"
                journal_existed_before_pubrec = (root / "subscription-inbound.json").is_file()
                child.kill()
                child.communicate(timeout=5)
            child = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b"\x20\x03\x01\x00\x00")
                ready(child)
                peer.sendall(b"\x62\x02\x00\x07")
                header, ack = packet(peer)
                assert header == 0x70 and ack[:2] == b"\x00\x07"
                if not journal_existed_before_pubrec:
                    raise AssertionError("PUBREC preceded durable inbound journal")
                if select.select([child.stdout], [], [], 0.1)[0]:
                    raise AssertionError("replayed PUBREL duplicated application output")
                # A reused identifier is a new publication after PUBCOMP.
                publish(peer, 7, b"owned-second")
                second = line(child)
                assert first["payload_bytes"] != second["payload_bytes"]
                header, ack = packet(peer)
                assert header == 0x50 and ack[:2] == b"\x00\x07"
                peer.sendall(b"\x62\x02\x00\x07")
                assert packet(peer) == (0x70, b"\x00\x07")
                time.sleep(0.05)
                assert child.poll() is None, "subscription stopped after QoS2 completion"
            return {"status": "PASS", "scenario": "crash_after_pubrec_resume_pubrel_and_id_reuse",
                    "journal_before_pubrec": True, "output_replayed_on_pubrel": False}
        finally:
            cleanup(child)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error("new report path required; failures are preserved")
    started = time.monotonic()
    binary = args.client.resolve(strict=True)
    report = {"client_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "scope": "owned synthetic loopback fixture; SIGKILL is not power loss"}
    with tempfile.TemporaryDirectory(prefix="xmqr-c129-inbound-", dir="/var/tmp") as temporary:
        try:
            report.update(recovery(binary, Path(temporary)))
        except Exception as failure:
            report.update(status="FAIL", error_type=type(failure).__name__,
                          reason=str(failure) if isinstance(failure, AssertionError)
                          else "bounded private fixture failed")
    report["elapsed_seconds"] = round(time.monotonic() - started, 4)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open("x", encoding="utf-8") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    print(json.dumps(report), flush=True)
    if report["status"] != "PASS":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
