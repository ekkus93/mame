#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root/tauri"

# Prove the window we are about to smoke is the MAME-style shell rather than a
# surviving legacy composition. The regression also owns the fixed-document
# overflow contract; xdotool is intentionally used only for stable native-window
# behavior rather than brittle GTK/WebKit pixel-coordinate assertions.
python3 "$repo_root/scripts/tauri/test-mame-ui-reproduction.py"

log_file="$(mktemp)"
child_pid=""

cleanup() {
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM -- "-$child_pid" 2>/dev/null || true
    for _ in $(seq 1 20); do
      if ! kill -0 "$child_pid" 2>/dev/null; then
        break
      fi
      sleep 0.25
    done
    kill -KILL -- "-$child_pid" 2>/dev/null || true
    wait "$child_pid" 2>/dev/null || true
  fi
  rm -f "$log_file"
}
trap cleanup EXIT

setsid npm run tauri -- dev --no-watch >"$log_file" 2>&1 &
child_pid=$!

verify_alive() {
  local context="$1"
  if ! kill -0 "$child_pid" 2>/dev/null; then
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

qualify_window_sizes() {
  local window_id="$1"
  local width height geometry
  for geometry in 1920x1080 1366x768 1280x720; do
    width="${geometry%x*}"
    height="${geometry#*x}"
    xdotool windowsize --sync "$window_id" "$width" "$height"
    sleep 0.5
    verify_alive "${geometry} viewport qualification"
    echo "Qualified live MAME Tauri window at ${geometry}:"
    xdotool getwindowgeometry --shell "$window_id"
  done
}

for _ in $(seq 1 180); do
  verify_alive "startup"

  window_id="$(xdotool search --name "MAME Tauri Frontend" 2>/dev/null | head -n 1 || true)"
  if [[ -n "$window_id" ]]; then
    echo "Observed MAME Tauri Frontend window under Xvfb; verifying backend survival."
    for _ in $(seq 1 20); do
      sleep 0.25
      verify_alive "post-window startup grace period"
    done

    qualify_window_sizes "$window_id"
    echo "MAME Tauri Frontend remained alive through startup and required desktop-size qualification."
    exit 0
  fi

  sleep 1
done

cat "$log_file"
echo "Timed out waiting for the MAME Tauri Frontend window." >&2
exit 1
