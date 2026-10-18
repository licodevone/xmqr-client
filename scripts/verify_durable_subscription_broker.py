#!/usr/bin/env python3
"""C108: persisted broker subscription resumes without a new SUBSCRIBE."""
import argparse
import os
import socket
import subprocess
import tempfile
import time
from pathlib import Path

from verify_disconnect_session_expiry_broker import start
from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet
from verify_durable_qos1_outbox_broker import free_port, stop
from verify_durable_qos1_outbox_mqtt5 import listener
from verify_durable_subscription_mqtt5 import command, stage


def wire(header, body):
    assert len(body) < 128
    return bytes([header, len(body)]) + body


def first(proxy_port, broker_port, directory):
    with listener(proxy_port) as proxy:
        child = subprocess.Popen(command(proxy_port, directory),
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with proxy.accept()[0] as upstream, socket.create_connection(
                    ("127.0.0.1", broker_port), timeout=4) as downstream:
                upstream.settimeout(4)
                downstream.settimeout(4)
                header, body = packet(upstream)
                downstream.sendall(wire(header, body))
                header, body = packet(downstream)
                assert header == 0x20 and body[1] == 0 and body[0] & 1 == 0
                upstream.sendall(wire(header, body))
                header, body = packet(upstream)
                assert header == 0x82
                downstream.sendall(wire(header, body))
                header, body = packet(downstream)
                assert header == 0x90
                upstream.sendall(wire(header, body))
                for _ in range(100):
                    if stage(directory) == "Active":
                        break
                    time.sleep(0.01)
                assert stage(directory) == "Active"
                child.kill()
            child.communicate(timeout=5)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def resumed(proxy_port, broker_port, directory):
    with listener(proxy_port) as proxy:
        child = subprocess.Popen(command(proxy_port, directory),
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with proxy.accept()[0] as upstream, socket.create_connection(
                    ("127.0.0.1", broker_port), timeout=4) as downstream:
                upstream.settimeout(4)
                downstream.settimeout(4)
                header, body = packet(upstream)
                assert header == 0x10
                downstream.sendall(wire(header, body))
                header, body = packet(downstream)
                assert header == 0x20 and body[1] == 0 and body[0] & 1 == 1
                upstream.sendall(wire(header, body))
                header, body = packet(downstream)
                assert header & 0xf0 == 0x30 and b"queued" in body
                downstream_publish = wire(header, body)
                upstream.sendall(downstream_publish)
                header, body = packet(upstream)
                assert header == 0x40, (header, body)  # No renewed SUBSCRIBE.
                downstream.sendall(wire(header, body))
                time.sleep(0.15)
                child.kill()
            stdout, stderr = child.communicate(timeout=5)
            assert b"queued" in stdout, (stdout, stderr)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def publish(broker_port):
    args = [str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
            "--host", "127.0.0.1", "--port", str(broker_port),
            "--topic", "c108/data", "--message", "queued", "--qos", "1",
            "--client-id", "c108-publisher", "--timeout-seconds", "4"]
    result = subprocess.run(args, capture_output=True, timeout=8)
    assert result.returncode == 0, result.stderr


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker", type=Path, required=True)
    args = parser.parse_args()
    assert args.broker.is_file() and CLIENT.is_file()
    with tempfile.TemporaryDirectory(prefix="xmqr-c108-broker-") as root:
        state = os.path.join(root, "state")
        subscription = os.path.join(root, "subscription")
        os.mkdir(state, 0o700)
        os.mkdir(subscription, 0o700)
        broker_port, proxy_port = free_port(), free_port()
        broker = start(args.broker, broker_port, state)
        try:
            first(proxy_port, broker_port, subscription)
            publish(broker_port)
        finally:
            stop(broker)
        broker = start(args.broker, broker_port, state)
        try:
            resumed(proxy_port, broker_port, subscription)
        finally:
            stop(broker)
    print("C108_SUBSCRIPTION_BROKER_PASS queued publication after restart without SUBSCRIBE")


if __name__ == "__main__":
    main()
