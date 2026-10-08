# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Verify C27 declared reused assets and unchanged production; never copies source."""
from pathlib import Path
import hashlib
import json

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    inputs = json.loads((HERE / "inputs.json").read_text(encoding="utf-8-sig"))
    production = {name: sha(ROOT / "src" / name)
                  for name in inputs["production_source_sha256"]}
    if production != inputs["production_source_sha256"]:
        raise SystemExit("FAIL: production source changed after C27 scope")
    asset_map = {"Cargo.toml": "Cargo.toml", "Cargo.lock": "Cargo.lock",
                 "LICENSE": "LICENSE", "tests/wire.rs": "tests/c26_wire_contract.rs"}
    for original, retained in asset_map.items():
        expected = inputs["allowed_reused_assets"][original]
        if sha(ROOT / original) != expected or sha(HERE / retained) != expected:
            raise SystemExit("FAIL: declared asset drift: " + original)
    reconstruction = {p.name: sha(p) for p in sorted((HERE / "src").glob("*.rs"))}
    if any(value in production.values() for value in reconstruction.values()):
        raise SystemExit("FAIL: byte-identical implementation source")
    report = {"production_source_unchanged": True,
              "declared_assets_unchanged": True,
              "no_byte_identical_implementation_files": True,
              "production_source_sha256": production,
              "reconstruction_source_sha256": reconstruction,
              "limitation": "Hashes alone do not establish independent authorship or behavioral equivalence."}
    (HERE / "provenance.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("PASS: production/declared assets unchanged; source hashes differ")


if __name__ == "__main__":
    main()
