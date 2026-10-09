#!/usr/bin/env bash
set -euo pipefail

verifier=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/verify-runtime-provenance.sh
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT
mkdir -p "$root/bin"
printf 'first executable bytes\n' >"$root/bin/mame"
digest=$(sha256sum "$root/bin/mame" | awk '{print $1}')
printf 'mode=real\nrelease_qualified=true\nmame_sha256=%s\n' "$digest" >"$root/runtime-provenance.txt"
"$verifier" "$root" >/dev/null

printf 'mutated executable bytes\n' >"$root/bin/mame"
set +e
"$verifier" "$root" >result.stdout 2>result.stderr
status=$?
set -e
[[ $status -ne 0 ]] || { echo 'Mismatched executable/provenance unexpectedly passed' >&2; exit 1; }
grep -q 'does not match runtime provenance' result.stderr || {
  echo 'Mismatched executable/provenance lacked actionable diagnosis' >&2
  cat result.stderr >&2
  exit 1
}
echo 'Runtime provenance executable-integrity validation passed.'
