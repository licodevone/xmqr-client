#!/usr/bin/env python3
"""C45: isolated anonymous wire probes; no services, credentials or durable state."""
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time


def child(client, port, extra):
    return subprocess.Popen(
        [str(client), *extra, "--open-lab", "--host", "127.0.0.1", "--port", str(port),
         "--topic", "t", "--client-id", "c45-isolated", "--output", "jsonl"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )


def read_exact(stream, size):
    result = bytearray()
    while len(result) < size:
        data = stream.recv(size - len(result))
        if not data:
            raise EOFError()
        result.extend(data)
    return bytes(result)


def packet(stream):
    header = read_exact(stream, 1)[0]
    length = 0
    for index in range(4):
        byte = read_exact(stream, 1)[0]
        length += (byte & 127) << (7 * index)
        if byte & 128 == 0:
            assert length <= 4096, "fixture packet exceeds bounded buffer"
            return header, read_exact(stream, length)
    raise AssertionError("invalid fixture Remaining Length")


def listener():
    server = socket.socket()
    server.bind(("127.0.0.1", 0))
    server.listen(4)
    server.settimeout(5)
    return server


def no_extra_connection(server):
    server.setblocking(False)
    try:
        connection, _ = server.accept()
    except BlockingIOError:
        return
    connection.close()
    raise AssertionError("unexpected extra connection")


def cleanup(process):
    if process.poll() is None:
        process.kill()
        process.wait(timeout=5)
    for pipe in (process.stdin, process.stdout, process.stderr):
        if pipe:
            pipe.close()


def idle_probe(client):
    with listener() as server:
        process = child(client, server.getsockname()[1],
                        ["sub", "--keepalive-seconds", "1", "--timeout-seconds", "2",
                         "--reconnect-attempts", "1"])
        try:
            stream, _ = server.accept()
            with stream:
                stream.settimeout(3)
                assert packet(stream)[0] == 0x10
                stream.sendall(b"\x20\x02\x00\x00")
                header, body = packet(stream)
                assert header == 0x82 and body[-1] == 0
                stream.sendall(b"\x90\x03" + body[:2] + b"\x00")
                started = time.monotonic()
                deadline = started + 96
                pings = 0
                while True:
                    assert time.monotonic() < deadline, "idle did not expire at 90 seconds"
                    try:
                        header, body = packet(stream)
                    except EOFError:
                        break
                    assert (header, body) == (0xC0, b""), "only keepalive expected in idle probe"
                    stream.sendall(b"\xD0\x00")
                    pings += 1
                elapsed = time.monotonic() - started
            process.stdin.close()
            process.stdin = None
            stdout, stderr = process.communicate(timeout=3)
            assert process.returncode == 1
            assert 88 <= elapsed < 95, "idle duration outside declared bound"
            assert pings >= 80, "keepalive exchange insufficient to demonstrate idle not renewed"
            assert stdout == b"" and b"tempo esgotado" in stderr
            assert b"Reconexao" not in stderr
            no_extra_connection(server)
            return {"name": "idle90_not_renewed_by_pingresp", "status": "PASS",
                    "seconds": round(elapsed, 3), "ping_responses": pings}
        finally:
            cleanup(process)


def input_signal_probe(client, stop_signal):
    with listener() as server:
        process = child(client, server.getsockname()[1],
                        ["pub", "--stdin", "--line-mode", "--reconnect-attempts", "10"])
        try:
            deadline = time.monotonic() + 4
            handler_bit = 1 << (int(stop_signal) - 1)
            while True:
                assert process.poll() is None, "client exited before signal readiness"
                status = Path(f"/proc/{process.pid}/status").read_text()
                caught = next(line.split()[1] for line in status.splitlines() if line.startswith("SigCgt:"))
                if int(caught, 16) & handler_bit:
                    break
                assert time.monotonic() < deadline, "signal handler not ready"
                time.sleep(0.01)
            started = time.monotonic()
            process.send_signal(stop_signal)
            process.wait(timeout=2)
            elapsed = time.monotonic() - started
            assert process.returncode == 0
            process.stdin.close()
            process.stdin = None
            stdout, stderr = process.communicate(timeout=1)
            assert stdout == b"" and b"Cancelado por sinal" in stderr
            no_extra_connection(server)
            return {"name": f"initial_input_cancel_{stop_signal.name}", "status": "PASS",
                    "seconds": round(elapsed, 3), "connections": 0}
        finally:
            cleanup(process)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    assert os.name == "posix" and Path("/proc").is_dir(), "Linux required; unavailable is NOT_RUN"
    client = args.client.resolve(strict=True)
    version = subprocess.check_output([str(client), "--version"], text=True).strip()
    started = time.monotonic()
    report = {"status": "RUNNING", "client_version": version,
              "client_sha256": hashlib.file_digest(client.open("rb"), "sha256").hexdigest(),
              "transport": "anonymous_loopback", "cases": []}
    try:
        for stop_signal in (signal.SIGINT, signal.SIGTERM):
            report["cases"].append(input_signal_probe(client, stop_signal))
        report["cases"].append(idle_probe(client))
        report["status"] = "PASS"
    except Exception as error:
        # Only category is reported; no command/stdout/payload/path values leaked.
        report["status"] = "FAIL"
        report["error_type"] = type(error).__name__
        raise
    finally:
        report["elapsed_seconds"] = round(time.monotonic() - started, 3)
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"status": report["status"], "cases": len(report["cases"])}))


if __name__ == "__main__":
    main()
