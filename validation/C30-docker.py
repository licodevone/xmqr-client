# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Check binary stdin/line-mode through the locally built XMQR images."""
import argparse
import json
import subprocess
import time
import uuid


def docker(*args, check=True, **kwargs):
    result = subprocess.run(["docker", *map(str, args)], capture_output=True, **kwargs)
    if check and result.returncode:
        raise RuntimeError(result.stderr.decode(errors="replace"))
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--broker-image", required=True)
    parser.add_argument("--client-image", required=True)
    args = parser.parse_args()

    suffix = uuid.uuid4().hex[:10]
    broker = "xmqr-c30-broker-" + suffix
    subscriber = "xmqr-c30-sub-" + suffix
    volume = "xmqr-c30-state-" + suffix
    docker("volume", "create", volume)
    try:
        docker(
            "run", "--detach", "--name", broker,
            "--mount", f"type=volume,src={volume},dst=/var/lib/xmqr",
            "--env", "MQTT_MODE=open-lab",
            "--env", "MQTT_BIND=127.0.0.1:1883",
            args.broker_image,
        )
        docker(
            "run", "--detach", "--name", subscriber,
            "--network", f"container:{broker}",
            args.client_image, "sub", "--open-lab", "--host", "127.0.0.1",
            "--port", "1883", "--topic", "docker/stdin", "--qos", "1",
            "--count", "2", "--output", "jsonl",
        )

        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            status = docker("inspect", "--format", "{{.State.Running}}", subscriber)
            if status.stdout.strip() != b"true":
                raise RuntimeError("subscriber exited before becoming active")
            logs = docker("logs", subscriber).stdout + docker("logs", subscriber).stderr
            if b"Assinatura ativa" in logs:
                break
            time.sleep(0.05)
        else:
            raise RuntimeError("subscriber did not report an active subscription")

        payload = b"\x00\xff\n\x22\n"
        published = docker(
            "run", "--rm", "--interactive", "--network", f"container:{broker}",
            args.client_image, "pub", "--open-lab", "--host", "127.0.0.1",
            "--port", "1883", "--topic", "docker/stdin", "--qos", "1",
            "--stdin", "--line-mode", "--output", "jsonl",
            input=payload, timeout=10,
        )
        if b"publish_complete" not in published.stdout:
            raise RuntimeError("client did not report line publication completions")

        exit_code = docker("wait", subscriber, timeout=10).stdout.strip()
        logs = docker("logs", subscriber)
        output = logs.stdout.decode(errors="replace").splitlines()
        records = [json.loads(line) for line in output if line.startswith("{")]
        if exit_code != b"0" or [item.get("payload_bytes") for item in records] != [[0, 255], [34]]:
            raise RuntimeError("subscriber JSONL did not preserve both binary lines")
        if any(item.get("topic") != "docker/stdin" for item in records):
            raise RuntimeError("subscriber returned an unexpected topic")
        print("binary stdin line-mode JSONL integration: PASS")
    finally:
        docker("rm", "--force", subscriber, check=False)
        docker("rm", "--force", broker, check=False)
        docker("volume", "rm", volume, check=False)


if __name__ == "__main__":
    main()
