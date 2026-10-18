#!/usr/bin/env python3
"""C105: interrupt PUBACK, restart XMQR, then resume the same QoS 1 packet."""
import argparse
import os
import socket
import subprocess
import tempfile
from pathlib import Path

from verify_disconnect_session_expiry_broker import start
from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet
from verify_durable_qos1_outbox_mqtt5 import command, listener, publish_fields


def free_port():
    with listener() as server:
        return server.getsockname()[1]


def stop(child):
    child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=5)


def exchange(proxy_port, broker_port, directory, resumed):
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
                downstream.sendall(bytes([header, len(body)]) + body)
                header, body = packet(downstream)
                assert header == 0x20 and body[1] == 0
                assert bool(body[0] & 1) is resumed, body
                upstream.sendall(bytes([header, len(body)]) + body)
                header, body = packet(upstream)
                assert header == (0x3a if resumed else 0x32), hex(header)
                topic, packet_id, tail = publish_fields(header, body)
                assert topic == b"c105/outbox" and packet_id == 1 and tail == b"\x00first"
                downstream.sendall(bytes([header, len(body)]) + body)
                ack_header, ack_body = packet(downstream)
                assert ack_header == 0x40 and ack_body[:2] == b"\x00\x01"
                if resumed:
                    upstream.sendall(bytes([ack_header, len(ack_body)]) + ack_body)
                    assert packet(upstream) == (0xe0, b"")
                    downstream.sendall(b"\xe0\x00")
                else:
                    child.kill()  # The broker ACK exists, but the client never sees it.
            _, stderr = child.communicate(timeout=6)
            assert (child.returncode == 0) is resumed, stderr
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker", type=Path, required=True)
    args = parser.parse_args()
    assert args.broker.is_file() and CLIENT.is_file(), "build Linux broker and client"
    with tempfile.TemporaryDirectory(prefix="xmqr-c105-broker-") as root:
        state = os.path.join(root, "state")
        outbox = os.path.join(root, "outbox")
        os.mkdir(state, 0o700)
        os.mkdir(outbox, 0o700)
        broker_port = free_port()
        proxy_port = free_port()
        broker = start(args.broker, broker_port, state)
        try:
            exchange(proxy_port, broker_port, outbox, False)
            assert os.path.isfile(os.path.join(outbox, "pending.json"))
        finally:
            stop(broker)
        broker = start(args.broker, broker_port, state)
        try:
            exchange(proxy_port, broker_port, outbox, True)
            assert not os.path.exists(os.path.join(outbox, "pending.json"))
        finally:
            stop(broker)
    print("C105_QOS1_OUTBOX_BROKER_PASS lost PUBACK, restart, Session Present, DUP, completion")


if __name__ == "__main__":
    main()
