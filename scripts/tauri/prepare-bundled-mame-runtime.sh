#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: prepare-bundled-mame-runtime.sh <destination>

Stages the MAME runtime used by Tauri Linux bundles.

Production/release staging requires:
  MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT=<path to MAME source tree>
  MAME_TAURI_REAL_RUNTIME_EXECUTABLE=<path to built MAME executable>

Fast structural package tests may opt into a synthetic fixture with:
  MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME=1

Synthetic staging is explicitly marked non-release-qualified in runtime-provenance.txt.
EOF
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 64
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
stager="$repo_root/scripts/tauri/stage-mame-runtime.sh"
destination=$1

fail() {
  printf 'prepare-bundled-mame-runtime: %s\n' "$*" >&2
  exit 1
}

write_provenance() {
  local mode=$1
  local runtime_root=$2
  local source_root=$3
  local executable=$4
  local version_line=$5
  local release_qualified=$6

  cat >"$runtime_root/runtime-provenance.txt" <<EOF
mode=$mode
release_qualified=$release_qualified
frontend_sha=${GITHUB_SHA:-unknown}
mame_source_root=$source_root
mame_executable=$executable
mame_version_line=$version_line
created_at_utc=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
EOF
}

stage_real_runtime() {
  local source_root=${MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT:-}
  local executable=${MAME_TAURI_REAL_RUNTIME_EXECUTABLE:-}

  [[ -n "$source_root" ]] || fail 'MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT is required for real runtime staging'
  [[ -n "$executable" ]] || fail 'MAME_TAURI_REAL_RUNTIME_EXECUTABLE is required for real runtime staging'
  [[ -d "$source_root" ]] || fail "real runtime source root is not a directory: $source_root"
  [[ -x "$executable" ]] || fail "real runtime executable is missing or not executable: $executable"

  local probe_output
  probe_output=$(mktemp)
  if ! "$executable" -noreadconfig -version >"$probe_output" 2>&1; then
    cat "$probe_output" >&2 || true
    rm -f "$probe_output"
    fail 'real runtime executable did not complete -noreadconfig -version successfully'
  fi
  local version_line
  version_line=$(head -n 1 "$probe_output" | tr -d '\r')
  rm -f "$probe_output"
  [[ -n "$version_line" ]] || fail 'real runtime version probe returned no output'
  case "$version_line" in
    *synthetic*|*Synthetic*|*MT-1305*)
      fail "real runtime version output looks synthetic: $version_line"
      ;;
  esac

  "$stager" "$source_root" "$executable" "$destination"
  write_provenance real "$destination" "$source_root" "$executable" "$version_line" true
  printf 'Prepared release-qualified bundled MAME runtime at %s (%s)\n' "$destination" "$version_line"
}

stage_synthetic_fixture() {
  [[ "${MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME:-}" == "1" ]] \
    || fail 'No real runtime configured. Set MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT and MAME_TAURI_REAL_RUNTIME_EXECUTABLE, or explicitly set MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME=1 for structural tests only.'

  local temp
  temp=$(mktemp -d)
  cleanup() {
    rm -rf "$temp"
  }
  trap cleanup RETURN

  mkdir -p \
    "$temp/source/hash" \
    "$temp/source/bgfx/chains" \
    "$temp/source/docs/legal"
  printf '<softwarelist name="fixture"/>\n' >"$temp/source/hash/fixture.xml"
  printf 'bgfx fixture\n' >"$temp/source/bgfx/chains/fixture.json"
  printf 'MAME COPYING fixture\n' >"$temp/source/COPYING"
  printf 'GPL fixture\n' >"$temp/source/docs/legal/GPL-2.0"
  cat >"$temp/mame" <<'EOF'
#!/usr/bin/env sh
if [ "${1:-}" = "-noreadconfig" ] && [ "${2:-}" = "-version" ]; then
  echo "MT-1305 synthetic payload (not release-qualified)"
  exit 0
fi
echo "MT-1305 synthetic payload"
EOF
  chmod 0755 "$temp/mame"

  "$stager" "$temp/source" "$temp/mame" "$destination"
  write_provenance synthetic-structural-fixture "$destination" "$temp/source" "$temp/mame" 'MT-1305 synthetic payload (not release-qualified)' false
  printf 'Prepared synthetic structural bundled MAME fixture at %s\n' "$destination"
}

if [[ -n "${MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT:-}" || -n "${MAME_TAURI_REAL_RUNTIME_EXECUTABLE:-}" ]]; then
  stage_real_runtime
else
  stage_synthetic_fixture
fi
