#!/usr/bin/env python3
"""C125: disposable one-way TLS bridge against two real XMQR brokers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import select
import signal
import ssl
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
CLIENT_ROOT = Path(__file__).resolve().parents[1]
CLIENT = Path(os.environ.get("XMQR_CLIENT_BIN", CLIENT_ROOT / "target/debug/mqtt-client"))
BRIDGE = Path(os.environ.get("XMQR_BRIDGE_BIN", CLIENT_ROOT / "target/debug/mqtt-bridge"))


def private(path, contents):
    path.write_text(contents)
    path.chmod(0o600)


def wait_markers(child, markers, seconds=12):
    deadline = time.monotonic() + seconds
    seen = bytearray()
    while time.monotonic() < deadline:
        assert child.poll() is None, ("bridge exited", seen)
        ready, _, _ = select.select([child.stderr], [], [], .2)
        if ready:
            seen.extend(os.read(child.stderr.fileno(), 4096))
            assert len(seen) < 16384, "excess bridge diagnostics"
            if all(marker.encode() in seen for marker in markers):
                return
    raise AssertionError(("bridge status timeout", markers, seen))


def ready_subscriber(root, port, topic):
    child = subprocess.Popen(command(root, port, "sub", topic),
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    line = bytearray()
    while not line.endswith(b"\n"):
        ready, _, _ = select.select([child.stdout], [], [], 8)
        assert ready, "subscriber SUBACK timeout"
        byte = os.read(child.stdout.fileno(), 1)
        assert byte, ("subscriber exited before SUBACK", child.stderr.read(1200))
        line.extend(byte)
    assert b"Assinatura ativa" in line, line
    return child


def wait_journal_empty(path, seconds=8, topic=None, allow_pubcomp_pending=False):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        document = json.loads(path.read_text())
        pending = [entry for entry in document["pending"]
                   if not allow_pubcomp_pending or entry.get("phase") != "pubcomp_pending"]
        if not pending or (topic is not None and all(entry["topic"] != topic for entry in pending)):
            return
        time.sleep(.1)
    pending_topics = sorted({entry["topic"] for entry in pending})
    raise AssertionError(("source QoS2 journal did not finish", topic, pending_topics))


def assert_no_new_delivery(root, target_port, topic, seconds=4):
    subscriber = ready_subscriber(root, target_port, topic)
    try:
        try:
            output, _ = subscriber.communicate(timeout=seconds)
            assert not output, output
        except subprocess.TimeoutExpired:
            subscriber.kill()
            output, _ = subscriber.communicate(timeout=5)
            assert not output, output
    finally:
        if subscriber.poll() is None:
            subscriber.kill()
            subscriber.communicate(timeout=5)


def command(root, port, mode, topic, payload=None, *, qos=0, retain=False, retained_only=False,
            ignore_retained=False):
    common = [str(CLIENT), mode, "--port", str(port), "--topic", topic,
              "--username", "device", "--password-file", str(root / "password"),
              "--ca", str(root / "ca.crt"), "--cert", str(root / "client.crt"),
              "--key", str(root / "client.key"), "--client-id", f"c125-{mode}",
              "--timeout-seconds", "5"]
    if mode == "pub":
        common += ["--message", payload, "--qos", str(qos)]
        if retain:
            common += ["--retain", "true"]
    else:
        common += ["--count", "1"]
        if qos == 2:
            common += ["--qos", "2"]
        if retained_only:
            common += ["--retained-only"]
        if ignore_retained:
            common += ["--ignore-retained"]
    return common


def exchange(root, source_port, target_port, suffix, payload, *, expected=True, qos=0):
    subscriber = subprocess.Popen(command(root, target_port, "sub", f"mirror/{suffix}",
                                          qos=qos, ignore_retained=True),
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        ready, _, _ = select.select([subscriber.stdout], [], [], 8)
        assert ready, "subscriber did not report SUBACK"
        assert "Assinatura ativa" in subscriber.stdout.readline()
        published = subprocess.run(command(root, source_port, "pub", f"sensors/{suffix}",
                                           payload, qos=qos), capture_output=True, timeout=10)
        assert published.returncode == 0, published.stderr.decode(errors="replace")
        if expected:
            stdout, stderr = subscriber.communicate(timeout=8)
            assert subscriber.returncode == 0 and payload in stdout, (stdout, stderr)
        else:
            try:
                stdout, stderr = subscriber.communicate(timeout=4)
                assert payload not in stdout, (stdout, stderr)
            except subprocess.TimeoutExpired:
                subscriber.kill()
                stdout, _ = subscriber.communicate(timeout=5)
                assert payload not in stdout
    finally:
        if subscriber.poll() is None:
            subscriber.kill()
            subscriber.communicate(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--broker-root", type=Path, required=True)
    parser.add_argument("--qos1", action="store_true")
    parser.add_argument("--qos2", action="store_true")
    parser.add_argument("--mqtt5-retain", action="store_true",
                        help="QoS 0/1 source subscription with MQTT 5 Retain As Published")
    parser.add_argument("--skip-crash-checkpoint", action="store_true",
                        help="release binary lacks the debug-only crash checkpoint")
    args = parser.parse_args()
    assert not (args.qos1 and args.qos2)
    assert not args.skip_crash_checkpoint or args.qos1 or args.qos2
    qos = 2 if args.qos2 else 1 if args.qos1 else 0
    broker_root = args.broker_root.resolve()
    sys.path.insert(0, str(broker_root / "scripts"))
    from verify_multi_endpoint import free_port, hash_fixture, wait_port
    from verify_wss_mtls import certificates

    broker = broker_root / "target/debug/mqtt-broker"
    admin = broker_root / "target/debug/mqtt-admin"
    assert all(path.is_file() for path in (broker, admin, CLIENT, BRIDGE))
    with tempfile.TemporaryDirectory(prefix="xmqr-c125-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        certificates(root)
        secret = secrets.token_urlsafe(18)
        private(root / "password", secret + "\n")
        fingerprint = hashlib.sha256(
            ssl.PEM_cert_to_DER_cert((root / "client.crt").read_text())).hexdigest()
        digest = hash_fixture(admin, secret)
        private(root / "users.toml",
                f'[[users]]\nusername="device"\npassword_hash="{digest}"\n'
                f'cert_sha256="sha256:{fingerprint}"\n')
        private(root / "source-acl.toml",
                '[[rules]]\nusername="device"\n'
                'publish=["sensors/allowed","sensors/denied","sensors/crash","sensors/session-loss","fault/inflight"]\n'
                'subscribe=["sensors/#","fault/#"]\n')
        private(root / "target-acl.toml",
                '[[rules]]\nusername="device"\npublish=["mirror/allowed","mirror/crash","mirror/session-loss","fault-mirror/inflight"]\n'
                'subscribe=["mirror/allowed","mirror/denied","mirror/crash","mirror/session-loss","fault-mirror/inflight"]\n')
        source_port, target_port = free_port(), free_port()
        assert source_port != target_port
        base = {key: value for key, value in os.environ.items() if not key.startswith("MQTT_")}
        base.update(MQTT_MODE="secure-mtls", MQTT_SERVER_CERT=str(root / "server.crt"),
                    MQTT_SERVER_KEY=str(root / "server.key"),
                    MQTT_CLIENT_CA=str(root / "ca.crt"),
                    MQTT_USERS_FILE=str(root / "users.toml"))
        source_env = dict(base, MQTT_BIND=f"127.0.0.1:{source_port}",
                          MQTT_STATE_DIR=str(root / "source-state"),
                          MQTT_ACL_FILE=str(root / "source-acl.toml"))
        target_env = dict(base, MQTT_BIND=f"127.0.0.1:{target_port}",
                          MQTT_STATE_DIR=str(root / "target-state"),
                          MQTT_ACL_FILE=str(root / "target-acl.toml"))
        def start(environment, port):
            log_path = root / f"broker-{port}.log"
            with log_path.open("ab") as log:
                child = subprocess.Popen([str(broker)], env=environment,
                                         stdout=log, stderr=log)
            try:
                wait_port(child, port)
            except AssertionError as error:
                raise AssertionError(log_path.read_text(errors="replace")[-1200:]) from error
            return child
        def stop(child):
            if child.poll() is None:
                child.send_signal(signal.SIGTERM)
                assert child.wait(timeout=8) == 0
        def endpoint(port, client_id, ca="ca.crt", pwd="password", host="127.0.0.1",
                     broker_id=None):
            return (f'host={json.dumps(host)}\nport={port}\nclient_id="{client_id}"\n'
                    + (f'broker_id="{broker_id}"\n' if broker_id else '')
                    + 'username="device"\n'
                    f'password_file={json.dumps(str(root / pwd))}\n'
                    f'ca={json.dumps(str(root / ca))}\n'
                    f'cert={json.dumps(str(root / "client.crt"))}\n'
                    f'key={json.dumps(str(root / "client.key"))}\n')
        def configuration(path, *, destination_port=target_port, dest_ca="ca.crt",
                          dest_pwd="password", dest_host="127.0.0.1", retries=8):
            header = f'version=1\nmax_retries={retries}\n'
            if args.mqtt5_retain:
                header += 'source_protocol="mqtt5"\n'
            if qos > 0:
                topology = path.with_suffix('.topology.toml')
                private(topology, 'version=1\n[[brokers]]\nid="source"\n'
                        + f'host="127.0.0.1"\nport={source_port}\n'
                        + '[[brokers]]\nid="target"\n'
                        + f'host={json.dumps(dest_host)}\nport={destination_port}\n'
                        + '[[links]]\nid="primary"\nsource_broker="source"\n'
                        + 'destination_broker="target"\nsource_prefix="sensors/"\n'
                        + f'target_prefix="mirror/"\nqos={qos}\n')
                header += f'topology_file={json.dumps(str(topology))}\nlink_id="primary"\n'
            if qos == 2:
                journal_dir = path.with_suffix('.journal')
                journal_dir.mkdir(mode=0o700, exist_ok=True)
                header += f'journal_dir={json.dumps(str(journal_dir))}\n'
            private(path, header + '[source]\n'
                    + endpoint(source_port, "c125-bridge-source",
                               broker_id="source" if qos > 0 else None)
                    + '[destination]\n'
                    + endpoint(destination_port, "c125-bridge-target", dest_ca,
                               dest_pwd, dest_host,
                               broker_id="target" if qos > 0 else None)
                    + f'[route]\nsource_prefix="sensors/"\ntarget_prefix="mirror/"\nqos={qos}\n')
        config = root / "bridge.toml"
        configuration(config)
        if args.mqtt5_retain and qos < 2:
            incompatible = root / "mqtt5-qos2.toml"
            configuration(incompatible)
            private(incompatible, incompatible.read_text().replace(f'qos={qos}', 'qos=3'))
            rejected = subprocess.run([str(BRIDGE), "--config", str(incompatible)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "MQTT 5 source accepted invalid QoS 3"
        same = root / "same.toml"
        configuration(same, destination_port=source_port)
        rejected = subprocess.run([str(BRIDGE), "--config", str(same)],
                                  capture_output=True, timeout=5)
        assert rejected.returncode == 2 and b"invalida" in rejected.stderr.lower()
        if qos > 0:
            alias = root / "alias.toml"
            configuration(alias, destination_port=source_port, dest_host="localhost")
            rejected = subprocess.run([str(BRIDGE), "--config", str(alias)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "localhost/127.0.0.1 alias accepted"
            cycle = root / "cycle.toml"
            configuration(cycle)
            topology = cycle.with_suffix('.topology.toml')
            private(topology, topology.read_text() +
                    '[[links]]\nid="reverse"\nsource_broker="target"\n'
                    'destination_broker="source"\nsource_prefix="mirror/"\n'
                    f'target_prefix="sensors/"\nqos={qos}\n')
            rejected = subprocess.run([str(BRIDGE), "--config", str(cycle)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "topology cycle accepted"
            mismatch = root / "mismatch.toml"
            configuration(mismatch)
            private(mismatch.with_suffix('.topology.toml'),
                    mismatch.with_suffix('.topology.toml').read_text().replace(
                        'target_prefix="mirror/"', 'target_prefix="wrong/"'))
            rejected = subprocess.run([str(BRIDGE), "--config", str(mismatch)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "route mismatch accepted"
            missing_link = root / "missing-link.toml"
            configuration(missing_link)
            private(missing_link, missing_link.read_text().replace(
                'link_id="primary"', 'link_id="absent"'))
            rejected = subprocess.run([str(BRIDGE), "--config", str(missing_link)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "unlisted local link accepted"
            duplicate = root / "duplicate.toml"
            configuration(duplicate)
            duplicate_topology = duplicate.with_suffix('.topology.toml')
            private(duplicate_topology, duplicate_topology.read_text() +
                    f'[[brokers]]\nid="alias"\nhost="localhost"\nport={source_port}\n')
            rejected = subprocess.run([str(BRIDGE), "--config", str(duplicate)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "broker alias with same address accepted"
            unsafe = root / "unsafe.toml"
            configuration(unsafe)
            unsafe.with_suffix('.topology.toml').chmod(0o644)
            rejected = subprocess.run([str(BRIDGE), "--config", str(unsafe)],
                                      capture_output=True, timeout=5)
            assert rejected.returncode == 2, "unsafe topology accepted"
        assert subprocess.run([str(BRIDGE), "--version"], capture_output=True,
                              timeout=5).stdout.startswith(b"mqtt-bridge 0.3.0")
        source = start(source_env, source_port)
        target = start(target_env, target_port)
        bridge_process = None
        try:
            private(root / "wrong-password", "incorrect-fixture-password\n")
            for name, changes in (
                ("wrong-ca", {"dest_ca": "other.crt"}),
                ("wrong-san", {"dest_host": "localhost"}),
                ("wrong-password", {"dest_pwd": "wrong-password"}),
            ):
                wrong = root / f"{name}.toml"
                configuration(wrong, retries=1, **changes)
                negative = subprocess.run([str(BRIDGE), "--config", str(wrong)],
                                          capture_output=True, timeout=15)
                assert negative.returncode != 0, name
                assert secret.encode() not in negative.stderr
            if qos > 0 or args.mqtt5_retain:
                retained = subprocess.run(command(root, source_port, "pub",
                                                  "sensors/allowed", "c136-replay" if args.mqtt5_retain else "c126-retained",
                                                  qos=qos, retain=True), capture_output=True, timeout=10)
                assert retained.returncode == 0, retained.stderr.decode(errors="replace")
            bridge_process = subprocess.Popen([str(BRIDGE), "--config", str(config)],
                                              stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                              bufsize=0)
            wait_markers(bridge_process, ("destino conectado", "origem assinada"))
            if qos > 0 or args.mqtt5_retain:
                retained_sub = subprocess.run(command(root, target_port, "sub", "mirror/allowed",
                                                     retained_only=True), capture_output=True,
                                              timeout=10)
                expected = b"c136-replay" if args.mqtt5_retain else b"c126-retained"
                assert retained_sub.returncode == 0 and expected in retained_sub.stdout, (retained_sub.stdout, retained_sub.stderr)
            exchange(root, source_port, target_port, "allowed", "c125-first", qos=qos)
            if args.mqtt5_retain:
                live = subprocess.run(command(root, source_port, "pub", "sensors/allowed",
                                              "c136-live-retained", qos=qos, retain=True),
                                      capture_output=True, timeout=10)
                assert live.returncode == 0, live.stderr.decode(errors="replace")
                for attempt in range(10):
                    retained_sub = subprocess.run(command(root, target_port, "sub",
                                                          "mirror/allowed", retained_only=True),
                                                  capture_output=True, timeout=10)
                    if retained_sub.returncode == 0 and b"c136-live-retained" in retained_sub.stdout:
                        break
                    time.sleep(.2)
                else:
                    raise AssertionError("live retained MQTT 5 publication not stored on destination")
                exchange(root, source_port, target_port, "allowed", "c136-ordinary", qos=qos)
                retained_sub = subprocess.run(command(root, target_port, "sub",
                                                      "mirror/allowed", retained_only=True),
                                              capture_output=True, timeout=10)
                assert retained_sub.returncode == 0 and b"c136-live-retained" in retained_sub.stdout
            if qos > 0:
                updated = subprocess.run(command(root, source_port, "pub", "sensors/allowed",
                                                 "c126-retained-updated", qos=qos, retain=True),
                                         capture_output=True, timeout=10)
                assert updated.returncode == 0, updated.stderr.decode(errors="replace")
            if qos == 0:
                exchange(root, source_port, target_port, "denied", "c125-denied",
                         expected=False, qos=qos)
            stop(target)
            wait_markers(bridge_process, ("destino desconectado",), seconds=10)
            target = start(target_env, target_port)
            wait_markers(bridge_process, ("destino conectado",), seconds=20)
            exchange(root, source_port, target_port, "allowed", "c125-after-restart", qos=qos)
            if qos == 1:
                bridge_process.terminate()
                bridge_process.communicate(timeout=8)
                process_env = (dict(os.environ, XMQR_BRIDGE_TEST_PAUSE_AFTER_DEST_ACK="1")
                               if not args.skip_crash_checkpoint else None)
                bridge_process = subprocess.Popen([str(BRIDGE), "--config", str(config)],
                                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                  bufsize=0, env=process_env)
                wait_markers(bridge_process, ("destino conectado", "origem assinada"))
                for attempt in range(10):
                    retained_sub = subprocess.run(command(root, target_port, "sub", "mirror/allowed",
                                                         retained_only=True), capture_output=True,
                                                  timeout=10)
                    if retained_sub.returncode == 0 and b"c126-retained-updated" in retained_sub.stdout:
                        break
                    time.sleep(.2)
                else:
                    raise AssertionError(("retained replay missing", retained_sub.stdout,
                                          retained_sub.stderr))
                if not args.skip_crash_checkpoint:
                    subscriber = ready_subscriber(root, target_port, "mirror/crash")
                    sent = subprocess.run(command(root, source_port, "pub", "sensors/crash",
                                                  "c126-crash-redelivery", qos=1),
                                          capture_output=True, timeout=10)
                    assert sent.returncode == 0, sent.stderr.decode(errors="replace")
                    wait_markers(bridge_process, ("checkpoint apos PUBACK do destino",))
                    first, _ = subscriber.communicate(timeout=8)
                    assert b"c126-crash-redelivery" in first, first
                    bridge_process.kill()
                    bridge_process.communicate(timeout=8)
                    subscriber = ready_subscriber(root, target_port, "mirror/crash")
                    bridge_process = subprocess.Popen([str(BRIDGE), "--config", str(config)],
                                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                      bufsize=0)
                    wait_markers(bridge_process, ("origem assinada",), seconds=15)
                    second, _ = subscriber.communicate(timeout=8)
                    assert b"c126-crash-redelivery" in second, second
                exchange(root, source_port, target_port, "denied", "c126-denied",
                         expected=False, qos=qos)
            if qos == 2 and not args.skip_crash_checkpoint:
                bridge_process.terminate()
                bridge_process.communicate(timeout=8)
                journal_path = config.with_suffix('.journal') / 'bridge-qos2.json'
                for checkpoint, marker, payload in (
                    ('XMQR_BRIDGE_TEST_PAUSE_AFTER_DEST_ACK',
                     'checkpoint apos PUBCOMP do destino', 'c134-before-pubrec'),
                    ('XMQR_BRIDGE_TEST_PAUSE_AFTER_SOURCE_PUBREC',
                     'checkpoint apos PUBREC da origem', 'c134-after-pubrec'),
                ):
                    print(f'C134 checkpoint {checkpoint}', flush=True)
                    process_env = dict(os.environ, **{checkpoint: '1'})
                    bridge_process = subprocess.Popen([str(BRIDGE), '--config', str(config)],
                                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                      bufsize=0, env=process_env)
                    wait_markers(bridge_process, ('destino conectado', 'origem assinada'))
                    subscriber = ready_subscriber(root, target_port, 'mirror/crash')
                    sent = subprocess.run(command(root, source_port, 'pub', 'sensors/crash',
                                                  payload, qos=2), capture_output=True, timeout=10)
                    assert sent.returncode == 0, sent.stderr.decode(errors='replace')
                    wait_markers(bridge_process, (marker,))
                    try:
                        delivered, _ = subscriber.communicate(timeout=8)
                    except subprocess.TimeoutExpired as error:
                        raise AssertionError((checkpoint, 'subscriber timeout',
                                              json.loads(journal_path.read_text())['pending'],
                                              bridge_process.poll())) from error
                    assert payload.encode() in delivered, delivered
                    pending = json.loads(journal_path.read_text())['pending']
                    active = [entry for entry in pending if entry['topic'] == 'sensors/crash'
                              and entry.get('phase') != 'pubcomp_pending']
                    assert len(active) == 1, (checkpoint, pending)
                    bridge_process.kill()
                    bridge_process.communicate(timeout=8)
                    bridge_process = subprocess.Popen([str(BRIDGE), '--config', str(config)],
                                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                      bufsize=0)
                    wait_markers(bridge_process, ('origem assinada',), seconds=15)
                    wait_journal_empty(journal_path, topic='sensors/crash',
                                       allow_pubcomp_pending=args.mqtt5_retain)
                    assert_no_new_delivery(root, target_port, 'mirror/crash')
                    bridge_process.terminate()
                    bridge_process.communicate(timeout=8)
                bridge_process = subprocess.Popen([str(BRIDGE), '--config', str(config)],
                                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                  bufsize=0)
                wait_markers(bridge_process, ('origem assinada',), seconds=15)
                exchange(root, source_port, target_port, 'denied', 'c134-denied',
                         expected=False, qos=qos)
                bridge_process.terminate()
                bridge_process.communicate(timeout=8)
                fault_config = root / 'fault.toml'
                configuration(fault_config)
                private(fault_config, fault_config.read_text()
                        .replace('c125-bridge-source', 'c134-fault-source')
                        .replace('c125-bridge-target', 'c134-fault-target')
                        .replace('sensors/', 'fault/')
                        .replace('mirror/', 'fault-mirror/'))
                fault_topology = fault_config.with_suffix('.topology.toml')
                private(fault_topology, fault_topology.read_text()
                        .replace('sensors/', 'fault/')
                        .replace('mirror/', 'fault-mirror/'))
                sent = subprocess.run(command(root, source_port, 'pub', 'fault/inflight',
                                              'c134-destination-restart', qos=2, retain=True),
                                      capture_output=True, timeout=10)
                assert sent.returncode == 0, sent.stderr.decode(errors='replace')
                bridge_process = subprocess.Popen(
                    [str(BRIDGE), '--config', str(fault_config)],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0,
                    env=dict(os.environ, XMQR_BRIDGE_TEST_PAUSE_AFTER_DEST_PUBLISH='1'))
                wait_markers(bridge_process, ('destino conectado', 'origem assinada',
                                              'checkpoint apos PUBLISH do destino'))
                stop(target)
                target = start(target_env, target_port)
                wait_markers(bridge_process, ('destino conectado',), seconds=15)
                fault_journal = fault_config.with_suffix('.journal') / 'bridge-qos2.json'
                wait_journal_empty(fault_journal, seconds=15,
                                   allow_pubcomp_pending=args.mqtt5_retain)
                retained_sub = subprocess.run(command(root, target_port, 'sub',
                                                     'fault-mirror/inflight', retained_only=True),
                                              capture_output=True, timeout=10)
                assert retained_sub.returncode == 0
                assert b'c134-destination-restart' in retained_sub.stdout
                bridge_process.terminate()
                bridge_process.communicate(timeout=8)
                loss_config = root / 'loss.toml'
                configuration(loss_config)
                private(loss_config, loss_config.read_text()
                        .replace('c125-bridge-source', 'c134-loss-source')
                        .replace('c125-bridge-target', 'c134-loss-target'))
                loss_journal_path = loss_config.with_suffix('.journal') / 'bridge-qos2.json'
                bridge_process = subprocess.Popen(
                    [str(BRIDGE), '--config', str(loss_config)],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0,
                    env=dict(os.environ, XMQR_BRIDGE_TEST_PAUSE_AFTER_SOURCE_PUBREC='session-loss'))
                wait_markers(bridge_process, ('destino conectado', 'origem assinada'))
                sent = subprocess.run(command(root, source_port, 'pub', 'sensors/session-loss',
                                              'c134-session-lost', qos=2),
                                      capture_output=True, timeout=10)
                assert sent.returncode == 0, sent.stderr.decode(errors='replace')
                wait_markers(bridge_process, ('checkpoint apos PUBREC da origem',))
                pending_before = json.loads(loss_journal_path.read_text())['pending']
                assert sum(entry['topic'] == 'sensors/session-loss'
                           for entry in pending_before) == 1
                bridge_process.kill()
                bridge_process.communicate(timeout=8)
                stop(source)
                source = start(dict(source_env, MQTT_STATE_DIR=str(root / 'source-empty-state')),
                               source_port)
                refused = subprocess.run([str(BRIDGE), '--config', str(loss_config)],
                                         capture_output=True, timeout=12)
                assert refused.returncode == 1, ('lost source session accepted', refused.stderr)
                expected = (b'sessao MQTT 5 de origem ausente; journal QoS 2 preservado'
                            if args.mqtt5_retain else
                            b'sessao de origem ausente; journal QoS 2 preservado')
                assert expected in refused.stderr
                assert json.loads(loss_journal_path.read_text())['pending'] == pending_before
            if qos == 2 and args.skip_crash_checkpoint:
                exchange(root, source_port, target_port, 'denied', 'c135-release-denied',
                         expected=False, qos=qos)
        except BaseException:
            if bridge_process is not None and bridge_process.poll() is None:
                ready, _, _ = select.select([bridge_process.stderr], [], [], .1)
                if ready:
                    print(os.read(bridge_process.stderr.fileno(), 4096).decode(errors="replace"),
                          file=sys.stderr)
            print((root / f"broker-{target_port}.log").read_text(errors="replace")[-1800:],
                  file=sys.stderr)
            raise
        finally:
            if bridge_process is not None:
                if bridge_process.poll() is None:
                    bridge_process.terminate()
                bridge_process.communicate(timeout=8)
            stop(source)
            stop(target)
    scope = " (release binary, no debug crash checkpoint)" if args.skip_crash_checkpoint else ""
    label = ('C139' if qos == 2 else 'C137' if qos == 1 else 'C136') if args.mqtt5_retain else 'C134' if qos == 2 else 'C125_C126'
    print(f"{label}_BRIDGE_XMQR_QOS{qos}_PASS mapping, ACL, CA/SAN/password, reconnect, same endpoint{scope}")


if __name__ == "__main__":
    main()
