#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 4 ]]; then
  echo 'Usage: qualify-mame-frame-seams.sh <mame> <machine> [rom-dir] [output-report]' >&2
  exit 64
fi

runtime=$(realpath "$1")
machine=$2
rom_dir=""
if [[ $# -ge 3 && -n "$3" ]]; then rom_dir=$(realpath -m "$3"); fi
if [[ $# -ge 4 && -n "$4" ]]; then
  output=$4
else
  output="artifacts/in-app-gameplay/frame-seam-$machine-$(date -u +%Y%m%dT%H%M%SZ).txt"
fi
mkdir -p "$(dirname "$output")"
output=$(realpath -m "$output")
[[ -x "$runtime" ]] || { echo "MAME executable is not executable: $runtime" >&2; exit 1; }
[[ -z "$rom_dir" || -d "$rom_dir" ]] || { echo "ROM directory does not exist: $rom_dir" >&2; exit 1; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
lua="$tmp/frame-seam.lua"

cat >"$lua" <<'LUA'
local report_path = os.getenv("MAME_TAURI_FRAME_SEAM_REPORT")
local machine_name = os.getenv("MAME_TAURI_FRAME_SEAM_MACHINE") or "unknown"
local warmup = tonumber(os.getenv("MAME_TAURI_FRAME_SEAM_WARMUP") or "30")
local samples = tonumber(os.getenv("MAME_TAURI_FRAME_SEAM_SAMPLES") or "120")
local frame_count, sample_count = 0, 0
local screen_cpu, snapshot_cpu = 0.0, 0.0
local _, screen = next(manager.machine.screens)
local sw, sh, vw, vh, sb, vb = 0, 0, 0, 0, 0, 0
local screen_first, snapshot_first = "none", "none"
local done = false

local function quad(data)
    if type(data) ~= "string" or #data < 4 then return "none" end
    local a,b,c,d = string.byte(data, 1, 4)
    return string.format("%02x%02x%02x%02x", a or 0, b or 0, c or 0, d or 0)
end

local function finish(status, detail)
    if done then return end
    done = true
    local f = io.open(report_path, "w")
    if f then
        f:write(string.format("status=%s\n", status))
        f:write(string.format("machine=%s\n", machine_name))
        f:write(string.format("samples=%d\n", sample_count))
        if detail then f:write(string.format("detail=%s\n", tostring(detail))) end
        f:write(string.format("screen=%dx%d bytes=%d first=%s\n", sw, sh, sb, screen_first))
        f:write(string.format("snapshot=%dx%d bytes=%d first=%s\n", vw, vh, vb, snapshot_first))
        if sample_count > 0 then
            f:write(string.format("screen_cpu_average_us=%.3f\n", screen_cpu * 1000000.0 / sample_count))
            f:write(string.format("snapshot_cpu_average_us=%.3f\n", snapshot_cpu * 1000000.0 / sample_count))
        end
        f:close()
    end
    manager.machine:exit()
end

emu.register_frame_done(function ()
    if done then return end
    frame_count = frame_count + 1
    if frame_count <= warmup then return end
    if screen == nil then finish("error", "machine has no screen"); return end

    local t = os.clock()
    local ok_s, sp, x, y = pcall(function ()
        local p, w, h = screen:pixels()
        return p, w, h
    end)
    screen_cpu = screen_cpu + (os.clock() - t)

    t = os.clock()
    local ok_v, x2, y2, vp = pcall(function ()
        local w, h = manager.machine.video:snapshot_size()
        return w, h, manager.machine.video:snapshot_pixels()
    end)
    snapshot_cpu = snapshot_cpu + (os.clock() - t)

    if not ok_s then finish("error", sp); return end
    if not ok_v then finish("error", x2); return end
    sw, sh, sb = math.floor(x), math.floor(y), #sp
    vw, vh, vb = math.floor(x2), math.floor(y2), #vp
    if sb ~= sw * sh * 4 then finish("error", "screen payload mismatch"); return end
    if vb ~= vw * vh * 4 then finish("error", "snapshot payload mismatch"); return end
    sample_count = sample_count + 1
    if screen_first == "none" then screen_first = quad(sp) end
    if snapshot_first == "none" then snapshot_first = quad(vp) end
    if sample_count >= samples then finish("ok", nil) end
end, "mame_tauri_frame_seam_qualification")
LUA

cmd=("$runtime" "$machine" -video none -nothrottle -skip_gameinfo -autoboot_script "$lua")
if [[ -n "$rom_dir" ]]; then cmd+=(-rompath "$rom_dir"); fi

set +e
MAME_TAURI_FRAME_SEAM_REPORT="$output" MAME_TAURI_FRAME_SEAM_MACHINE="$machine" timeout 120s "${cmd[@]}" >"$output.stdout.log" 2>"$output.stderr.log"
status=$?
set -e
[[ $status -eq 0 ]] || { echo "MAME frame-seam qualification failed with status $status" >&2; tail -n 120 "$output.stderr.log" >&2 || true; exit $status; }
grep -q '^status=ok$' "$output" || { cat "$output" >&2; exit 1; }
printf 'Frame-seam qualification report: %s\n' "$output"
cat "$output"
