#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: build-linux-bundled-mame-runtime.sh <destination>

Builds a real Linux MAME executable from this repository and stages it as the
Tauri bundled runtime. This script is for release/runtime qualification; fast
package-layout tests must use prepare-bundled-mame-runtime.sh with the explicit
synthetic-fixture opt-in instead.

Environment:
  MAME_TAURI_RUNTIME_MAKE_TARGET      make target to build, default: mame
  MAME_TAURI_RUNTIME_MAKE_JOBS        parallel make jobs, default: nproc
  MAME_TAURI_RUNTIME_MAKE_EXTRA_ARGS  extra make args, default: NOWERROR=1 TOOLS=0 TESTS=0
  MAME_TAURI_RUNTIME_EXECUTABLE       optional prebuilt executable override
EOF
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 64
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
destination=$1
prepare="$repo_root/scripts/tauri/prepare-bundled-mame-runtime.sh"

fail() {
  printf 'build-linux-bundled-mame-runtime: %s\n' "$*" >&2
  exit 1
}

cpu_count() {
  nproc 2>/dev/null || getconf _NPROCESSORS_ONLN 2>/dev/null || printf '2\n'
}

select_executable() {
  local candidate
  for candidate in "$@"; do
    if [[ -n "$candidate" && -x "$candidate" && -f "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

make_target=${MAME_TAURI_RUNTIME_MAKE_TARGET:-mame}
make_jobs=${MAME_TAURI_RUNTIME_MAKE_JOBS:-$(cpu_count)}
make_extra_args=${MAME_TAURI_RUNTIME_MAKE_EXTRA_ARGS:-NOWERROR=1 TOOLS=0 TESTS=0}

case "$make_jobs" in
  ''|*[!0-9]*) fail "MAME_TAURI_RUNTIME_MAKE_JOBS must be a positive integer: $make_jobs" ;;
esac
[[ "$make_jobs" -gt 0 ]] || fail "MAME_TAURI_RUNTIME_MAKE_JOBS must be positive: $make_jobs"

if [[ -n "${MAME_TAURI_RUNTIME_EXECUTABLE:-}" ]]; then
  executable=$(select_executable "$MAME_TAURI_RUNTIME_EXECUTABLE") \
    || fail "prebuilt runtime executable is missing or not executable: $MAME_TAURI_RUNTIME_EXECUTABLE"
else
  printf 'Building real MAME runtime: target=%s jobs=%s extra=%s\n' \
    "$make_target" "$make_jobs" "$make_extra_args"
  (
    cd "$repo_root"
    # MAME make arguments are intentionally passed as words because the upstream
    # build system is make-variable driven, for example: NOWERROR=1 TOOLS=0.
    # shellcheck disable=SC2086
    make -j "$make_jobs" $make_extra_args "$make_target"
  )
  executable=$(select_executable \
    "$repo_root/$make_target" \
    "$repo_root/mame" \
    "$repo_root/mame64") \
    || fail "could not find built MAME executable after make target $make_target"
fi

printf 'Staging real MAME runtime executable: %s\n' "$executable"
MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT="$repo_root" \
MAME_TAURI_REAL_RUNTIME_EXECUTABLE="$executable" \
  "$prepare" "$destination"
