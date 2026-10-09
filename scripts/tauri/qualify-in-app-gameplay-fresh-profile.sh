#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  qualify-in-app-gameplay-fresh-profile.sh <full.AppImage> <machine> [rom-directory] [output-directory]

Runs the full AppImage gameplay qualification using a brand-new temporary HOME,
XDG_CONFIG_HOME, XDG_DATA_HOME, XDG_STATE_HOME and XDG_CACHE_HOME. The normal ROM directory is
made visible at the fresh profile's ~/mame/roms via a symlink; no ROMs are copied.
The temporary profile is removed after the AppImage exits, while the evidence
directory and exported diagnostics are retained.

Run from a real Linux desktop session with DISPLAY or WAYLAND_DISPLAY set.
This is an interactive review, not an automated claim that gameplay works.
EOF
}

if [[ $# -lt 2 || $# -gt 4 ]]; then
  usage >&2
  exit 64
fi
if [[ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]]; then
  echo "A real desktop session is required (DISPLAY or WAYLAND_DISPLAY)." >&2
  exit 1
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
appimage=$(realpath "$1")
machine=$2
rom_dir=$(realpath -m "${3:-${HOME:?}/mame/roms}")
timestamp=$(date -u +%Y%m%dT%H%M%SZ)
output_dir=${4:-"artifacts/in-app-gameplay/fresh-profile-${timestamp}"}
mkdir -p "$output_dir"
output_dir=$(realpath "$output_dir")

[[ -f "$appimage" && -x "$appimage" ]] || {
  echo "Full AppImage is missing or not executable: $appimage" >&2
  exit 1
}
[[ -d "$rom_dir" && -r "$rom_dir" ]] || {
  echo "ROM directory is missing or unreadable: $rom_dir" >&2
  exit 1
}

profile_root=$(mktemp -d)
cleanup() { rm -rf -- "$profile_root"; }
trap cleanup EXIT
mkdir -p "$profile_root/home/mame" "$profile_root/config" "$profile_root/data" "$profile_root/state" "$profile_root/cache"
ln -s -- "$rom_dir" "$profile_root/home/mame/roms"

{
  printf 'profile_mode=isolated_fresh_home_and_xdg\n'
  printf 'profile_created_utc=%s\n' "$timestamp"
  printf 'profile_home=%s\n' "$profile_root/home"
  printf 'default_rompath=%s\n' "$profile_root/home/mame/roms"
  printf 'xdg_state_home=%s\n' "$profile_root/state"
  printf 'rompath_symlink_target=%s\n' "$rom_dir"
  printf 'existing_user_config_used=no\n'
  printf 'desktop_display=%s\n' "${DISPLAY:-}"
  printf 'desktop_wayland_display=%s\n' "${WAYLAND_DISPLAY:-}"
} >"$output_dir/fresh-profile-manifest.txt"

cat >>"$output_dir/fresh-profile-instructions.txt" <<'EOF'
This is a fresh-profile first-run review. No pre-existing catalog, audit, or
application settings should be present. Confirm that the first-run UI is usable,
the conventional ~/mame/roms path appears without manual configuration, catalog
metadata is available without a manual metadata-import step, and audit and
Start agree on the effective content paths. Then complete qualification.md
inside the actual AppImage and export Diagnostics before closing it.
The ROM symlink is read-only with respect to path setup, but MAME itself can
access files in the real ROM directory; do not use writable test media.
EOF

printf 'Isolated first-run profile: %s\n' "$profile_root/home"
printf 'Persistent evidence directory: %s\n' "$output_dir"

env \
  HOME="$profile_root/home" \
  XDG_CONFIG_HOME="$profile_root/config" \
  XDG_DATA_HOME="$profile_root/data" \
  XDG_STATE_HOME="$profile_root/state" \
  XDG_CACHE_HOME="$profile_root/cache" \
  "$repo_root/scripts/tauri/qualify-in-app-gameplay.sh" \
    "$appimage" "$machine" "$rom_dir" "$output_dir"
