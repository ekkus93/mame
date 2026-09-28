#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
preparer="$repo_root/scripts/tauri/prepare-bundled-mame-runtime.sh"

temp=$(mktemp -d)
cleanup() {
  rm -rf "$temp"
}
trap cleanup EXIT

if "$preparer" "$temp/without-runtime" >/dev/null 2>&1; then
  printf 'Expected preparation without real runtime inputs or explicit synthetic opt-in to fail\n' >&2
  exit 1
fi

synthetic_dest="$temp/synthetic/mame-runtime"
MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME=1 "$preparer" "$synthetic_dest"
test -x "$synthetic_dest/bin/mame"
test -f "$synthetic_dest/hash/fixture.xml"
test -f "$synthetic_dest/bgfx/chains/fixture.json"
test -f "$synthetic_dest/licenses/COPYING"
test -f "$synthetic_dest/licenses/legal/GPL-2.0"
test -f "$synthetic_dest/runtime-provenance.txt"
grep -Fxq 'mode=synthetic-structural-fixture' "$synthetic_dest/runtime-provenance.txt"
grep -Fxq 'release_qualified=false' "$synthetic_dest/runtime-provenance.txt"

real_source="$temp/real-source"
real_dest="$temp/real-output/mame-runtime"
mkdir -p \
  "$real_source/hash" \
  "$real_source/bgfx/chains" \
  "$real_source/docs/legal"
printf '<softwarelist name="realfixture"/>\n' >"$real_source/hash/fixture.xml"
printf 'real bgfx fixture\n' >"$real_source/bgfx/chains/fixture.json"
printf 'real copying\n' >"$real_source/COPYING"
printf 'real GPL\n' >"$real_source/docs/legal/GPL-2.0"
cat >"$temp/real-mame" <<'EOF'
#!/usr/bin/env sh
if [ "${1:-}" = "-noreadconfig" ] && [ "${2:-}" = "-version" ]; then
  echo "0.288 test-real-runtime"
  exit 0
fi
exit 1
EOF
chmod 0755 "$temp/real-mame"

MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT="$real_source" \
MAME_TAURI_REAL_RUNTIME_EXECUTABLE="$temp/real-mame" \
  "$preparer" "$real_dest"
test -x "$real_dest/bin/mame"
test -f "$real_dest/runtime-provenance.txt"
grep -Fxq 'mode=real' "$real_dest/runtime-provenance.txt"
grep -Fxq 'release_qualified=true' "$real_dest/runtime-provenance.txt"
grep -Fxq 'mame_version_line=0.288 test-real-runtime' "$real_dest/runtime-provenance.txt"

if MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT="$real_source" \
  MAME_TAURI_REAL_RUNTIME_EXECUTABLE="$synthetic_dest/bin/mame" \
  "$preparer" "$temp/rejected-synthetic-real" >/dev/null 2>&1; then
  printf 'Expected real staging to reject synthetic-looking version output\n' >&2
  exit 1
fi

printf 'BMR bundled runtime preparation contract passed\n'
