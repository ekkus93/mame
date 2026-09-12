#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
tauri_root="$repo_root/tauri"
runtime_root="$tauri_root/src-tauri/bundle-resources/mame-runtime"

fail() {
  echo "build-macos-release: $*" >&2
  exit 1
}

[[ "${APPLE_SIGNING_IDENTITY:-}" != "" ]] || fail "APPLE_SIGNING_IDENTITY is required for a release build"
[[ "${APPLE_SIGNING_IDENTITY}" != "-" ]] || fail "ad-hoc identity '-' is CI-smoke-only and is not a release signing identity"

api_complete=0
apple_id_complete=0
if [[ -n "${APPLE_API_ISSUER:-}" && -n "${APPLE_API_KEY:-}" && -n "${APPLE_API_KEY_PATH:-}" ]]; then
  [[ -f "$APPLE_API_KEY_PATH" ]] || fail "APPLE_API_KEY_PATH does not point to a readable private key"
  api_complete=1
fi
if [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]]; then
  apple_id_complete=1
fi
[[ $api_complete -eq 1 || $apple_id_complete -eq 1 ]] || \
  fail "notarization credentials are required: APPLE_API_ISSUER/APPLE_API_KEY/APPLE_API_KEY_PATH or APPLE_ID/APPLE_PASSWORD/APPLE_TEAM_ID"

[[ -x "$runtime_root/bin/mame" ]] || fail "qualified staged MAME executable is missing at $runtime_root/bin/mame"
[[ -d "$runtime_root/hash" ]] || fail "qualified staged MAME hash resources are missing"
[[ -d "$runtime_root/bgfx" ]] || fail "qualified staged MAME BGFX resources are missing"
[[ -s "$runtime_root/licenses/COPYING" ]] || fail "qualified staged MAME license notice is missing"
[[ -d "$runtime_root/licenses/legal" ]] || fail "qualified staged MAME legal resources are missing"

cd "$tauri_root"
npm ci
cargo test --locked bundled_runtime
npm run tauri -- build --bundles app,dmg

app_bundle=$(find src-tauri/target/release/bundle/macos -maxdepth 1 -type d -name '*.app' -print -quit)
dmg=$(find src-tauri/target/release/bundle/dmg -maxdepth 1 -type f -name '*.dmg' -print -quit)
[[ -n "$app_bundle" ]] || fail "release app bundle was not produced"
[[ -n "$dmg" ]] || fail "release DMG was not produced"

"$repo_root/scripts/tauri/test-macos-bundle.sh" "$app_bundle" "$dmg"

echo "Signed and notarization-enabled macOS release build completed; inspect Tauri/notary output before publication."
