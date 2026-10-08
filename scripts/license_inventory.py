# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Report declared dependency licenses; --check detects unreviewed inventory changes.

This inspects Cargo metadata, not upstream source license/NOTICE contents.
The inventory includes dev dependencies and targets other than the build host.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tomllib


def inventory(root):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root))
    package = tomllib.loads((root / "Cargo.toml").read_text())["package"]
    if package.get("license") != "MIT":
        raise ValueError("XMQR must declare license = MIT")
    license_text = (root / "LICENSE").read_text()
    if not license_text.startswith("MIT License") or "Copyright (c) 2026 Luis E. S. Pinheiro" not in license_text:
        raise ValueError("project MIT notice missing or changed; review required")
    lines = [
        "# Licenças declaradas das dependências", "",
        "Inventário gerado por `python3 scripts/license_inventory.py` com o `Cargo.lock` atual.",
        "Inclui dependências de desenvolvimento e de outras plataformas; não é uma lista dos componentes de um binário específico.",
        "As expressões abaixo são as declarações originais dos pacotes, sem relicenciamento.",
        "Para distribuição, preserve os textos LICENSE/COPYING e os avisos NOTICE originais exigidos pelos componentes incluídos.",
        "Este inventário não substitui esses textos nem uma revisão de licença dos fontes vendorizados.", "",
        "| Pacote | Versão | Licença declarada |", "| --- | --- | --- |",
    ]
    for dependency in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if dependency["id"] in metadata["workspace_members"]:
            continue
        declared = dependency.get("license")
        if not declared:
            raise ValueError(f"license metadata missing: {dependency['name']}")
        lines.append(f"| {dependency['name']} | {dependency['version']} | `{declared}` |")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    rendered = inventory(root)
    if args.check:
        path = root / "docs/third-party-licenses.md"
        if not path.exists() or path.read_text() != rendered:
            sys.exit("license inventory changed: review metadata and update docs/third-party-licenses.md")
        print("Project MIT declaration and dependency license inventory match.")
    else:
        print(rendered, end="")


if __name__ == "__main__":
    main()
