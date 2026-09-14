#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root/tauri"

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

for _ in $(seq 1 180); do
  if ! kill -0 "$child_pid" 2>/dev/null; then
    cat "$log_file"
    echo "Tauri development process exited before creating the main window." >&2
    exit 1
  fi

  if xdotool search --name "MAME Tauri Frontend" >/dev/null 2>&1; then
    echo "Observed MAME Tauri Frontend window under Xvfb; verifying backend survival."
    for _ in $(seq 1 20); do
      sleep 0.25
      if ! kill -0 "$child_pid" 2>/dev/null; then
        cat "$log_file"
        echo "Tauri development process exited during the post-window startup grace period." >&2
        exit 1
      fi
    done
    if grep -Eq "panicked at|Failed to setup app" "$log_file"; then
      cat "$log_file"
      echo "Tauri development process logged a startup panic after creating the main window." >&2
      exit 1
    fi
    echo "MAME Tauri Frontend window remained alive through the startup grace period."
    exit 0
  fi

  sleep 1
done

cat "$log_file"
echo "Timed out waiting for the MAME Tauri Frontend window." >&2
exit 1
