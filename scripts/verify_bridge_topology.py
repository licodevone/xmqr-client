#!/usr/bin/env python3
"""C127: two bridges, three real XMQR brokers, one acyclic TLS/mTLS topology."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import signal
import ssl
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
from verify_bridge_brokers import BRIDGE, CLIENT, command, private, ready_subscriber, wait_markers


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--broker-root", type=Path, required=True)
    args = parser.parse_args()
    broker_root = args.broker_root.resolve()
    sys.path.insert(0, str(broker_root / "scripts"))
    from verify_multi_endpoint import free_port, hash_fixture, wait_port
    from verify_wss_mtls import certificates

    broker = broker_root / "target/debug/mqtt-broker"
    admin = broker_root / "target/debug/mqtt-admin"
    assert all(path.is_file() for path in (broker, admin, CLIENT, BRIDGE))
    with tempfile.TemporaryDirectory(prefix="xmqr-c127-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        certificates(root)
        secret = secrets.token_urlsafe(18)
        private(root / "password", secret + "\n")
        fingerprint = hashlib.sha256(ssl.PEM_cert_to_DER_cert(
            (root / "client.crt").read_text())).hexdigest()
        digest = hash_fixture(admin, secret)
        private(root / "users.toml",
                f'[[users]]\nusername="device"\npassword_hash="{digest}"\n'
                f'cert_sha256="sha256:{fingerprint}"\n')
        ports = [free_port() for _ in range(3)]
        assert len(set(ports)) == 3
        topics = ("sensors", "mirror", "archive")
        for index, prefix in enumerate(topics):
            private(root / f"acl-{index}.toml",
                    f'[[rules]]\nusername="device"\n'
                    f'publish=["{prefix}/allowed"]\n'
                    f'subscribe=["{prefix}/#","{prefix}/allowed"]\n')

        brokers = ''.join(f'[[brokers]]\nid="{name}"\nhost="127.0.0.1"\nport={port}\n'
                          for name, port in zip(("a", "b", "c"), ports))
        links = ('[[links]]\nid="a-b"\nsource_broker="a"\n'
                 'destination_broker="b"\nsource_prefix="sensors/"\n'
                 'target_prefix="mirror/"\nqos=1\n'
                 '[[links]]\nid="b-c"\nsource_broker="b"\n'
                 'destination_broker="c"\nsource_prefix="mirror/"\n'
                 'target_prefix="archive/"\nqos=1\n')
        topology = root / "topology.toml"
        private(topology, 'version=1\n' + brokers + links)
        def endpoint(index, client_id):
            return (f'broker_id="{"abc"[index]}"\nhost="127.0.0.1"\n'
                    f'port={ports[index]}\nclient_id="{client_id}"\n'
                    'username="device"\n'
                    f'password_file={json.dumps(str(root / "password"))}\n'
                    f'ca={json.dumps(str(root / "ca.crt"))}\n'
                    f'cert={json.dumps(str(root / "client.crt"))}\n'
                    f'key={json.dumps(str(root / "client.key"))}\n')
        def config(path, link_id, source, target, source_prefix, target_prefix,
                   topology_path=topology):
            private(path, 'version=1\nmax_retries=8\n'
                    + f'topology_file={json.dumps(str(topology_path))}\n'
                    + f'link_id="{link_id}"\n[source]\n'
                    + endpoint(source, f"c127-{link_id}-source")
                    + '[destination]\n'
                    + endpoint(target, f"c127-{link_id}-target")
                    + '[route]\n'
                    + f'source_prefix="{source_prefix}/"\n'
                    + f'target_prefix="{target_prefix}/"\nqos=1\n')
        first = root / "first.toml"
        second = root / "second.toml"
        config(first, "a-b", 0, 1, "sensors", "mirror")
        config(second, "b-c", 1, 2, "mirror", "archive")
        cyclic = root / "cyclic.toml"
        private(cyclic, 'version=1\n' + brokers + links
                + '[[links]]\nid="c-a"\nsource_broker="c"\n'
                'destination_broker="a"\nsource_prefix="archive/"\n'
                'target_prefix="sensors/"\nqos=1\n')
        cycle_config = root / "cycle-config.toml"
        config(cycle_config, "a-b", 0, 1, "sensors", "mirror", cyclic)
        rejected = subprocess.run([str(BRIDGE), "--config", str(cycle_config)],
                                  capture_output=True, timeout=5)
        assert rejected.returncode == 2, "three-link cycle was accepted"
        base = {key: value for key, value in os.environ.items() if not key.startswith("MQTT_")}
        base.update(MQTT_MODE="secure-mtls", MQTT_SERVER_CERT=str(root / "server.crt"),
                    MQTT_SERVER_KEY=str(root / "server.key"),
                    MQTT_CLIENT_CA=str(root / "ca.crt"),
                    MQTT_USERS_FILE=str(root / "users.toml"))
        broker_processes = []
        bridge_processes = []
        try:
            for index, port in enumerate(ports):
                env = dict(base, MQTT_BIND=f"127.0.0.1:{port}",
                           MQTT_STATE_DIR=str(root / f"state-{index}"),
                           MQTT_ACL_FILE=str(root / f"acl-{index}.toml"))
                with (root / f"broker-{index}.log").open("ab") as log:
                    child = subprocess.Popen([str(broker)], env=env, stdout=log, stderr=log)
                broker_processes.append(child)
                wait_port(child, port)
            for config_path in (second, first):
                child = subprocess.Popen([str(BRIDGE), "--config", str(config_path)],
                                         stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                         bufsize=0)
                bridge_processes.append(child)
                wait_markers(child, ("destino conectado", "origem assinada"))
            subscriber = ready_subscriber(root, ports[2], "archive/allowed")
            sent = subprocess.run(command(root, ports[0], "pub", "sensors/allowed",
                                          "c127-chain", qos=1), capture_output=True, timeout=10)
            assert sent.returncode == 0, sent.stderr.decode(errors="replace")
            received, errors = subscriber.communicate(timeout=8)
            assert subscriber.returncode == 0 and b"c127-chain" in received, (received, errors)
        finally:
            for child in bridge_processes:
                if child.poll() is None:
                    child.terminate()
                child.communicate(timeout=8)
            for child in broker_processes:
                if child.poll() is None:
                    child.send_signal(signal.SIGTERM)
                assert child.wait(timeout=8) == 0
    print("C127_BRIDGE_TOPOLOGY_PASS chain, three-link cycle rejection")


if __name__ == "__main__":
    main()
