#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: test-macos-bundle.sh <app-bundle> <dmg>" >&2
  exit 64
fi

app_bundle=$1
dmg=$2

validate_app() {
  local app=$1
  local plist="$app/Contents/Info.plist"
  local resources="$app/Contents/Resources/mame-runtime"
  local executable

  [[ -d "$app" ]] || { echo "Missing app bundle: $app" >&2; return 1; }
  [[ -f "$plist" ]] || { echo "Missing Info.plist: $plist" >&2; return 1; }

  executable=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$plist")
  [[ -n "$executable" ]] || { echo "CFBundleExecutable is empty" >&2; return 1; }
  [[ -x "$app/Contents/MacOS/$executable" ]] || {
    echo "Missing executable: $app/Contents/MacOS/$executable" >&2
    return 1
  }

  [[ -x "$resources/bin/mame" ]] || { echo "Missing bundled MAME executable" >&2; return 1; }
  [[ -f "$resources/hash/fixture.xml" ]] || { echo "Missing bundled hash fixture" >&2; return 1; }
  [[ -f "$resources/bgfx/chains/fixture.json" ]] || { echo "Missing bundled BGFX fixture" >&2; return 1; }
  [[ -s "$resources/licenses/COPYING" ]] || { echo "Missing bundled COPYING notice" >&2; return 1; }
  [[ -f "$resources/licenses/legal/GPL-2.0" ]] || { echo "Missing bundled legal fixture" >&2; return 1; }

  codesign --verify --deep --strict --verbose=2 "$app"
}

[[ -f "$dmg" ]] || { echo "Missing DMG: $dmg" >&2; exit 1; }
validate_app "$app_bundle"

mount_dir=$(mktemp -d)
install_root=$(mktemp -d)
attached=0
cleanup() {
  if [[ $attached -eq 1 ]]; then
    hdiutil detach "$mount_dir" -quiet || true
  fi
  rm -rf "$mount_dir" "$install_root"
}
trap cleanup EXIT

hdiutil attach "$dmg" -nobrowse -readonly -mountpoint "$mount_dir" -quiet
attached=1

mounted_app=$(find "$mount_dir" -maxdepth 1 -type d -name '*.app' -print -quit)
[[ -n "$mounted_app" ]] || { echo "DMG does not contain an application bundle" >&2; exit 1; }
validate_app "$mounted_app"

mkdir -p "$install_root/Applications"
installed_app="$install_root/Applications/$(basename "$mounted_app")"
ditto "$mounted_app" "$installed_app"
validate_app "$installed_app"

hdiutil detach "$mount_dir" -quiet
attached=0

echo "MT-1304 macOS app, DMG, packaged runtime, relocation, and code-sign smoke passed"
