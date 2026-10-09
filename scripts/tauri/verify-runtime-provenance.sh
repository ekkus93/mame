#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo 'Usage: verify-runtime-provenance.sh <mame-runtime-root>' >&2
  exit 64
fi

root=$(realpath "$1")
executable="$root/bin/mame"
provenance="$root/runtime-provenance.txt"
[[ -f "$executable" ]] || { echo "Bundled MAME executable is missing: $executable" >&2; exit 1; }
[[ -f "$provenance" ]] || { echo "Bundled runtime provenance is missing: $provenance" >&2; exit 1; }

mapfile -t digest_lines < <(sed -n 's/^mame_sha256=//p' "$provenance")
[[ ${#digest_lines[@]} -eq 1 ]] || {
  echo 'Bundled runtime provenance must contain exactly one mame_sha256 entry.' >&2
  exit 1
}
expected=${digest_lines[0],,}
[[ "$expected" =~ ^[0-9a-f]{64}$ ]] || {
  echo "Bundled runtime provenance contains an invalid mame_sha256: $expected" >&2
  exit 1
}
actual=$(sha256sum "$executable" | awk '{print $1}')
if [[ "$actual" != "$expected" ]]; then
  echo "Bundled MAME executable SHA-256 does not match runtime provenance: expected=$expected actual=$actual" >&2
  exit 1
fi
printf 'Bundled runtime provenance matches executable SHA-256: %s\n' "$actual"
