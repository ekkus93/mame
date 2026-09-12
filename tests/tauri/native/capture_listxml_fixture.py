#!/usr/bin/env python3
"""Create bounded, provenance-bearing listxml evidence from a real MAME run."""

from __future__ import annotations

import argparse
import hashlib
import json
from copy import deepcopy
from pathlib import Path
import xml.etree.ElementTree as ET

KEEP_CHILDREN = {
    "description",
    "year",
    "manufacturer",
    "display",
    "chip",
    "driver",
    "device",
    "softwarelist",
}
MAX_MACHINES = 8


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--provenance", required=True, type=Path)
    parser.add_argument("--version-file", required=True, type=Path)
    parser.add_argument("--git-sha", required=True)
    return parser.parse_args()


def machine_score(machine: ET.Element) -> tuple[int, str]:
    score = 0
    if machine.get("cloneof"):
        score += 16
    if machine.get("isdevice") == "yes" or machine.get("runnable") == "no":
        score += 8
    if machine.find("display") is not None:
        score += 4
    if machine.find("device") is not None:
        score += 2
    if machine.find("softwarelist") is not None:
        score += 32
    return (-score, machine.get("name", ""))


def select_machines(root: ET.Element) -> list[ET.Element]:
    machines = list(root.findall("machine"))
    by_name = {machine.get("name"): machine for machine in machines if machine.get("name")}
    selected: list[ET.Element] = []
    selected_names: set[str] = set()

    for machine in sorted(machines, key=machine_score):
        if len(selected) >= MAX_MACHINES:
            break
        name = machine.get("name")
        if not name or name in selected_names:
            continue
        parent_name = machine.get("cloneof")
        if parent_name and parent_name in by_name and parent_name not in selected_names:
            selected.append(by_name[parent_name])
            selected_names.add(parent_name)
            if len(selected) >= MAX_MACHINES:
                break
        selected.append(machine)
        selected_names.add(name)

    return selected[:MAX_MACHINES]


def bounded_machine(machine: ET.Element) -> ET.Element:
    result = ET.Element("machine", dict(machine.attrib))
    for child in machine:
        if child.tag in KEEP_CHILDREN:
            result.append(deepcopy(child))
    return result


def main() -> None:
    args = parse_args()
    full_bytes = args.input.read_bytes()
    source_root = ET.fromstring(full_bytes)
    selected = select_machines(source_root)
    if not selected:
        raise SystemExit("captured listxml contains no machine entries")

    output_root = ET.Element("mame", dict(source_root.attrib))
    for machine in selected:
        output_root.append(bounded_machine(machine))

    tree = ET.ElementTree(output_root)
    ET.indent(tree, space="  ")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    tree.write(args.output, encoding="utf-8", xml_declaration=True)

    version = args.version_file.read_text(encoding="utf-8").strip()
    provenance = {
        "schemaVersion": 1,
        "repository": "ekkus93/mame",
        "gitSha": args.git_sha,
        "subtarget": "tiny",
        "mameVersionOutput": version,
        "fullListxmlSha256": hashlib.sha256(full_bytes).hexdigest(),
        "selectedMachines": [machine.get("name") for machine in selected],
        "selectionPolicy": "bounded representative metadata extracted from real SUBTARGET=tiny -listxml",
    }
    args.provenance.write_text(
        json.dumps(provenance, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
