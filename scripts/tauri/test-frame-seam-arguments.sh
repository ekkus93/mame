#!/usr/bin/env bash
set -euo pipefail

script=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/qualify-mame-frame-seams.sh
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT
cd "$root"

# Invalid machine names must fail before creating default report directories.
for machine in '../escape' 'foo/bar' 'PACMAN' '-help' 'name;other' 'abcdefghijklmnopq'; do
  set +e
  "$script" /bin/true "$machine" >result.stdout 2>result.stderr
  status=$?
  set -e
  [[ $status -eq 64 ]] || { echo "Invalid $machine returned status $status" >&2; exit 1; }
  grep -q 'Invalid MAME machine short name' result.stderr || {
    echo "Invalid $machine did not emit an actionable diagnosis" >&2
    exit 1
  }
  [[ ! -e artifacts ]] || { echo "Unexpected report directory for $machine" >&2; exit 1; }
done
echo 'Frame-seam machine argument validation passed.'
