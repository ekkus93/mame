#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
stager="$repo_root/scripts/tauri/stage-mame-runtime.sh"

temp=$(mktemp -d)
cleanup() {
  chmod -R u+w "$temp" 2>/dev/null || true
  rm -rf "$temp"
}
trap cleanup EXIT

source_root="$temp/source"
destination="$temp/output/mame-runtime"
mkdir -p \
  "$source_root/hash" \
  "$source_root/bgfx/chains" \
  "$source_root/docs/legal"
printf '<softwarelist name="fixture"/>\n' > "$source_root/hash/fixture.xml"
printf 'shader fixture\n' > "$source_root/bgfx/chains/fixture.json"
printf 'MAME copying fixture\n' > "$source_root/COPYING"
printf 'GPL fixture\n' > "$source_root/docs/legal/GPL-2.0"
cat > "$temp/mame" <<'EOF'
#!/usr/bin/env sh
if [ "${1:-}" = "-noreadconfig" ] && [ "${2:-}" = "-version" ]; then
  echo "0.288 structural runtime fixture"
  exit 0
fi
exit 0
EOF
chmod 0755 "$temp/mame"

"$stager" "$source_root" "$temp/mame" "$destination"

test -x "$destination/bin/mame"
test -f "$destination/hash/fixture.xml"
test -f "$destination/bgfx/chains/fixture.json"
test -s "$destination/licenses/COPYING"
test -f "$destination/licenses/legal/GPL-2.0"

chmod -R a-w "$destination"
arbitrary_cwd="$temp/arbitrary-cwd"
mkdir -p "$arbitrary_cwd"
(
  cd "$arbitrary_cwd"
  "$destination/bin/mame" -noreadconfig -version
) | grep -Fxq '0.288 structural runtime fixture'
chmod -R u+w "$destination"

printf 'preserve me\n' > "$destination/sentinel"
invalid_source="$temp/invalid-source"
mkdir -p "$invalid_source/hash" "$invalid_source/docs/legal"
printf '<softwarelist/>\n' > "$invalid_source/hash/fixture.xml"
printf 'copying\n' > "$invalid_source/COPYING"
printf 'license\n' > "$invalid_source/docs/legal/GPL-2.0"

if "$stager" "$invalid_source" "$temp/mame" "$destination" >/dev/null 2>&1; then
  printf 'Expected staging with missing BGFX resources to fail\n' >&2
  exit 1
fi

test -f "$destination/sentinel"
printf 'MT-1302 runtime staging contract passed\n'
