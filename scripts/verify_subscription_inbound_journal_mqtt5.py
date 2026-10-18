#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C129: strict, private MQTT 5 receiver journal recovery over real TCP."""
import argparse
import fcntl
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


def publication(identifier, value, qos=2, duplicate=False):
    body = b"\x00\x08c129/two" + identifier.to_bytes(2, "big") + b"\x00" + value
    return bytes([0x30 | (qos << 1) | (8 if duplicate else 0), len(body)]) + body


def publish(peer, identifier, value, qos=2, duplicate=False):
    peer.sendall(publication(identifier, value, qos, duplicate))


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


def continuous(binary, _root):
    with socket.socket() as server:
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        server.settimeout(5)
        child = subprocess.Popen([str(binary), "sub", "--open-lab", "--protocol-version", "5",
                                  "--port", str(server.getsockname()[1]), "--topic", "c129/#",
                                  "--qos", "2", "--output", "jsonl"],
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b"\x20\x03\x00\x00\x00")
                header, subscribe = packet(peer)
                assert header == 0x82
                peer.sendall(b"\x90\x04" + subscribe[:2] + b"\x00\x02")
                ready(child)
                for identifier in (7, 8):
                    publish(peer, identifier, b"owned-continuous")
                    line(child)
                    assert packet(peer)[0] == 0x50
                    peer.sendall(b"\x62\x02" + identifier.to_bytes(2, "big"))
                    assert packet(peer) == (0x70, identifier.to_bytes(2, "big"))
                    time.sleep(0.05)
                    assert child.poll() is None, "continuous default stopped after QoS2 completion"
            return {"status": "PASS", "scenario": "default_continuous_two_qos2"}
        finally:
            cleanup(child)


def listener():
    server = socket.socket()
    server.bind(("127.0.0.1", 0))
    server.listen(2)
    server.settimeout(5)
    return server


def arguments(binary, root, server, extra=()):
    return [str(binary), "sub", "--open-lab", "--protocol-version", "5",
            "--host", "127.0.0.1", "--port", str(server.getsockname()[1]),
            "--topic", "c129/#", "--qos", "2", "--client-id", "c129-receiver",
            "--clean-start", "false", "--session-expiry-seconds", "never",
            "--subscription-state-dir", str(root), "--output", "jsonl",
            "--timeout-seconds", "2", *extra]


def activate(binary, root, server, extra=(), stdout=subprocess.PIPE):
    child = subprocess.Popen(arguments(binary, root, server, extra), stdout=stdout,
                             stderr=subprocess.PIPE, bufsize=0)
    try:
        peer = server.accept()[0]
        peer.settimeout(5)
        assert packet(peer)[0] == 0x10
        peer.sendall(b"\x20\x03\x00\x00\x00")
        header, subscribe = packet(peer)
        assert header == 0x82
        peer.sendall(b"\x90\x04" + subscribe[:2] + b"\x00\x02")
        ready(child)
        return child, peer
    except BaseException:
        cleanup(child)
        raise


def seed(binary, root, server):
    child, peer = activate(binary, root, server)
    try:
        publish(peer, 7, b"owned-seed")
        line(child)
        assert packet(peer) == (0x50, b"\x00\x07")
    finally:
        cleanup(child)
        peer.close()
    return (root / "subscription-inbound.json").read_bytes()


def refusal(binary, root, kind):
    with listener() as server:
        before = seed(binary, root, server)
        path = root / "subscription-inbound.json"
        args = arguments(binary, root, server)
        held_lock = None
        if kind == "corrupt":
            path.write_bytes(b"{}")
            before = path.read_bytes()
        elif kind == "permission":
            path.chmod(0o644)
        elif kind == "identity":
            args[args.index("--topic") + 1] = "different/#"
        elif kind == "lock":
            held_lock = (root / "outbox.lock").open("rb")
            fcntl.flock(held_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        child = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            if kind == "session_absent":
                with server.accept()[0] as peer:
                    peer.settimeout(5)
                    assert packet(peer)[0] == 0x10
                    peer.sendall(b"\x20\x03\x00\x00\x00")
                    assert peer.recv(1) == b"", "state replayed/ACKed without Session Present"
            stdout, _ = child.communicate(timeout=5)
            assert child.returncode != 0 and not stdout
            assert path.read_bytes() == before, "refusal replaced durable inbound state"
            if kind != "session_absent":
                server.settimeout(0.1)
                try:
                    unexpected, _ = server.accept()
                except socket.timeout:
                    pass
                else:
                    unexpected.close()
                    raise AssertionError("invalid durable state opened a network connection")
            return dict(status="PASS", scenario="refuse_" + kind,
                        no_replay_or_ack=True, state_preserved=True)
        finally:
            cleanup(child)
            if held_lock is not None:
                held_lock.close()


def output_failure(binary, root, qos):
    with listener() as server, open("/dev/full", "wb") as full:
        child, peer = activate(binary, root, server, stdout=full)
        try:
            publish(peer, 7, b"owned-output-failure", qos)
            child.communicate(timeout=5)
            assert child.returncode != 0
            assert peer.recv(1) == b"", "ACK preceded successful output/commit"
            document = json.loads((root / "subscription-inbound.json").read_bytes())
            assert document["entries"][0]["stage"] == "Received"
            return dict(status="PASS", scenario=f"qos{qos}_output_failure_no_ack",
                        recoverable_received_record=True)
        finally:
            cleanup(child)
            peer.close()


def received_recovery(binary, root):
    with listener() as server, open("/dev/full", "wb") as full:
        child, peer = activate(binary, root, server, stdout=full)
        try:
            publish(peer, 7, b"owned-received-recovery")
            child.communicate(timeout=5)
            assert child.returncode != 0 and peer.recv(1) == b""
        finally:
            cleanup(child)
            peer.close()
        child = subprocess.Popen(arguments(binary, root, server), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, bufsize=0)
        try:
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b"\x20\x03\x01\x00\x00")
                ready(child)
                publish(peer, 7, b"owned-received-recovery", duplicate=True)
                line(child)
                assert packet(peer) == (0x50, b"\x00\x07")
                peer.sendall(b"\x62\x02\x00\x07")
                assert packet(peer) == (0x70, b"\x00\x07")
                assert child.poll() is None
                return dict(status="PASS", scenario="received_after_output_error_replays_on_sp1")
        finally:
            cleanup(child)


