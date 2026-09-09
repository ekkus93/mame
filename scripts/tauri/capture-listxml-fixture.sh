#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 MAME_EXECUTABLE OUTPUT_XML [MACHINE ...]" >&2
  exit 64
fi

mame=$1
output=$2
shift 2
machines=("${@:-galaxian}" )
if [[ $# -eq 0 ]]; then
  machines=(galaxian galaxiana apple2e z80)
fi

if [[ ! -x "$mame" ]]; then
  echo "MAME executable is not executable: $mame" >&2
  exit 66
fi

identity=$($mame -noreadconfig -version | head -n 1)
if [[ -z "$identity" ]]; then
  echo "MAME version probe returned no identity" >&2
  exit 70
fi

mkdir -p "$(dirname "$output")"
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

"$mame" -noreadconfig -listxml >"$tmp"

python3 - "$tmp" "$output" "$identity" "${machines[@]}" <<'PY'
import copy
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

source = Path(sys.argv[1])
destination = Path(sys.argv[2])
identity = sys.argv[3]
wanted = set(sys.argv[4:])

context = ET.iterparse(source, events=("start", "end"))
_, root = next(context)
root_attributes = dict(root.attrib)
selected = []
for event, element in context:
    if event == "end" and element.tag == "machine":
        if element.attrib.get("name") in wanted:
            selected.append(copy.deepcopy(element))
        root.remove(element)
        element.clear()

found = {machine.attrib.get("name") for machine in selected}
missing = sorted(wanted - found)
if missing:
    raise SystemExit(f"requested machine(s) missing from -listxml: {', '.join(missing)}")

reduced = ET.Element("mame", root_attributes)
for machine in selected:
    reduced.append(machine)
ET.indent(reduced, space="  ")
ET.ElementTree(reduced).write(destination, encoding="utf-8", xml_declaration=True)

destination.with_suffix(destination.suffix + ".identity.txt").write_text(
    identity + "\n", encoding="utf-8"
)
PY

printf 'captured %s\nidentity: %s\n' "$output" "$identity"
