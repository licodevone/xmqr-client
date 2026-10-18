#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C129: run existing client fixtures against an explicit private ELF, not target/debug."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


SCENARIOS = [
    ("verify_durable_qos1_outbox_mqtt5", ["C105_QOS1_OUTBOX_WIRE_PASS"]),
    ("verify_durable_qos2_outbox_mqtt5", ["C107_QOS2_OUTBOX_WIRE_PASS"]),
    ("verify_durable_subscription_mqtt5", ["C108_SUBSCRIPTION_WIRE_PASS"]),
    ("verify_subscription_ack_after_output_mqtt5", ["C110_ACK_AFTER_OUTPUT_PASS"]),
    ("verify_wss_durable_subscription", ["C123_WSS_DURABLE_SUBSCRIPTION_PASS"]),
    ("verify_wss_client", ["C114_WSS_MTLS_FIXTURE_PASS", "C116_PROFILE_FIXTURE_PASS",
                           "C119_WSS_QOS1_OUTBOX_PASS", "C120_WSS_QOS1_RECOVERY_PASS",
                           "C122_WSS_QOS2_OUTBOX_PASS"]),
]

BOOTSTRAP = """
import importlib, pathlib, sys
sys.path.insert(0, sys.argv[3])
import verify_disconnect_session_expiry_mqtt5 as common
common.CLIENT = pathlib.Path(sys.argv[1])
module = importlib.import_module(sys.argv[2])
for name, imported in tuple(sys.modules.items()):
    if name.startswith('verify_') and hasattr(imported, 'CLIENT'):
        imported.CLIENT = common.CLIENT
module.main()
"""


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def accepted(returncode, stdout, required):
    tokens = stdout.decode("utf-8", errors="replace").split()
    return returncode == 0 and bool(required) and all(marker in tokens for marker in required)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error("new report path required; failures are preserved")
    binary = args.client.resolve(strict=True)
    scripts = Path(__file__).resolve().parent
    environment = dict(os.environ, TMPDIR="/var/tmp", PYTHONDONTWRITEBYTECODE="1")
    report = dict(client_sha256=sha(binary), harness_sha256=sha(Path(__file__)),
                  scope="existing synthetic TCP/WSS mTLS fixtures; no operator state",
                  scenarios=[])
    started = time.monotonic()
    for module, required in SCENARIOS:
        began = time.monotonic()
        try:
            result = subprocess.run([sys.executable, "-B", "-c", BOOTSTRAP,
                                     str(binary), module, str(scripts)],
                                    env=environment, capture_output=True, timeout=120)
            # Preserve no raw assertion, credential, certificate or payload in
            # the shareable report. Expected marker is checked as a full token.
            passed = accepted(result.returncode, result.stdout, required)
            scenario = dict(status="PASS" if passed else "FAIL", module=module,
                            exit_code=result.returncode, expected_markers=required,
                            markers_observed=accepted(0, result.stdout, required))
        except subprocess.TimeoutExpired:
            scenario = dict(status="FAIL", module=module, reason="bounded fixture timeout")
        scenario["harness_sha256"] = sha(scripts / (module + ".py"))
        scenario["elapsed_seconds"] = round(time.monotonic() - began, 4)
        report["scenarios"].append(scenario)
        print(json.dumps(scenario), flush=True)
    report["status"] = "PASS" if all(item["status"] == "PASS" for item in report["scenarios"]) else "FAIL"
    report["elapsed_seconds"] = round(time.monotonic() - started, 4)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open("x", encoding="utf-8") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    print("C129_COMPATIBILITY_" + report["status"], flush=True)
    if report["status"] != "PASS":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