def conflicting_qos1_identifier(binary, root):
    with listener() as server, open("/dev/full", "wb") as full:
        child, peer = activate(binary, root, server, stdout=full)
        try:
            publish(peer, 7, b"owned-original", qos=1)
            child.communicate(timeout=5)
            assert child.returncode != 0 and peer.recv(1) == b""
        finally:
            cleanup(child)
            peer.close()
        before = (root / "subscription-inbound.json").read_bytes()
        child = subprocess.Popen(arguments(binary, root, server), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, bufsize=0)
        try:
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b"\x20\x03\x01\x00\x00")
                ready(child)
                publish(peer, 7, b"owned-conflict", qos=1)
                stdout, _ = child.communicate(timeout=5)
                assert child.returncode != 0 and not stdout and peer.recv(1) == b""
                assert (root / "subscription-inbound.json").read_bytes() == before
                return dict(status="PASS", scenario="qos1_conflicting_live_identifier_refused",
                            state_preserved=True, no_replay_or_ack=True)
        finally:
            cleanup(child)


def deadline(binary, root):
    with listener() as server:
        args = arguments(binary, root, server)
        args[args.index("--timeout-seconds") + 1] = "1"
        child = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
        try:
            with server.accept()[0] as peer:
                peer.settimeout(5)
                assert packet(peer)[0] == 0x10
                peer.sendall(b"\x20\x03\x00\x00\x00")
                _, subscribe = packet(peer)
                peer.sendall(b"\x90\x04" + subscribe[:2] + b"\x00\x02")
                ready(child)
                publish(peer, 7, b"owned-deadline")
                line(child)
                assert packet(peer) == (0x50, b"\x00\x07")
                started = time.monotonic()
                repeats = 0
                while child.poll() is None and time.monotonic() - started < 3:
                    time.sleep(0.2)
                    try:
                        publish(peer, 7, b"owned-deadline", duplicate=True)
                        header, _ = packet(peer)
                    except (AssertionError, BrokenPipeError, ConnectionResetError):
                        break
                    assert header == 0x50
                    repeats += 1
                stdout, _ = child.communicate(timeout=3)
                elapsed = time.monotonic() - started
                assert repeats >= 2 and elapsed < 2.5 and child.returncode != 0
                assert not stdout, "DUP PUBLISH repeated confirmed application output"
                document = json.loads((root / "subscription-inbound.json").read_bytes())
                assert document["entries"][0]["stage"] == "AwaitPubRel"
                return dict(status="PASS", scenario="absolute_pubrel_deadline",
                            repeats=repeats, elapsed_seconds=round(elapsed, 4), state_preserved=True)
        finally:
            cleanup(child)


def pubrel_commit_failure(binary, root):
    with listener() as server:
        child, peer = activate(binary, root, server)
        try:
            publish(peer, 7, b"owned-pubrel-failure")
            line(child)
            assert packet(peer) == (0x50, b"\x00\x07")
            before = (root / "subscription-inbound.json").read_bytes()
            (root / "subscription-inbound.tmp").mkdir()
            peer.sendall(b"\x62\x02\x00\x07")
            child.communicate(timeout=5)
            assert child.returncode != 0
            assert peer.recv(1) == b"", "PUBCOMP preceded durable PUBREL transition"
            assert (root / "subscription-inbound.json").read_bytes() == before
            return dict(status="PASS", scenario="pubrel_commit_failure_no_pubcomp",
                        state_preserved=True)
        finally:
            cleanup(child)
            peer.close()


