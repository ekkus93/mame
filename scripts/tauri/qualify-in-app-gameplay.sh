#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  qualify-in-app-gameplay.sh <full.AppImage> <machine> [rom-directory] [output-directory]

Launches the exact full MAME Tauri AppImage for an interactive desktop gameplay
qualification and records reproducible artifact/runtime/environment evidence.

During the run:
  1. Verify the configured ROM path in Settings/Diagnostics.
  2. Audit and start the requested machine.
  3. Confirm gameplay is drawn inside the Tauri window with no separate MAME window.
  4. Exercise keyboard and a gamepad, native sound/mute, resize, fullscreen, stop, and relaunch.
  5. Open Diagnostics and save a diagnostics bundle before closing the application.

The script records AppImage SHA-256, bundled MAME identity/provenance, launch logs,
desktop environment details, and copies the newest diagnostics export created
during the run when one is available.
EOF
}

if [[ $# -lt 2 || $# -gt 4 ]]; then
  usage >&2
  exit 64
fi

appimage=$(realpath "$1")
machine=$2
rom_dir=$(realpath -m "${3:-${HOME}/mame/roms}")
timestamp=$(date -u +%Y%m%dT%H%M%SZ)
output_dir=${4:-"artifacts/in-app-gameplay/local-${timestamp}"}
mkdir -p "$output_dir"
output_dir=$(realpath "$output_dir")

[[ -f "$appimage" ]] || { echo "AppImage does not exist: $appimage" >&2; exit 1; }
[[ -x "$appimage" ]] || { echo "AppImage is not executable: $appimage" >&2; exit 1; }
[[ -d "$rom_dir" ]] || { echo "ROM directory does not exist: $rom_dir" >&2; exit 1; }

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
manifest="$output_dir/manifest.txt"
app_stdout="$output_dir/app.stdout.log"
app_stderr="$output_dir/app.stderr.log"
resource_log="$output_dir/resource-usage.txt"
note="$output_dir/qualification.md"
start_epoch=$(date +%s)

git_head=$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf 'unknown')
git_status=$(git -C "$repo_root" status --porcelain=v1 2>/dev/null || true)
appimage_sha256=$(sha256sum "$appimage" | awk '{print $1}')
appimage_size=$(stat -c '%s' "$appimage")
appimage_type=$(file -b "$appimage")

extract_root=$(mktemp -d)
cleanup() {
  rm -rf "$extract_root"
}
trap cleanup EXIT

(
  cd "$extract_root"
  "$appimage" --appimage-extract >/dev/null
)
app_root="$extract_root/squashfs-root"
runtime_bin=$(find "$app_root" -type f -path '*/mame-runtime/bin/mame' -print -quit)
[[ -n "$runtime_bin" && -x "$runtime_bin" ]] || {
  echo "The AppImage does not contain an executable mame-runtime/bin/mame." >&2
  exit 1
}
runtime_root=${runtime_bin%/bin/mame}
runtime_version=$("$runtime_bin" -noreadconfig -version | head -n 1)
runtime_provenance="$runtime_root/runtime-provenance.txt"

{
  printf 'qualification_timestamp_utc=%s\n' "$timestamp"
  printf 'repository_head=%s\n' "$git_head"
  printf 'repository_dirty=%s\n' "$([[ -n "$git_status" ]] && printf yes || printf no)"
  printf 'appimage_path=%s\n' "$appimage"
  printf 'appimage_sha256=%s\n' "$appimage_sha256"
  printf 'appimage_size_bytes=%s\n' "$appimage_size"
  printf 'appimage_file_type=%s\n' "$appimage_type"
  printf 'bundled_mame_version=%s\n' "$runtime_version"
  printf 'machine=%s\n' "$machine"
  printf 'rom_directory=%s\n' "$rom_dir"
  printf 'display=%s\n' "${DISPLAY:-}"
  printf 'wayland_display=%s\n' "${WAYLAND_DISPLAY:-}"
  printf 'xdg_session_type=%s\n' "${XDG_SESSION_TYPE:-}"
  printf 'desktop_session=%s\n' "${DESKTOP_SESSION:-}"
  printf 'host=%s\n' "$(uname -a)"
} >"$manifest"

if [[ -f "$runtime_provenance" ]]; then
  cp "$runtime_provenance" "$output_dir/runtime-provenance.txt"
fi
if [[ -n "$git_status" ]]; then
  printf '%s\n' "$git_status" >"$output_dir/repository-status.txt"
fi

cat >"$note" <<EOF
# MAME Tauri in-app gameplay qualification

- Qualification UTC: \`$timestamp\`
- Repository HEAD: \`$git_head\`
- AppImage: \`$appimage\`
- AppImage SHA-256: \`$appimage_sha256\`
- AppImage bytes: \`$appimage_size\`
- Bundled MAME: \`$runtime_version\`
- Machine: \`$machine\`
- ROM directory: \`$rom_dir\`
- Desktop: \`${XDG_SESSION_TYPE:-unknown}\` / \`${DESKTOP_SESSION:-unknown}\`
- Evidence directory: \`$output_dir\`

## Acceptance observations

Complete these observations during the launched AppImage session.

- [ ] The effective ROM path includes the intended local ROM directory and reports it readable.
- [ ] Audit and Start use the same effective ROM path.
- [ ] The selected machine reaches runtime-ready and first-frame-presented.
- [ ] Live gameplay is visible inside the Tauri window.
- [ ] No separate visible MAME gameplay window appears.
- [ ] Keyboard gameplay input works and browser/library shortcuts do not steal focused gameplay keys.
- [ ] A supported gamepad works, including buttons and axes.
- [ ] Blur/fullscreen/stop does not leave stuck keys or buttons.
- [ ] Native MAME sound is audible and mute/unmute reflects actual runtime state.
- [ ] Resize preserves aspect ratio and the selected nearest/smooth scaling behavior.
- [ ] Fullscreen enters/exits without restarting the MAME child.
- [ ] Stop returns cleanly and relaunch starts a fresh frame stream without stale video.
- [ ] Diagnostics were saved while gameplay was active or immediately after the run.
- [ ] The diagnostics video metrics show received and presented frames and bounded frame age/drop behavior.
- [ ] CPU/memory/emulation-speed observations are acceptable for this machine.

## Negative-path observations

- [ ] A known missing-content machine reports a content/audit diagnosis rather than only CONTROL_CHANNEL_CLOSED.
- [ ] An invalid or unreadable ROM path reports a path/content diagnosis.
- [ ] A no-first-frame condition is distinguishable from MAME startup failure.
- [ ] A post-start frame stall is distinguishable from process exit.

## Files

- \`manifest.txt\` — exact artifact/runtime/environment identity.
- \`runtime-provenance.txt\` — packaged runtime provenance when present.
- \`app.stdout.log\` / \`app.stderr.log\` — AppImage launcher/backend console output.
- \`diagnostics.json\` — copied application diagnostics export when saved during this run.
EOF

printf 'Evidence directory: %s\n' "$output_dir"
printf 'AppImage SHA-256: %s\n' "$appimage_sha256"
printf 'Bundled MAME: %s\n' "$runtime_version"
printf 'Machine: %s\n' "$machine"
printf 'ROM directory: %s\n' "$rom_dir"
printf '\nThe AppImage will now launch. Complete the checklist in the generated qualification note,\n'
printf 'save a Diagnostics bundle from the UI, then close the application to finish capture.\n\n'

set +e
if [[ -x /usr/bin/time ]]; then
  /usr/bin/time -v -o "$resource_log" "$appimage" >"$app_stdout" 2>"$app_stderr"
  app_status=$?
else
  "$appimage" >"$app_stdout" 2>"$app_stderr"
  app_status=$?
  printf '%s\n' 'resource_usage=unavailable (/usr/bin/time not installed)' >"$resource_log"
fi
set -e
printf 'app_exit_code=%s\n' "$app_status" >>"$manifest"

data_root=${XDG_DATA_HOME:-"$HOME/.local/share"}
diagnostics_candidate=
if [[ -d "$data_root" ]]; then
  diagnostics_candidate=$(
    find "$data_root" -maxdepth 6 -type f -path '*/support-diagnostics/diagnostics-*.json' -printf '%T@ %p\n' 2>/dev/null |
      sort -nr |
      awk -v start="$start_epoch" '$1 >= start { sub(/^[^ ]+ /, ""); print; exit }'
  )
fi
if [[ -n "$diagnostics_candidate" && -f "$diagnostics_candidate" ]]; then
  cp "$diagnostics_candidate" "$output_dir/diagnostics.json"
  printf 'diagnostics_source=%s\n' "$diagnostics_candidate" >>"$manifest"
  printf 'Copied diagnostics export: %s\n' "$diagnostics_candidate"
else
  printf 'diagnostics_source=not_found\n' >>"$manifest"
  printf 'No diagnostics export created during this run was found under %s.\n' "$data_root" >&2
fi

printf 'Qualification capture finished with application exit code %s.\n' "$app_status"
printf 'Review and complete: %s\n' "$note"
exit "$app_status"
