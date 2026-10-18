#!/usr/bin/env python3
"""C125: disposable XMQR↔Mosquitto TLS bridge in both directions."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import select
import signal
import socket
import ssl
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
CLIENT = ROOT / "target/debug/mqtt-client"
BRIDGE = ROOT / "target/debug/mqtt-bridge"
from verify_bridge_brokers import private, wait_markers, exchange


def docker(*args):
    return subprocess.run(["docker", *map(str, args)], check=True, capture_output=True,
                          text=True, timeout=20)


def wait_docker(port):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=.2):
                return
        except OSError:
            time.sleep(.1)
    raise AssertionError("Mosquitto TLS listener unavailable")


def case(broker_root, direction, qos):
    sys.path.insert(0, str(broker_root / "scripts"))
    from verify_multi_endpoint import free_port, hash_fixture, wait_port
    from verify_wss_mtls import certificates
    broker = broker_root / "target/debug/mqtt-broker"
    admin = broker_root / "target/debug/mqtt-admin"
    assert all(path.is_file() for path in (broker, admin, CLIENT, BRIDGE))
    with tempfile.TemporaryDirectory(prefix=f"xmqr-c125-{direction}-") as temporary:
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
        xmqr_is_source = direction == "xmqr-to-mosquitto"
        acl = ('[[rules]]\nusername="device"\n'
               'publish=["sensors/allowed"]\nsubscribe=["sensors/#"]\n'
               if xmqr_is_source else
               '[[rules]]\nusername="device"\n'
               'publish=["mirror/allowed"]\nsubscribe=["mirror/allowed"]\n')
        private(root / "xmqr-acl.toml", acl)
        mosq_acl = ('user device\ntopic read mirror/#\ntopic write mirror/allowed\n'
                    if xmqr_is_source else
                    'user device\ntopic read sensors/#\ntopic write sensors/allowed\n')
        private(root / "mosquitto.acl", mosq_acl)
        private(root / "mosquitto.conf",
                'listener 1883 0.0.0.0\nallow_anonymous false\n'
                'log_dest stderr\nlog_type all\nuser root\n'
                'cafile /fixture/ca.crt\ncertfile /fixture/server.crt\n'
                'keyfile /fixture/server.key\nrequire_certificate true\n'
                'use_identity_as_username false\n'
                'password_file /fixture/mosquitto.password\n'
                'acl_file /fixture/mosquitto.acl\n')
        docker("run", "--rm", "--user", "0:0", "--entrypoint", "mosquitto_passwd",
               "-v", f"{root}:/fixture", "eclipse-mosquitto:2.0.22",
               "-b", "-c", "/fixture/mosquitto.password", "device", secret)
        assert (root / "mosquitto.password").is_file(), "Docker did not write password file"
        xmqr_port, mosq_port = free_port(), free_port()
        assert xmqr_port != mosq_port
        source_port = xmqr_port if xmqr_is_source else mosq_port
        target_port = mosq_port if xmqr_is_source else xmqr_port
        env = {key: value for key, value in os.environ.items() if not key.startswith("MQTT_")}
        env.update(MQTT_MODE="secure-mtls", MQTT_BIND=f"127.0.0.1:{xmqr_port}",
                   MQTT_STATE_DIR=str(root / "broker-state"),
                   MQTT_SERVER_CERT=str(root / "server.crt"),
                   MQTT_SERVER_KEY=str(root / "server.key"),
                   MQTT_CLIENT_CA=str(root / "ca.crt"),
                   MQTT_USERS_FILE=str(root / "users.toml"),
                   MQTT_ACL_FILE=str(root / "xmqr-acl.toml"))
        def endpoint(port, identifier, broker_id=None):
            return (f'host="127.0.0.1"\nport={port}\nclient_id="{identifier}"\n'
                    + (f'broker_id="{broker_id}"\n' if broker_id else '')
                    + 'username="device"\n'
                    f'password_file={json.dumps(str(root / "password"))}\n'
                    f'ca={json.dumps(str(root / "ca.crt"))}\n'
                    f'cert={json.dumps(str(root / "client.crt"))}\n'
                    f'key={json.dumps(str(root / "client.key"))}\n')
        header = 'version=1\nmax_retries=8\n'
        if qos == 1:
            private(root / "topology.toml", 'version=1\n[[brokers]]\nid="source"\n'
                    + f'host="127.0.0.1"\nport={source_port}\n'
                    + '[[brokers]]\nid="target"\n'
                    + f'host="127.0.0.1"\nport={target_port}\n'
                    + '[[links]]\nid="primary"\nsource_broker="source"\n'
                    + 'destination_broker="target"\nsource_prefix="sensors/"\n'
                    + 'target_prefix="mirror/"\nqos=1\n')
            header += f'topology_file={json.dumps(str(root / "topology.toml"))}\nlink_id="primary"\n'
        private(root / "bridge.toml", header + '[source]\n'
                + endpoint(source_port, f"c125-{direction}-source",
                           broker_id="source" if qos == 1 else None)
                + '[destination]\n'
                + endpoint(target_port, f"c125-{direction}-target",
                           broker_id="target" if qos == 1 else None)
                + f'[route]\nsource_prefix="sensors/"\ntarget_prefix="mirror/"\nqos={qos}\n')
        container = f"xmqr-c125-{secrets.token_hex(5)}"
        broker_log = root / "broker.log"
        with broker_log.open("wb") as log:
            xmqr = subprocess.Popen([str(broker)], env=env, stdout=log, stderr=log)
        bridge = None
        try:
            wait_port(xmqr, xmqr_port)
            docker("run", "-d", "--name", container, "--user", "0:0",
                   "-p", f"127.0.0.1:{mosq_port}:1883", "-v", f"{root}:/fixture:ro",
                   "eclipse-mosquitto:2.0.22", "mosquitto", "-c", "/fixture/mosquitto.conf")
            wait_docker(mosq_port)
            bridge = subprocess.Popen([str(BRIDGE), "--config", str(root / "bridge.toml")],
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
            wait_markers(bridge, ("destino conectado", "origem assinada"), seconds=20)
            exchange(root, source_port, target_port, "allowed", f"c125-{direction}-first", qos=qos)
            docker("stop", container)
            marker = "destino desconectado" if xmqr_is_source else "origem desconectada"
            wait_markers(bridge, (marker,), seconds=10)
            docker("start", container)
            wait_docker(mosq_port)
            marker = "destino conectado" if xmqr_is_source else "origem assinada"
            wait_markers(bridge, (marker,), seconds=20)
            exchange(root, source_port, target_port, "allowed", f"c125-{direction}-restart", qos=qos)
        except BaseException:
            if bridge is not None and bridge.poll() is None:
                ready, _, _ = select.select([bridge.stderr], [], [], .1)
                if ready:
                    print(os.read(bridge.stderr.fileno(), 4096).decode(errors="replace"),
                          file=sys.stderr)
            print(broker_log.read_text(errors="replace")[-1200:], file=sys.stderr)
            try:
                logs = docker("logs", container)
                print((logs.stdout + logs.stderr)[-1800:], file=sys.stderr)
            except subprocess.CalledProcessError:
                pass
            raise
        finally:
            if bridge is not None:
                if bridge.poll() is None: bridge.terminate()
                bridge.communicate(timeout=8)
            if xmqr.poll() is None:
                xmqr.send_signal(signal.SIGTERM)
                xmqr.wait(timeout=8)
            subprocess.run(["docker", "rm", "-f", container], capture_output=True, timeout=20)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--broker-root", type=Path, required=True)
    parser.add_argument("--qos1", action="store_true")
    args = parser.parse_args()
    qos = 1 if args.qos1 else 0
    for direction in ("xmqr-to-mosquitto", "mosquitto-to-xmqr"):
        case(args.broker_root.resolve(), direction, qos)
        print(f"C125_C126_BRIDGE_{direction.upper().replace('-', '_')}_QOS{qos}_PASS")


if __name__ == "__main__":
    main()