def unknown_pubrel(binary, root):
    with listener() as server:
        child, peer = activate(binary, root, server)
        try:
            peer.sendall(b"\x62\x02\x00\x09")
            header, ack = packet(peer)
            assert header == 0x70 and ack[:3] == b"\x00\x09\x92"
            publish(peer, 7, b"owned-after-unknown")
            line(child)
            assert packet(peer) == (0x50, b"\x00\x07")
            peer.sendall(b"\x62\x02\x00\x07")
            assert packet(peer) == (0x70, b"\x00\x07")
            peer.sendall(b"\x62\x02\x00\x07")
            header, ack = packet(peer)
            assert header == 0x70 and ack[:3] == b"\x00\x07\x92"
            assert child.poll() is None
            return dict(status="PASS", scenario="unknown_and_repeated_pubrel_0x92")
        finally:
            cleanup(child)
            peer.close()


def batch(binary, root):
    with listener() as server:
        child, peer = activate(binary, root, server)
        try:
            ids = range(1, 33)
            peer.sendall(b"".join(publication(identifier, b"owned-batch") for identifier in ids))
            acknowledgements = [packet(peer) for _ in ids]
            assert acknowledgements == [(0x50, identifier.to_bytes(2, "big")) for identifier in ids]
            peer.sendall(b"".join(b"\x62\x02" + identifier.to_bytes(2, "big") for identifier in ids))
            assert [packet(peer) for _ in ids] == [
                (0x70, identifier.to_bytes(2, "big")) for identifier in ids]
            assert child.poll() is None
            child.kill()
            stdout, _ = child.communicate(timeout=5)
            assert len(stdout.splitlines()) == len(ids)
            return dict(status="PASS", scenario="batch_32_no_ack_channel_deadlock", publications=len(ids))
        finally:
            cleanup(child)
            peer.close()


def count_rejected(binary, root):
    with listener() as server:
        child = subprocess.Popen(arguments(binary, root, server, ("--count", "1")),
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            stdout, stderr = child.communicate(timeout=5)
            assert child.returncode != 0 and not stdout and b"sem --count" in stderr
            assert not (root / "subscription.json").exists()
            server.settimeout(0.1)
            try:
                unexpected, _ = server.accept()
            except socket.timeout:
                pass
            else:
                unexpected.close()
                raise AssertionError("unsupported count opened a network connection")
            return dict(status="PASS", scenario="durable_count_statically_rejected_without_network")
        finally:
            cleanup(child)


def cancelled_or_idle(binary, root, cancel):
    with listener() as server:
        extra = () if cancel else ("--total-timeout-seconds", "1")
        child, peer = activate(binary, root, server, extra)
        try:
            if cancel:
                child.terminate()
            stdout, _ = child.communicate(timeout=3)
            assert not stdout
            assert (child.returncode == 0) == cancel
            assert json.loads((root / "subscription.json").read_bytes())["pending"]["stage"] == "Active"
            return dict(status="PASS", scenario="cancel_preserves_idle_manifest" if cancel
                        else "total_idle_deadline_preserves_manifest")
        finally:
            cleanup(child)
            peer.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    scenarios = [
        ("recovery", recovery), ("continuous", continuous),
        *[(name, lambda binary, root, kind=name: refusal(binary, root, kind))
          for name in ("session_absent", "corrupt", "permission", "identity", "lock")],
        *[(f"output_qos{qos}", lambda binary, root, qos=qos: output_failure(binary, root, qos))
          for qos in (1, 2)],
        ("received_recovery", received_recovery),
        ("qos1_conflict", conflicting_qos1_identifier),
        ("deadline", deadline), ("pubrel_commit_failure", pubrel_commit_failure),
        ("unknown_pubrel", unknown_pubrel),
        ("batch32", batch), ("count", count_rejected),
        ("cancel", lambda binary, root: cancelled_or_idle(binary, root, True)),
        ("idle", lambda binary, root: cancelled_or_idle(binary, root, False)),
    ]
    parser.add_argument("--case", choices=("all", *[name for name, _ in scenarios]), default="all")
    args = parser.parse_args()
    if args.report.exists():
        parser.error("new report path required; failures are preserved")
    started = time.monotonic()
    binary = args.client.resolve(strict=True)
    report = {"client_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "scope": "owned synthetic loopback fixture; SIGKILL is not power loss"}
    report["scenarios"] = []
    for name, scenario in scenarios:
        if args.case not in ("all", name):
            continue
        with tempfile.TemporaryDirectory(prefix="xmqr-c129-inbound-", dir="/var/tmp") as temporary:
            try:
                result = scenario(binary, Path(temporary))
            except Exception as failure:
                result = dict(status="FAIL", scenario=name, error_type=type(failure).__name__,
                              reason=str(failure) if isinstance(failure, AssertionError)
                              else "bounded private fixture failed")
            report["scenarios"].append(result)
    report["status"] = "PASS" if all(item["status"] == "PASS" for item in report["scenarios"]) else "FAIL"
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
