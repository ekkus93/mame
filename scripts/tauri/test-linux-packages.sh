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

listing=$(mktemp)
smoke_tmp=$(mktemp -d)
trap 'rm -f "$listing"; rm -rf "$smoke_tmp"' EXIT

dpkg-deb --contents "$DEB" >"$listing"
grep -Eq '/usr/bin/[^/]+$' "$listing"
grep -Fq '/usr/lib/mame-tauri-frontend/mame-runtime/bin/mame' "$listing"
grep -Fq '/usr/lib/mame-tauri-frontend/mame-runtime/hash/fixture.xml' "$listing"
grep -Fq '/usr/lib/mame-tauri-frontend/mame-runtime/bgfx/chains/fixture.json' "$listing"
grep -Fq '/usr/lib/mame-tauri-frontend/mame-runtime/licenses/COPYING' "$listing"
grep -Fq '/usr/lib/mame-tauri-frontend/mame-runtime/licenses/legal/GPL-2.0' "$listing"
grep -Eq '/usr/share/applications/.*\.desktop$' "$listing"

sudo apt-get install -y "$DEB"

binary=$(dpkg -L "$package" | grep '^/usr/bin/' | head -n 1)
runtime_bin=$(dpkg -L "$package" | grep '/mame-runtime/bin/mame$' | head -n 1)
desktop_file=$(dpkg -L "$package" | grep '^/usr/share/applications/.*\.desktop$' | head -n 1)

[[ -x "$binary" ]] || { echo "installed frontend executable is missing: $binary" >&2; exit 1; }
[[ -x "$runtime_bin" ]] || { echo "installed MAME executable is missing or non-executable: $runtime_bin" >&2; exit 1; }
[[ -f "$desktop_file" ]] || { echo "installed desktop entry is missing: $desktop_file" >&2; exit 1; }

grep -Fq 'Name=MAME Tauri Frontend' "$desktop_file"
grep -Eq '^Exec=.*mame-tauri' "$desktop_file"

runtime_root=${runtime_bin%/bin/mame}
[[ -f "$runtime_root/hash/fixture.xml" ]]
[[ -f "$runtime_root/bgfx/chains/fixture.json" ]]
[[ -f "$runtime_root/licenses/COPYING" ]]
[[ -f "$runtime_root/licenses/legal/GPL-2.0" ]]

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
        cat "$log_path" >&2
        exit 1
      fi
      sleep 0.25
    done
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
[[ -n "$app_runtime_bin" && -x "$app_runtime_bin" ]] || { echo "AppImage bundled MAME executable is missing or non-executable" >&2; exit 1; }
app_runtime_root=${app_runtime_bin%/bin/mame}
[[ -f "$app_runtime_root/hash/fixture.xml" ]]
[[ -f "$app_runtime_root/bgfx/chains/fixture.json" ]]
[[ -f "$app_runtime_root/licenses/COPYING" ]]
[[ -f "$app_runtime_root/licenses/legal/GPL-2.0" ]]
app_desktop=$(find "$app_root" -name '*.desktop' -type f -print -quit)
[[ -n "$app_desktop" ]] || { echo "AppImage desktop entry is missing" >&2; exit 1; }
grep -Fq 'Name=MAME Tauri Frontend' "$app_desktop"

printf 'MT-1305 Debian install/uninstall, X11 launch, desktop integration, and AppImage extraction/launch smoke passed\n'
