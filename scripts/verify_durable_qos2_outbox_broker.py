#!/usr/bin/env python3
"""C107: resume both QoS 2 stages through a disposable XMQR broker restart."""
import argparse
import os
import socket
import subprocess
import tempfile
from pathlib import Path

from verify_disconnect_session_expiry_broker import start
from verify_disconnect_session_expiry_mqtt5 import CLIENT, packet
from verify_durable_qos1_outbox_broker import free_port, stop
from verify_durable_qos1_outbox_mqtt5 import listener
from verify_durable_qos2_outbox_mqtt5 import command, pending_path, publish_fields, stage


def wire(header, body):
    assert len(body) < 128, "test packet unexpectedly large"
    return bytes([header, len(body)]) + body


def subscriber(port):
    stream = socket.create_connection(("127.0.0.1", port), timeout=4)
    stream.settimeout(4)
    client_id = b"c107-observer"
    connect = (b"\x00\x04MQTT\x05\x02\x00\x1e\x00"
               + len(client_id).to_bytes(2, "big") + client_id)
    stream.sendall(wire(0x10, connect))
    assert packet(stream)[0] == 0x20
    topic = b"c105/outbox"
    subscribe = b"\x00\x01\x00" + len(topic).to_bytes(2, "big") + topic + b"\x00"
    stream.sendall(wire(0x82, subscribe))
    header, body = packet(stream)
    assert header == 0x90 and body[:2] == b"\x00\x01" and body[-1] == 0
    return stream


def deliveries(stream, expected):
    stream.settimeout(0.5 if expected == 0 else 4)
    try:
        header, body = packet(stream)
    except socket.timeout:
        assert expected == 0, "broker did not deliver QoS 2 publication"
        return
    assert expected == 1 and header & 0xf0 == 0x30, (header, body)
    topic_len = int.from_bytes(body[:2], "big")
    assert body[2:2 + topic_len] == b"c105/outbox"
    assert body[3 + topic_len:] == b"first"  # MQTT 5 zero properties
    stream.settimeout(0.5)
    try:
        duplicate = packet(stream)
    except socket.timeout:
        return
    raise AssertionError(f"duplicate subscriber delivery: {duplicate}")


def exchange(proxy_port, broker_port, directory, phase, resumed):
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
                assert header == 0x20 and body[1] == 0
                assert bool(body[0] & 1) is resumed, body
                upstream.sendall(wire(header, body))
                header, body = packet(upstream)
                if phase == "publish" or not resumed:
                    assert header == (0x3c if resumed else 0x34), hex(header)
                    topic, pkid, tail = publish_fields(header, body)
                    assert topic == b"c105/outbox" and pkid == 1 and tail == b"\x00first"
                    downstream.sendall(wire(header, body))
                    ack_header, ack_body = packet(downstream)
                    assert ack_header == 0x50 and ack_body[:2] == b"\x00\x01"
                    if phase == "publish" and not resumed:
                        child.kill()  # Broker received PUBLISH; PUBREC never reached client.
                    else:
                        upstream.sendall(wire(ack_header, ack_body))
                        header, body = packet(upstream)
                if not (phase == "publish" and not resumed):
                    assert (header, body[:2]) == (0x62, b"\x00\x01")
                    assert stage(directory) == "PubRelReady"
                    if phase == "pubrel" and not resumed:
                        child.kill()  # Client committed PUBREL; broker has not seen it.
                    else:
                        downstream.sendall(wire(header, body))
                        ack_header, ack_body = packet(downstream)
                        assert ack_header == 0x70 and ack_body[:2] == b"\x00\x01"
                        upstream.sendall(wire(ack_header, ack_body))
                        assert packet(upstream) == (0xe0, b"")
                        downstream.sendall(b"\xe0\x00")
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
    for phase in ("publish", "pubrel"):
        with tempfile.TemporaryDirectory(prefix=f"xmqr-c107-{phase}-") as root:
            state = os.path.join(root, "state")
            outbox = os.path.join(root, "outbox")
            os.mkdir(state, 0o700)
            os.mkdir(outbox, 0o700)
            broker_port, proxy_port = free_port(), free_port()
            broker = start(args.broker, broker_port, state)
            try:
                with subscriber(broker_port) as observer:
                    exchange(proxy_port, broker_port, outbox, phase, False)
                    assert os.path.isfile(pending_path(outbox))
                    deliveries(observer, 0)
            finally:
                stop(broker)
            broker = start(args.broker, broker_port, state)
            try:
                with subscriber(broker_port) as observer:
                    exchange(proxy_port, broker_port, outbox, phase, True)
                    assert not os.path.exists(pending_path(outbox))
                    deliveries(observer, 1)
            finally:
                stop(broker)
    print("C107_QOS2_OUTBOX_BROKER_PASS PUBLISH DUP, PUBREL resume, restart, subscriber once")


if __name__ == "__main__":
    main()
