#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: stage-mame-runtime.sh <mame-source-root> <mame-executable> <destination>

Stages the project-qualified MAME sidecar runtime expected by the Tauri package layer.
The destination is replaced atomically enough for CI/package staging: it is removed only
after all source preflight checks have passed.
EOF
}

if [[ $# -ne 3 ]]; then
  usage >&2
  exit 64
fi

source_root=$1
mame_executable=$2
destination=$3
qualification=${MAME_RUNTIME_QUALIFICATION:-structural}
script_dir=$(cd "$(dirname "$0")" && pwd)

fail() {
  printf 'stage-mame-runtime: %s\n' "$*" >&2
  exit 1
}

[[ -d "$source_root" ]] || fail "MAME source root is not a directory: $source_root"
[[ -f "$mame_executable" ]] || fail "MAME executable is not a regular file: $mame_executable"
[[ -d "$source_root/hash" ]] || fail "Required MAME hash directory is missing: $source_root/hash"
[[ -d "$source_root/bgfx" ]] || fail "Required MAME BGFX directory is missing: $source_root/bgfx"
[[ -f "$source_root/COPYING" ]] || fail "Required MAME COPYING file is missing: $source_root/COPYING"
[[ -d "$source_root/docs/legal" ]] || fail "Required MAME legal directory is missing: $source_root/docs/legal"
[[ -n "$(find "$source_root/hash" -maxdepth 1 -type f -name '*.xml' -print -quit)" ]] \
  || fail "MAME hash directory contains no software-list XML files"
[[ -n "$(find "$source_root/bgfx" -type f -print -quit)" ]] \
  || fail "MAME BGFX directory contains no runtime files"
[[ -n "$(find "$source_root/docs/legal" -type f -print -quit)" ]] \
  || fail "MAME legal directory contains no license files"

[[ -n "$destination" && "$destination" != "/" ]] \
  || fail "Refusing unsafe destination: $destination"

version_line=
if [[ "$qualification" == "real" ]]; then
  [[ -n "${MAME_SOURCE_SHA:-}" ]] || fail "MAME_SOURCE_SHA is required for real-runtime staging"
  bash "$script_dir/validate-real-mame-runtime.sh" "$mame_executable"
  version_line=$("$mame_executable" -noreadconfig -version 2>&1 | head -n 1)
elif [[ "$qualification" != "structural" ]]; then
  fail "Unsupported MAME_RUNTIME_QUALIFICATION: $qualification"
fi

case "$(basename "$mame_executable")" in
  *.exe|*.EXE)
    executable_name=mame.exe
    ;;
  *)
    executable_name=mame
    ;;
esac

staging_parent=$(dirname "$destination")
mkdir -p "$staging_parent"
staging=$(mktemp -d "$staging_parent/.mame-runtime.XXXXXX")
cleanup() {
  rm -rf "$staging"
}
trap cleanup EXIT

mkdir -p "$staging/bin" "$staging/licenses"
cp -p "$mame_executable" "$staging/bin/$executable_name"
if [[ "$executable_name" == "mame" ]]; then
  chmod 0755 "$staging/bin/$executable_name"
fi
cp -R "$source_root/hash" "$staging/hash"
cp -R "$source_root/bgfx" "$staging/bgfx"
cp -p "$source_root/COPYING" "$staging/licenses/COPYING"
cp -R "$source_root/docs/legal" "$staging/licenses/legal"

if [[ "$qualification" == "real" ]]; then
  binary_sha256=$(sha256sum "$mame_executable" | awk '{print $1}')
  python3 - \
    "$staging/provenance.json" \
    "$MAME_SOURCE_SHA" \
    "$version_line" \
    "${MAME_BUILD_TARGET:-mame}" \
    "${MAME_BUILD_PROFILE:-release}" \
    "$binary_sha256" \
    "$(uname -sm)" <<'PY'
import json
import sys
from pathlib import Path

(
    path,
    source_sha,
    version_line,
    build_target,
    build_profile,
    binary_sha256,
    build_host,
) = sys.argv[1:]
Path(path).write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "sourceRevision": source_sha,
            "versionLine": version_line,
            "buildTarget": build_target,
            "buildProfile": build_profile,
            "binarySha256": binary_sha256,
            "buildHost": build_host,
            "qualification": "real",
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
PY
fi

[[ -f "$staging/bin/$executable_name" ]] || fail "Staged executable is missing"
[[ -n "$(find "$staging/hash" -maxdepth 1 -type f -name '*.xml' -print -quit)" ]] \
  || fail "Staged hash directory is incomplete"
[[ -n "$(find "$staging/bgfx" -type f -print -quit)" ]] \
  || fail "Staged BGFX directory is incomplete"
[[ -s "$staging/licenses/COPYING" ]] || fail "Staged COPYING file is missing or empty"
[[ -n "$(find "$staging/licenses/legal" -type f -print -quit)" ]] \
  || fail "Staged legal directory is incomplete"

rm -rf "$destination"
mv "$staging" "$destination"
trap - EXIT

printf 'Staged MAME runtime at %s\n' "$destination"
