#!/usr/bin/env bash
set -euo pipefail

script=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/qualify-mame-frame-seams.sh
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT
cd "$root"

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

export MAME_TAURI_FRAME_SEAM_EVIDENCE_ROOT="$root/evidence"
export MAME_TAURI_FRAME_SEAM_VALIDATE_OUTPUT_ONLY=1
simple=$("$script" /bin/true pacman '' report.txt)
[[ "$simple" == "$root/evidence/report.txt" ]] || { echo "Unexpected simple report path: $simple" >&2; exit 1; }
nested=$("$script" /bin/true pacman '' nested/report.txt)
[[ "$nested" == "$root/evidence/nested/report.txt" ]] || { echo "Unexpected nested report path: $nested" >&2; exit 1; }

set +e
"$script" /bin/true pacman '' ../escape.txt >result.stdout 2>result.stderr
status=$?
set -e
[[ $status -eq 64 ]] || { echo "Traversal output returned status $status" >&2; exit 1; }
grep -q 'escapes evidence root' result.stderr || { echo 'Traversal output lacked diagnosis' >&2; exit 1; }

mkdir -p "$root/evidence" "$root/outside"
ln -s "$root/outside" "$root/evidence/link"
set +e
"$script" /bin/true pacman '' link/report.txt >result.stdout 2>result.stderr
status=$?
set -e
[[ $status -eq 64 ]] || { echo "Symlink escape returned status $status" >&2; exit 1; }
grep -q 'escapes evidence root' result.stderr || { echo 'Symlink escape lacked diagnosis' >&2; exit 1; }

absolute="$root/absolute-report.txt"
resolved=$("$script" /bin/true pacman '' "$absolute")
[[ "$resolved" == "$absolute" ]] || { echo "Absolute output policy changed: $resolved" >&2; exit 1; }

[[ ! -e "$root/escape.txt" ]] || { echo 'Traversal validation created escaped output' >&2; exit 1; }
echo 'Frame-seam machine and output-path validation passed.'
