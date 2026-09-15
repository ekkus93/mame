#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root/tauri"

profile_root="$(mktemp -d)"
log_file="$(mktemp)"
artifact_dir="$repo_root/artifacts/mame-ui-remediation"
mkdir -p "$artifact_dir"
rm -f "$artifact_dir"/*.png

export XDG_CONFIG_HOME="$profile_root/config"
export XDG_DATA_HOME="$profile_root/data"
export XDG_CACHE_HOME="$profile_root/cache"
app_config_dir="$XDG_CONFIG_HOME/io.github.ekkus93.mame-tauri"
mkdir -p "$app_config_dir" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"

child_pid=""
window_id=""

stop_app() {
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM -- "-$child_pid" 2>/dev/null || true
    for _ in $(seq 1 30); do
      if ! kill -0 "$child_pid" 2>/dev/null; then
        break
      fi
      sleep 0.2
    done
    kill -KILL -- "-$child_pid" 2>/dev/null || true
    wait "$child_pid" 2>/dev/null || true
  fi
  child_pid=""
  window_id=""
}

cleanup() {
  stop_app
  rm -rf "$profile_root"
  rm -f "$log_file"
}
trap cleanup EXIT

verify_alive() {
  local context="$1"
  if [[ -z "$child_pid" ]] || ! kill -0 "$child_pid" 2>/dev/null; then
    cat "$log_file"
    echo "Tauri development process exited during ${context}." >&2
    exit 1
  fi
  if grep -Eq "panicked at|Failed to setup app" "$log_file"; then
    cat "$log_file"
    echo "Tauri development process logged a startup panic during ${context}." >&2
    exit 1
  fi
}

start_app() {
  : >"$log_file"
  setsid npm run tauri -- dev --no-watch >"$log_file" 2>&1 &
  child_pid=$!
  for _ in $(seq 1 180); do
    verify_alive "visual-smoke startup"
    window_id="$(xdotool search --name "MAME Tauri Frontend" 2>/dev/null | head -n 1 || true)"
    if [[ -n "$window_id" ]]; then
      xdotool windowfocus "$window_id" 2>/dev/null || true
      sleep 1
      return
    fi
    sleep 1
  done
  cat "$log_file"
  echo "Timed out waiting for visual-smoke Tauri window." >&2
  exit 1
}

color_count() {
  local image="$1"
  local color="$2"
  convert "$image" -format "%c" histogram:info:- | awk -v wanted="#${color}" '
    index(toupper($0), toupper(wanted)) {
      value=$1
      gsub(":", "", value)
      total += value
    }
    END { print total + 0 }
  '
}

require_color() {
  local image="$1"
  local color="$2"
  local label="$3"
  local minimum="${4:-20}"
  local count
  count="$(color_count "$image" "$color")"
  if (( count < minimum )); then
    echo "Expected at least ${minimum} pixels of ${label} (#${color}) in ${image}; found ${count}." >&2
    exit 1
  fi
  echo "Rendered ${label} (#${color}) pixels: ${count}"
}

capture_window() {
  local name="$1"
  local width="$2"
  local height="$3"
  local selected_palette="${4:-no}"
  local image="$artifact_dir/${name}.png"

  xdotool windowsize --sync "$window_id" "$width" "$height"
  sleep 0.8
  verify_alive "${name} capture"

  local geometry actual_width actual_height
  geometry="$(xdotool getwindowgeometry --shell "$window_id")"
  actual_width="$(printf '%s\n' "$geometry" | awk -F= '$1 == "WIDTH" {print $2}')"
  actual_height="$(printf '%s\n' "$geometry" | awk -F= '$1 == "HEIGHT" {print $2}')"
  if [[ "$actual_width" != "$width" || "$actual_height" != "$height" ]]; then
    printf '%s\n' "$geometry" >&2
    echo "Window geometry mismatch for ${name}: expected ${width}x${height}." >&2
    exit 1
  fi

  import -window "$window_id" "$image"
  identify "$image"

  require_color "$image" "090B2C" "deep navy"
  require_color "$image" "174F7A" "structural blue"
  require_color "$image" "FFED19" "yellow accent"
  require_color "$image" "087119" "green status bar"
  if [[ "$selected_palette" == "yes" ]]; then
    require_color "$image" "00B7F4" "selected cyan" 5
    require_color "$image" "0075C9" "selected blue" 5
  fi

  local thumbnail
  thumbnail="$(convert "$image" -resize 320x180 -strip -quality 42 jpg:- | base64 -w 0)"
  echo "MAME_UI_THUMBNAIL::${name}::${thumbnail}"
}

wait_for_active_catalog() {
  local catalog=""
  for _ in $(seq 1 100); do
    catalog="$(find "$XDG_DATA_HOME" -name catalog.sqlite3 -type f -print -quit 2>/dev/null || true)"
    if [[ -n "$catalog" ]] && python3 - "$catalog" <<'PY'
import sqlite3
import sys

path = sys.argv[1]
try:
    with sqlite3.connect(path) as connection:
        row = connection.execute(
            "SELECT machine_count FROM metadata_generation WHERE active = 1"
        ).fetchone()
except sqlite3.Error:
    raise SystemExit(1)
raise SystemExit(0 if row and row[0] > 0 else 1)
PY
    then
      echo "Observed active imported catalog: ${catalog}"
      return
    fi
    sleep 0.2
  done
  cat "$log_file"
  echo "Timed out waiting for active imported metadata generation." >&2
  exit 1
}

# State 1: fresh install. The primary workspace must be explicit rather than a fake zero-machine list.
start_app
capture_window "not-configured-1366x768" 1366 768 no
stop_app

# Configure a deterministic fake MAME that exposes the checked-in representative -listxml fixture.
fake_mame="$profile_root/fake-mame"
fixture="$repo_root/tauri/tests/fixtures/listxml-representative.xml"
cat >"$fake_mame" <<EOF
#!/bin/sh
if [ "\$1" = '-noreadconfig' ] && [ "\$2" = '-version' ]; then
  printf '%s\\n' '0.288 test-fixture'
  exit 0
fi
if [ "\$1" = '-noreadconfig' ] && [ "\$2" = '-listxml' ]; then
  cat '$fixture'
  exit 0
fi
exit 99
EOF
chmod 755 "$fake_mame"
mkdir -p "$app_config_dir"
python3 - "$app_config_dir/settings.json" "$fake_mame" <<'PY'
import json
import sys

path, executable = sys.argv[1:]
with open(path, "w", encoding="utf-8") as handle:
    json.dump({"schemaVersion": 2, "mameExecutable": executable}, handle)
    handle.write("\n")
PY

# State 2: valid executable, but no active generation. The Import action is autofocused for keyboard use.
start_app
capture_window "metadata-missing-1366x768" 1366 768 no
xdotool windowfocus "$window_id" 2>/dev/null || true
xdotool key --window "$window_id" Return
wait_for_active_catalog
sleep 1.5
verify_alive "post-import ready transition"

# State 3: populated Images view at every required desktop size.
capture_window "populated-images-1920x1080" 1920 1080 yes
capture_window "populated-images-1366x768" 1366 768 yes
capture_window "populated-images-1280x720" 1280 720 yes
stop_app

# State 4: persisted Info mode over the same active catalog.
cat >"$app_config_dir/mame-ui-state.json" <<'EOF'
{
  "schemaVersion": 1,
  "lastMachine": "galaxian",
  "filter": "all",
  "filterValue": null,
  "rightPanelMode": "info",
  "artworkKind": "screenshot",
  "softwareRightPanelMode": "images",
  "softwareArtworkKind": "screenshot"
}
EOF
start_app
capture_window "populated-info-1366x768" 1366 768 yes
stop_app

echo "MAME UI post-closure visual remediation smoke passed."
