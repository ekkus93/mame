#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <package.deb> <bundled-mame-executable>" >&2
  exit 64
fi

deb=$(realpath "$1")
mame=$(realpath "$2")
[[ -f "$deb" ]] || { echo "missing Debian package: $deb" >&2; exit 1; }
[[ -x "$mame" ]] || { echo "missing bundled MAME executable: $mame" >&2; exit 1; }

if ldd "$mame" | grep -q 'not found'; then
  echo "bundled MAME has unresolved build-host dependencies" >&2
  ldd "$mame" >&2
  exit 1
fi

declare -A seen=()
runtime_packages=()
while IFS= read -r library; do
  [[ -n "$library" ]] || continue
  canonical=$(readlink -f "$library")
  owner=$(dpkg-query -S "$canonical" 2>/dev/null | head -n 1 | cut -d: -f1 || true)
  if [[ -z "$owner" ]]; then
    owner=$(dpkg-query -S "$library" 2>/dev/null | head -n 1 | cut -d: -f1 || true)
  fi
  [[ -n "$owner" ]] || {
    echo "no Debian package owns runtime library: $library" >&2
    exit 1
  }
  owner=${owner%%:*}
  if [[ -z "${seen[$owner]:-}" ]]; then
    seen[$owner]=1
    runtime_packages+=("$owner")
  fi
done < <(
  ldd "$mame" |
    awk '
      /=> \/[^ ]+/ { print $3 }
      /^[[:space:]]*\/[^ ]+/ { print $1 }
    ' |
    sort -u
)

if [[ ${#runtime_packages[@]} -eq 0 ]]; then
  echo "bundled MAME has no shared-library package dependencies to add" >&2
  exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

dpkg-deb -R "$deb" "$tmp/root"
printf '%s\n' "${runtime_packages[@]}" >"$tmp/runtime-packages.txt"
python3 - "$tmp/root/DEBIAN/control" "$tmp/runtime-packages.txt" <<'PY'
from pathlib import Path
import re
import sys

control_path = Path(sys.argv[1])
runtime_packages = [line.strip() for line in Path(sys.argv[2]).read_text().splitlines() if line.strip()]
text = control_path.read_text()

match = re.search(r"(?ms)^Depends:\s*(.*?)(?=\n[^ \t]|\Z)", text)
if not match:
    raise SystemExit("Debian control file has no Depends field")

existing = " ".join(line.strip() for line in match.group(1).splitlines())
clauses = [clause.strip() for clause in existing.split(",") if clause.strip()]
names = {re.split(r"\s|\(|:", clause, maxsplit=1)[0] for clause in clauses}
for package in runtime_packages:
    if package not in names:
        clauses.append(package)
        names.add(package)

replacement = "Depends: " + ", ".join(clauses)
text = text[: match.start()] + replacement + text[match.end() :]
control_path.write_text(text)
PY

rebuilt="$tmp/rebuilt.deb"
dpkg-deb --build --root-owner-group "$tmp/root" "$rebuilt" >/dev/null
mv "$rebuilt" "$deb"

python3 - "$deb" "$tmp/runtime-packages.txt" <<'PY'
import re
import subprocess
import sys
from pathlib import Path

deb = sys.argv[1]
expected = [line.strip() for line in Path(sys.argv[2]).read_text().splitlines() if line.strip()]
depends = subprocess.check_output(["dpkg-deb", "--field", deb, "Depends"], text=True)
clauses = [clause.strip() for clause in depends.split(",") if clause.strip()]
names = {re.split(r"\s|\(|:", clause, maxsplit=1)[0] for clause in clauses}
missing = [package for package in expected if package not in names]
if missing:
    raise SystemExit(f"rebuilt Debian package does not declare bundled MAME dependencies {missing}; Depends: {depends}")
PY

printf 'Bundled MAME Debian dependencies: %s\n' "${runtime_packages[*]}"
