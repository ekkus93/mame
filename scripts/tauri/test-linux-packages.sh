#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <package.deb> <package.AppImage>" >&2
  exit 64
fi

DEB=$(realpath "$1")
APPIMAGE=$(realpath "$2")

[[ -f "$DEB" ]] || { echo "missing Debian package: $DEB" >&2; exit 1; }
[[ -f "$APPIMAGE" ]] || { echo "missing AppImage: $APPIMAGE" >&2; exit 1; }

package=$(dpkg-deb --field "$DEB" Package)
depends=$(dpkg-deb --field "$DEB" Depends)

if ! grep -Eq 'libwebkit2gtk-4\.1' <<<"$depends"; then
  echo "Debian package does not declare the WebKitGTK 4.1 runtime dependency: $depends" >&2
  exit 1
fi
if ! grep -Eq 'libgtk-3' <<<"$depends"; then
  echo "Debian package does not declare the GTK 3 runtime dependency: $depends" >&2
  exit 1
fi

payload=$(mktemp)
smoke_tmp=$(mktemp -d)
trap 'rm -f "$payload"; rm -rf "$smoke_tmp"' EXIT

dpkg-deb --fsys-tarfile "$DEB" | tar -tf - >"$payload"

dump_payload() {
  echo "--- Debian package payload ($DEB) ---" >&2
  cat "$payload" >&2
  echo "--- end Debian package payload ---" >&2
}

require_payload_match() {
  local pattern=$1
  local description=$2
  local match
  match=$(grep -Em1 "$pattern" "$payload" || true)
  if [[ -z "$match" ]]; then
    echo "Debian package is missing $description (pattern: $pattern)" >&2
    dump_payload
    exit 1
  fi
  printf '%s\n' "$match"
}

frontend_rel=$(require_payload_match '^\./usr/bin/[^/]+$' 'frontend executable')
runtime_rel=$(require_payload_match '/mame-runtime/bin/mame$' 'bundled MAME executable')
desktop_rel=$(require_payload_match '^\./usr/share/applications/.*\.desktop$' 'desktop entry')
runtime_root_rel=${runtime_rel%/bin/mame}

for required in \
  "$runtime_root_rel/hash/fixture.xml" \
  "$runtime_root_rel/bgfx/chains/fixture.json" \
  "$runtime_root_rel/licenses/COPYING" \
  "$runtime_root_rel/licenses/legal/GPL-2.0"; do
  if ! grep -Fxq "$required" "$payload"; then
    echo "Debian package is missing packaged runtime resource: $required" >&2
    dump_payload
    exit 1
  fi
done

printf 'MT-1305 Debian payload root: %s\n' "${runtime_root_rel#./}"
printf 'MT-1305 Debian package dependencies: %s\n' "$depends"

sudo apt-get install -y "$DEB"

installed=$(dpkg -L "$package")
binary=$(grep -m1 '^/usr/bin/[^/]*$' <<<"$installed" || true)
runtime_bin=$(grep -m1 '/mame-runtime/bin/mame$' <<<"$installed" || true)
desktop_file=$(grep -m1 '^/usr/share/applications/.*\.desktop$' <<<"$installed" || true)

[[ -n "$binary" && -x "$binary" ]] || { echo "installed frontend executable is missing or non-executable: ${binary:-<not found>}" >&2; printf '%s\n' "$installed" >&2; exit 1; }
[[ -n "$runtime_bin" && -x "$runtime_bin" ]] || { echo "installed MAME executable is missing or non-executable: ${runtime_bin:-<not found>}" >&2; printf '%s\n' "$installed" >&2; exit 1; }
[[ -n "$desktop_file" && -f "$desktop_file" ]] || { echo "installed desktop entry is missing: ${desktop_file:-<not found>}" >&2; printf '%s\n' "$installed" >&2; exit 1; }

grep -Fq 'Name=MAME Tauri Frontend' "$desktop_file" || { echo "desktop entry has unexpected Name:" >&2; cat "$desktop_file" >&2; exit 1; }
frontend_name=$(basename "$binary")
grep -Eq "^Exec=.*${frontend_name}" "$desktop_file" || { echo "desktop entry does not launch $frontend_name:" >&2; cat "$desktop_file" >&2; exit 1; }

runtime_root=${runtime_bin%/bin/mame}
for required in \
  "$runtime_root/hash/fixture.xml" \
  "$runtime_root/bgfx/chains/fixture.json" \
  "$runtime_root/licenses/COPYING" \
  "$runtime_root/licenses/legal/GPL-2.0"; do
  [[ -f "$required" ]] || { echo "installed runtime resource is missing: $required" >&2; exit 1; }
done

x11_smoke() {
  local log_path=$1
  shift
  xvfb-run -a bash -c '
    set -euo pipefail
    log_path=$1
    shift
    "$@" >"$log_path" 2>&1 &
    pid=$!
    trap "kill $pid 2>/dev/null || true" EXIT
    for _ in $(seq 1 60); do
      if xdotool search --onlyvisible --name "MAME Tauri Frontend" >/dev/null 2>&1; then
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
        exit 0
      fi
      if ! kill -0 "$pid" 2>/dev/null; then
        echo "application exited before exposing the expected window" >&2
        cat "$log_path" >&2
        exit 1
      fi
      sleep 0.25
    done
    echo "application did not expose the expected window before timeout" >&2
    cat "$log_path" >&2
    exit 1
  ' bash "$log_path" "$@"
}

x11_smoke "$smoke_tmp/deb-launch.log" "$binary"

sudo apt-get remove -y "$package"
[[ ! -e "$binary" ]] || { echo "frontend executable remains after Debian uninstall: $binary" >&2; exit 1; }
[[ ! -e "$runtime_bin" ]] || { echo "bundled runtime remains after Debian uninstall: $runtime_bin" >&2; exit 1; }

chmod +x "$APPIMAGE"
x11_smoke "$smoke_tmp/appimage-launch.log" env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE"

extract_dir="$smoke_tmp/appimage"
mkdir -p "$extract_dir"
(
  cd "$extract_dir"
  "$APPIMAGE" --appimage-extract >/dev/null
)

app_root="$extract_dir/squashfs-root"
[[ -x "$app_root/AppRun" ]] || { echo "AppImage did not contain an executable AppRun" >&2; exit 1; }
app_runtime_bin=$(find "$app_root" -path '*/mame-runtime/bin/mame' -type f -print -quit)
[[ -n "$app_runtime_bin" && -x "$app_runtime_bin" ]] || { echo "AppImage bundled MAME executable is missing or non-executable" >&2; find "$app_root" -maxdepth 5 -type f -print >&2; exit 1; }
app_runtime_root=${app_runtime_bin%/bin/mame}
for required in \
  "$app_runtime_root/hash/fixture.xml" \
  "$app_runtime_root/bgfx/chains/fixture.json" \
  "$app_runtime_root/licenses/COPYING" \
  "$app_runtime_root/licenses/legal/GPL-2.0"; do
  [[ -f "$required" ]] || { echo "AppImage runtime resource is missing: $required" >&2; exit 1; }
done
app_desktop=$(find "$app_root" -name '*.desktop' -type f -print -quit)
[[ -n "$app_desktop" ]] || { echo "AppImage desktop entry is missing" >&2; exit 1; }
grep -Fq 'Name=MAME Tauri Frontend' "$app_desktop" || { echo "AppImage desktop entry has unexpected Name:" >&2; cat "$app_desktop" >&2; exit 1; }

printf 'MT-1305 Debian install/uninstall, X11 launch, desktop integration, and AppImage extraction/launch smoke passed\n'
