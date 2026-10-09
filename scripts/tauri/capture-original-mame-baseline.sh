#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'HELP'
Usage: capture-original-mame-baseline.sh <available-machine> [missing-content-machine] [output-directory]

Capture the original installed MAME executable and effective configuration
under the current user's normal HOME/XDG settings. Runs noninteractive ROM
audits only; never starts a gameplay window or changes configuration.

Choose an actually installed machine for the first argument, and optionally a
known MAME machine whose content is absent for the second. Do not point PATH
at an extracted AppImage runtime: this is an independent installed-MAME baseline.
HELP
}

if [[ $# -lt 1 || $# -gt 3 ]]; then usage >&2; exit 64; fi
available=$1
missing=${2:-}
for machine in "$available" "$missing"; do
  [[ -z "$machine" || "$machine" =~ ^[a-zA-Z0-9_][a-zA-Z0-9_.-]*$ ]] || {
    echo "Invalid MAME short name: $machine" >&2; exit 64;
  }
done

if ! command -v mame >/dev/null 2>&1; then
  echo 'No installed mame executable was found on PATH; cannot capture original-MAME baseline.' >&2
  exit 1
fi
mame_bin=$(realpath "$(command -v mame)")
[[ -x "$mame_bin" ]] || { echo "Installed MAME is not executable: $mame_bin" >&2; exit 1; }

stamp=$(date -u +%Y%m%dT%H%M%SZ)
output_dir=${3:-"artifacts/in-app-gameplay/original-mame-${stamp}"}
mkdir -p "$output_dir"
output_dir=$(realpath "$output_dir")
manifest="$output_dir/original-mame-manifest.txt"

{
  printf 'captured_at_utc=%s\n' "$stamp"
  printf 'installed_mame_path=%s\n' "$mame_bin"
  printf 'installed_mame_sha256=%s\n' "$(sha256sum "$mame_bin" | awk '{print $1}')"
  printf 'home=%s\n' "${HOME:-}"
  printf 'xdg_config_home=%s\n' "${XDG_CONFIG_HOME:-}"
  printf 'xdg_data_home=%s\n' "${XDG_DATA_HOME:-}"
  printf 'available_machine=%s\n' "$available"
  printf 'missing_content_machine=%s\n' "${missing:-not_tested}"
} >"$manifest"

capture() {
  local label=$1
  shift
  local code=0
  timeout 120s "$mame_bin" "$@" >"$output_dir/$label.stdout.txt" 2>"$output_dir/$label.stderr.txt" || code=$?
  printf '%s_exit_code=%s\n' "$label" "$code" >>"$manifest"
  printf '%s_command=' "$label" >>"$manifest"
  printf '%q ' "$mame_bin" "$@" >>"$manifest"
  printf '\n' >>"$manifest"
}

capture version -version
capture showconfig -showconfig
if [[ -s "$output_dir/showconfig.stdout.txt" ]]; then
  grep -E '^[[:space:]]*(rompath|inipath|cfg_directory|nvram_directory)[[:space:]]' \
    "$output_dir/showconfig.stdout.txt" >"$output_dir/effective-path-settings.txt" || true
fi
capture available_verifyroms -verifyroms "$available"
if [[ -n "$missing" ]]; then capture missing_verifyroms -verifyroms "$missing"; fi

cat >"$output_dir/README.txt" <<'HELP'
These files describe the ORIGINAL installed MAME runtime, using the normal
user profile. They are not evidence that the Tauri AppImage displays gameplay.
Compare effective-path-settings.txt and audit output against Tauri's exported
diagnostics for the same machine and content. A missing-content audit is
expected to exit nonzero. An available-machine audit must be interpreted from
its output and exit code, not just the machine name. Do not upload this folder
publicly: configuration and diagnostics can reveal local file paths.
HELP

printf 'Original installed-MAME baseline saved to: %s\n' "$output_dir"
printf 'MAME: %s\n' "$mame_bin"
printf 'Review: %s\n' "$manifest"
