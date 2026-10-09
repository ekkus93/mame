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
lua="$tmp/boot.lua"

cat >"$lua" <<'LUA'
local report_path = assert(os.getenv("MAME_TAURI_FRAME_SEAM_REPORT"), "missing frame seam report path")
local machine_name = os.getenv("MAME_TAURI_FRAME_SEAM_MACHINE") or "unknown"
local warmup = math.max(0, math.floor(tonumber(os.getenv("MAME_TAURI_FRAME_SEAM_WARMUP")) or 30))
local samples = math.max(1, math.floor(tonumber(os.getenv("MAME_TAURI_FRAME_SEAM_SAMPLES")) or 120))
local frame_count, sample_count, screen_samples = 0, 0, 0
local screen_cpu, snapshot_cpu = 0.0, 0.0
local screen_max_cpu, snapshot_max_cpu = 0.0, 0.0
local screen = nil
local screen_checked = false
local sw, sh, vw, vh, sb, vb = 0, 0, 0, 0, 0, 0
local screen_first, snapshot_first = "none", "none"
local screen_error = nil
local done = false

local function quad(data)
    if type(data) ~= "string" or #data < 4 then return "none" end
    local a,b,c,d = string.byte(data, 1, 4)
    return string.format("%02x%02x%02x%02x", a or 0, b or 0, c or 0, d or 0)
end

local function validate_pixels(data, width, height)
    if type(data) ~= "string" or type(width) ~= "number" or type(height) ~= "number" then
        return false, "non-string pixels or non-numeric dimensions"
    end
    if width < 1 or height < 1 or width % 1 ~= 0 or height % 1 ~= 0 then
        return false, "invalid dimensions"
    end
    if #data ~= width * height * 4 then return false, "payload length differs from width*height*4" end
    return true, nil
end

local function finish(status, detail)
    if done then return end
    done = true
    local f = io.open(report_path, "w")
    if f then
        f:write(string.format("status=%s\n", status))
        f:write(string.format("machine=%s\n", machine_name))
        f:write(string.format("warmup_frames=%d\n", warmup))
        f:write(string.format("callback_frames=%d\n", frame_count))
        f:write(string.format("samples=%d\n", sample_count))
        if detail then f:write(string.format("detail=%s\n", tostring(detail))) end
        f:write(string.format("screen_status=%s\n", screen_error and "unsupported" or "ok"))
        if screen_error then f:write(string.format("screen_error=%s\n", tostring(screen_error))) end
        f:write(string.format("screen_samples=%d\n", screen_samples))
        f:write(string.format("screen=%dx%d bytes=%d first=%s\n", sw, sh, sb, screen_first))
        f:write(string.format("snapshot_status=%s\n", status == "ok" and "ok" or "error"))
        f:write(string.format("snapshot=%dx%d bytes=%d first=%s\n", vw, vh, vb, snapshot_first))
        if screen_samples > 0 then
            f:write(string.format("screen_cpu_average_us=%.3f\n", screen_cpu * 1000000.0 / screen_samples))
            f:write(string.format("screen_cpu_max_us=%.3f\n", screen_max_cpu * 1000000.0))
        end
        if sample_count > 0 then
            f:write(string.format("snapshot_cpu_average_us=%.3f\n", snapshot_cpu * 1000000.0 / sample_count))
            f:write(string.format("snapshot_cpu_max_us=%.3f\n", snapshot_max_cpu * 1000000.0))
        end
        f:close()
    end
    manager.machine:exit()
end

emu.register_frame_done(function ()
    if done then return end
    frame_count = frame_count + 1
    if not screen_checked then
        screen_checked = true
        local _, first = next(manager.machine.screens)
        screen = first
        if screen == nil then screen_error = "machine has no screen device" end
    end
    if frame_count <= warmup then return end

    -- This diagnostic alternative may be absent or return non-RGBA pixels.
    -- Never fail the selected production snapshot seam because of that.
    if screen ~= nil and screen_error == nil then
        local t = os.clock()
        local ok_s, sp, x, y = pcall(function ()
            local p, w, h = screen:pixels()
            return p, w, h
        end)
        local elapsed = os.clock() - t
        if not ok_s then
            screen_error = tostring(sp)
        else
            local valid, reason = validate_pixels(sp, x, y)
            if not valid then
                screen_error = reason
            else
                sw, sh, sb = x, y, #sp
                screen_first = screen_first == "none" and quad(sp) or screen_first
                screen_samples = screen_samples + 1
                screen_cpu = screen_cpu + elapsed
                screen_max_cpu = math.max(screen_max_cpu, elapsed)
            end
        end
    end

    local t = os.clock()
    local ok_v, x2, y2, vp = pcall(function ()
        local w, h = manager.machine.video:snapshot_size()
        return w, h, manager.machine.video:snapshot_pixels()
    end)
    local elapsed = os.clock() - t

    if not ok_v then finish("error", "snapshot_pixels failed: " .. tostring(x2)); return end
    local valid, reason = validate_pixels(vp, x2, y2)
    if not valid then finish("error", "snapshot_pixels: " .. reason); return end

    vw, vh, vb = x2, y2, #vp
    snapshot_first = snapshot_first == "none" and quad(vp) or snapshot_first
    sample_count = sample_count + 1
    snapshot_cpu = snapshot_cpu + elapsed
    snapshot_max_cpu = math.max(snapshot_max_cpu, elapsed)
    if sample_count >= samples then finish("ok", nil) end
end, "mame_tauri_frame_seam_qualification")
LUA

# Screen-capture benchmarking deliberately disables video presentation and
# sound. Native user-facing audio must be qualified separately on a desktop.
cmd=("$runtime" "$machine" -video none -sound none -nothrottle -skip_gameinfo -pluginspath "$tmp")
if [[ -n "$rom_dir" ]]; then cmd+=(-rompath "$rom_dir"); fi

set +e
SDL_VIDEODRIVER=dummy MAME_TAURI_FRAME_SEAM_REPORT="$output" MAME_TAURI_FRAME_SEAM_MACHINE="$machine" timeout --kill-after=5s 120s "${cmd[@]}" >"$output.stdout.log" 2>"$output.stderr.log"
status=$?
set -e
[[ $status -eq 0 ]] || { echo "MAME frame-seam qualification failed with status $status" >&2; tail -n 120 "$output.stderr.log" >&2 || true; exit $status; }
grep -q '^status=ok$' "$output" || { cat "$output" >&2; exit 1; }
{
  printf 'sdl_video_driver=dummy\n'
  printf 'runtime_path=%s\n' "$runtime"
  printf 'runtime_sha256=%s\n' "$(sha256sum "$runtime" | awk '{print $1}')"
  printf 'runtime_version=%s\n' "$("$runtime" -noreadconfig -version | head -n 1)"
  printf 'rom_directory=%s\n' "$rom_dir"
  printf 'invocation='
  printf '%q ' "${cmd[@]}"
  printf '\n'
} >>"$output"
printf 'Frame-seam qualification report: %s\n' "$output"
cat "$output"
