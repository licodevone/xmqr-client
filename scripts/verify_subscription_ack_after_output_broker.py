#!/usr/bin/env python3
"""C110: failed stdout leaves QoS 1 unacknowledged for broker redelivery."""
import argparse
import os
import subprocess
import tempfile
from pathlib import Path

from verify_disconnect_session_expiry_mqtt5 import CLIENT
from verify_durable_qos1_outbox_broker import free_port, stop
from verify_durable_subscription_broker import first, publish, resumed
from verify_disconnect_session_expiry_broker import start
from verify_durable_subscription_mqtt5 import command


def failed_output(broker_port, directory):
    with open("/dev/full", "wb") as full:
        child = subprocess.run(command(broker_port, directory) + ["--output", "jsonl"],
                               stdout=full, stderr=subprocess.PIPE, timeout=8)
    assert child.returncode != 0, child.stderr


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker", type=Path, required=True)
    args = parser.parse_args()
    assert args.broker.is_file() and CLIENT.is_file()
    with tempfile.TemporaryDirectory(prefix="xmqr-c110-broker-") as root:
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
            failed_output(broker_port, subscription)
        finally:
            stop(broker)
        broker = start(args.broker, broker_port, state)
        try:
            resumed(proxy_port, broker_port, subscription)
        finally:
            stop(broker)
    print("C110_BROKER_REDELIVERY_PASS failed stdout, broker restart, QoS1 redelivery")


if __name__ == "__main__":
    main()
