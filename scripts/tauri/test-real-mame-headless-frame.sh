#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: test-real-mame-headless-frame.sh <real-mame-executable>

Launch the real MAME ___empty driver with -video none, capture a frame through
video:snapshot_pixels(), verify the composed snapshot dimensions/payload, and
confirm the headless process exits cleanly.
EOF
}

[[ $# -eq 1 ]] || { usage; exit 2; }
runtime=$(realpath "$1")
[[ -x "$runtime" ]] || { echo "real MAME executable is not executable: $runtime" >&2; exit 1; }

tmp=$(mktemp -d)
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

lua="$tmp/headless-frame.lua"
report="$tmp/headless-frame.txt"
stdout_log="$tmp/mame.stdout.log"
stderr_log="$tmp/mame.stderr.log"

cat >"$lua" <<'LUA'
local report_path = os.getenv("MAME_TAURI_HEADLESS_FRAME_REPORT")
local frame_count = 0
local finished = false

emu.register_frame_done(function ()
    if finished then return end
    frame_count = frame_count + 1
    if frame_count < 2 then return end

    local ok, width, height, pixels = pcall(function ()
        local snapshot_width, snapshot_height = manager.machine.video:snapshot_size()
        local data = manager.machine.video:snapshot_pixels()
        return snapshot_width, snapshot_height, data
    end)

    local report = io.open(report_path, "w")
    if report ~= nil then
        if ok and type(width) == "number" and type(height) == "number" and type(pixels) == "string" then
            local b1, b2, b3, b4 = string.byte(pixels, 1, 4)
            report:write("status=ok\n")
            report:write(string.format("frames=%d\n", frame_count))
            report:write(string.format("width=%d\n", width))
            report:write(string.format("height=%d\n", height))
            report:write(string.format("bytes=%d\n", #pixels))
            report:write(string.format("first_pixel_bytes=%02x%02x%02x%02x\n", b1 or 0, b2 or 0, b3 or 0, b4 or 0))
        else
            report:write("status=error\n")
            report:write(string.format("detail=%s\n", tostring(width)))
        end
        report:close()
    end

    finished = true
    manager.machine:exit()
end, "mame_tauri_headless_frame_qualification")
LUA

set +e
# SDL initializes its video subsystem before applying -video none. A headless
# CI runner has no DISPLAY, so provide a null SDL video backend explicitly.
# This is a smoke-test setting; it is not applied to desktop gameplay/audio.
timeout 30s env \
  SDL_VIDEODRIVER=dummy \
  MAME_TAURI_HEADLESS_FRAME_REPORT="$report" \
  "$runtime" ___empty \
    -video none \
    -sound none \
    -nothrottle \
    -skip_gameinfo \
    -autoboot_script "$lua" \
    >"$stdout_log" 2>"$stderr_log"
status=$?
set -e

if [[ $status -eq 124 ]]; then
  echo "real MAME headless frame smoke timed out" >&2
  cat "$stderr_log" >&2 || true
  exit 1
fi
if [[ $status -ne 0 ]]; then
  echo "real MAME headless frame smoke exited with status $status" >&2
  cat "$stderr_log" >&2 || true
  exit "$status"
fi
[[ -f "$report" ]] || {
  echo "real MAME headless frame smoke produced no capture report" >&2
  cat "$stdout_log" >&2 || true
  cat "$stderr_log" >&2 || true
  exit 1
}

status_value=$(sed -n 's/^status=//p' "$report")
width=$(sed -n 's/^width=//p' "$report")
height=$(sed -n 's/^height=//p' "$report")
bytes=$(sed -n 's/^bytes=//p' "$report")
frames=$(sed -n 's/^frames=//p' "$report")
first_pixel=$(sed -n 's/^first_pixel_bytes=//p' "$report")

[[ "$status_value" == "ok" ]] || {
  echo "real MAME headless frame capture failed:" >&2
  cat "$report" >&2
  exit 1
}
[[ "$width" =~ ^[0-9]+$ && "$height" =~ ^[0-9]+$ && "$bytes" =~ ^[0-9]+$ && "$frames" =~ ^[0-9]+$ ]] || {
  echo "real MAME headless frame report contains invalid numeric fields" >&2
  cat "$report" >&2
  exit 1
}
[[ "$width" -gt 0 && "$height" -gt 0 ]] || {
  echo "real MAME headless frame has invalid dimensions: $width x $height" >&2
  exit 1
}
expected_bytes=$((width * height * 4))
[[ "$bytes" -eq "$expected_bytes" ]] || {
  echo "real MAME headless frame payload mismatch: got $bytes expected $expected_bytes" >&2
  exit 1
}
[[ "$frames" -ge 2 ]] || {
  echo "real MAME headless frame callback did not advance" >&2
  exit 1
}
[[ "$first_pixel" =~ ^[0-9a-f]{8}$ ]] || {
  echo "real MAME headless frame first-pixel encoding is malformed: $first_pixel" >&2
  exit 1
}

printf 'Real MAME headless snapshot capture passed: %sx%s, %s bytes, frame %s, first pixel %s\n' \
  "$width" "$height" "$bytes" "$frames" "$first_pixel"
