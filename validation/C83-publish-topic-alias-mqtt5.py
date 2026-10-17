#!/usr/bin/env python3
"""C83: compact MQTT 5 topic alias against XMQR and optional Mosquitto."""
import os
import subprocess
import sys
import tempfile
from pathlib import Path

BROKER_ROOT = Path(__file__).resolve().parents[2] / "xmqr"
sys.path.insert(0, str(BROKER_ROOT / "scripts"))
from verify_mqtt5_sessions import BROKER, CLIENT, start, stop  # noqa: E402
from importlib.machinery import SourceFileLoader

c81 = SourceFileLoader("c81", str(Path(__file__).with_name("C81-repeat-mqtt5.py"))).load_module()


def run_case(port, label):
    stream = c81.subscriber(port, f"c83-{label}".encode())
    try:
        for qos in (0, 1, 2):
            command = [str(CLIENT), "pub", "--open-lab", "--protocol-version", "5",
                       "--port", str(port), "--topic", c81.TOPIC.decode(),
                       "--message", "hello", "--qos", str(qos), "--repeat", "3",
                       "--repeat-interval-ms", "10", "--publish-topic-alias", "1",
                       "--payload-format", "1", "--content-type", "text/plain",
                       "--publish-user-property", "trace=one", "--output", "jsonl"]
            publisher = subprocess.Popen(command, stdout=subprocess.PIPE,
                                         stderr=subprocess.PIPE)
            for _ in range(3):
                c81.publication(stream, qos)
            output, errors = publisher.communicate(timeout=8)
            assert publisher.returncode == 0, errors
            assert output.count(b'"event":"publish_complete"') == 3, output
    finally:
        stream.close()


def main():
    assert BROKER.is_file() and CLIENT.is_file(), "build Linux broker/client first"
    with tempfile.TemporaryDirectory(prefix="xmqr-c83-alias-") as state:
        port = c81.free_port()
        broker = start(port, state)
        try:
            run_case(port, "xmqr")
        finally:
            stop(broker)
    external = os.environ.get("C83_MOSQUITTO_PORT")
    if external:
        run_case(int(external), "mosquitto")
    print("PASS: C83 compact MQTT 5 Topic Alias QoS0/1/2 against XMQR"
          + (" and external Mosquitto" if external else ""))


if __name__ == "__main__":
    main()
