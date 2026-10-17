#!/usr/bin/env python3
"""C80: inspect JSONL publication metadata over real MQTT 5 TCP."""
import json
from pathlib import Path
import select
import socket
import subprocess
import sys
import tempfile

BROKER_ROOT = Path(__file__).resolve().parents[2] / "xmqr"
sys.path.insert(0, str(BROKER_ROOT / "scripts"))
from verify_mqtt5_sessions import BROKER, CLIENT, start, stop  # noqa: E402


def subscribe(port, topic, qos=2):
    process = subprocess.Popen([
        str(CLIENT), "sub", "--open-lab", "--protocol-version", "5",
        "--port", str(port), "--topic", topic, "--qos", str(qos),
        "--count", "1", "--output", "jsonl", "--client-id", "c80-" + topic.replace("/", "-"),
    ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    ready, _, _ = select.select([process.stderr], [], [], 6)
    assert ready, "subscriber readiness timeout"
    line = process.stderr.readline()
    assert b"Assinatura ativa" in line, line
    return process


def received(process, qos, retain=False, payload=b"payload", correlation_hex="00ff07"):
    try:
        output, errors = process.communicate(timeout=8)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=3)
    assert process.returncode == 0, errors
    lines = output.splitlines()
    assert len(lines) == 1, output
    event = json.loads(lines[0])
    assert event["event"] == "publish" and event["qos"] == qos
    assert event["retain"] is retain and event["payload_bytes"] == list(payload)
    metadata = event["metadata"]
    assert metadata["payload_format_indicator"] == 1
    assert metadata["content_type"] == "text/plain"
    assert metadata["response_topic"] == "reply/t"
    assert metadata["correlation_data_hex"] == correlation_hex
    assert metadata["user_properties"] == [["trace", "one"], ["trace", "two"]]
    return metadata


def publish_client(port, topic, qos, retain=False):
    result = subprocess.run([
        str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
        "--port", str(port), "--topic", topic, "--qos", str(qos),
        "--message", "payload", "--retain", str(retain).lower(),
        "--message-expiry-seconds", "30", "--payload-format", "1",
        "--content-type", "text/plain", "--response-topic", "reply/t",
        "--correlation-data-hex", "00ff07",
        "--publish-user-property", "trace=one",
        "--publish-user-property", "trace=two",
    ], capture_output=True, timeout=10)
    assert result.returncode == 0, result.stderr


def publish_mosquitto(port, topic):
    result = subprocess.run([
        "mosquitto_pub", "-V", "mqttv5", "-i", "c80-mosquitto-pub",
        "-h", "127.0.0.1", "-p", str(port),
        "-t", topic, "-q", "1", "-m", "payload",
        "-D", "publish", "payload-format-indicator", "1",
        "-D", "publish", "content-type", "text/plain",
        "-D", "publish", "response-topic", "reply/t",
        "-D", "publish", "correlation-data", "00ff07",
        "-D", "publish", "user-property", "trace", "one",
        "-D", "publish", "user-property", "trace", "two",
    ], capture_output=True, timeout=8)
    assert result.returncode == 0, result.stderr


def main():
    assert BROKER.is_file() and CLIENT.is_file(), "build Linux broker/client first"
    with tempfile.TemporaryDirectory(prefix="xmqr-c80-metadata-") as state:
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        broker = start(port, state)
        try:
            for qos in (0, 1, 2):
                topic = f"c80/live/{qos}"
                process = subscribe(port, topic)
                publish_client(port, topic, qos)
                metadata = received(process, qos)
                assert 0 < metadata["message_expiry_interval"] <= 30
            topic = "c80/mosquitto"
            process = subscribe(port, topic, qos=1)
            publish_mosquitto(port, topic)
            received(process, 1, correlation_hex="303066663037")
            publish_client(port, "c80/retained", 1, retain=True)
        finally:
            stop(broker)
        broker = start(port, state)
        try:
            process = subscribe(port, "c80/retained", qos=1)
            metadata = received(process, 1, retain=True)
            assert 0 < metadata["message_expiry_interval"] <= 30
        finally:
            stop(broker)
    print("PASS: C80 JSONL MQTT 5 metadata QoS0/1/2, retained/restart, XMQR and Mosquitto publishers")


if __name__ == "__main__":
    main()
