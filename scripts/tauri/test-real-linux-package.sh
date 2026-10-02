#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: test-real-linux-package.sh <package.deb> [package.AppImage]

Release-grade Linux package qualification for the bundled MAME runtime.
Unlike test-linux-packages.sh, this script rejects synthetic MT-1305 fixtures and
requires a release-qualified real runtime provenance record.
EOF
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage >&2
  exit 64
fi

DEB=$(realpath "$1")
APPIMAGE=${2:-}
if [[ -n "$APPIMAGE" ]]; then
  APPIMAGE=$(realpath "$APPIMAGE")
fi

[[ -f "$DEB" ]] || { echo "missing Debian package: $DEB" >&2; exit 1; }
if [[ -n "$APPIMAGE" ]]; then
  [[ -f "$APPIMAGE" ]] || { echo "missing AppImage: $APPIMAGE" >&2; exit 1; }
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
validator="$repo_root/scripts/tauri/validate-real-mame-runtime.sh"
smoke_machine=${MAME_TAURI_REAL_RUNTIME_SMOKE_MACHINE:-pacman}
[[ -x "$validator" ]] || { echo "missing real runtime validator: $validator" >&2; exit 1; }

package=$(dpkg-deb --field "$DEB" Package)
depends=$(dpkg-deb --field "$DEB" Depends || true)

if grep -Eq '(^|[ ,|])mame([ ,(]|$)' <<<"$depends"; then
  echo "Debian package must not depend on the distro mame package: $depends" >&2
  exit 1
fi

payload=$(mktemp)
smoke_tmp=$(mktemp -d)
extract_tmp=$(mktemp -d)
trap 'rm -f "$payload"; rm -rf "$smoke_tmp" "$extract_tmp"' EXIT

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

require_file_under() {
  local root=$1
  local relative=$2
  local description=$3
  [[ -f "$root/$relative" ]] || {
    echo "missing $description: $root/$relative" >&2
    find "$root" -maxdepth 6 -type f -print >&2 || true
    exit 1
  }
}

require_any_file_under() {
  local root=$1
  local find_args=$2
  local description=$3
  local found
  # shellcheck disable=SC2086
  found=$(find "$root" $find_args -print -quit)
  [[ -n "$found" ]] || {
    echo "missing $description under $root" >&2
    find "$root" -maxdepth 6 -type f -print >&2 || true
    exit 1
  }
}

assert_real_provenance() {
  local provenance=$1
  grep -Fq 'mode=real' "$provenance" || {
    echo "runtime provenance is not real-mode:" >&2
    cat "$provenance" >&2
    exit 1
  }
  grep -Fq 'release_qualified=true' "$provenance" || {
    echo "runtime provenance is not release-qualified:" >&2
    cat "$provenance" >&2
    exit 1
  }
  if grep -Eiq 'synthetic|fixture payload|MT-1305' "$provenance"; then
    echo "runtime provenance contains synthetic fixture markers:" >&2
    cat "$provenance" >&2
    exit 1
  fi
}

assert_real_executable_type() {
  local executable=$1
  local description
  description=$(file -b "$executable")
  if grep -Eiq 'shell script|text executable|ASCII text' <<<"$description"; then
    echo "bundled runtime is not a real binary executable: $description" >&2
    exit 1
  fi
  printf 'Real runtime executable type: %s\n' "$description"
}

assert_no_missing_shared_libraries() {
  local executable=$1
  local ldd_log=$2
  if ldd "$executable" >"$ldd_log" 2>&1; then
    if grep -Fq 'not found' "$ldd_log"; then
      echo "bundled MAME has unresolved shared-library dependencies:" >&2
      cat "$ldd_log" >&2
      exit 1
    fi
  elif grep -Fq 'not a dynamic executable' "$ldd_log"; then
    printf 'Bundled MAME is not dynamically linked according to ldd\n'
  else
    echo "ldd failed for bundled MAME:" >&2
    cat "$ldd_log" >&2
    exit 1
  fi
}

assert_deb_declares_runtime_library_dependencies() {
  local executable=$1
  local deb_depends=$2
  local ldd_log=$3
  local declared
  declared=$(python3 - "$deb_depends" <<'PY'
import re
import sys

for clause in sys.argv[1].split(','):
    for alternative in clause.split('|'):
        token = alternative.strip().split(maxsplit=1)[0] if alternative.strip() else ''
        if not token:
            continue
        print(re.split(r'[:(]', token, maxsplit=1)[0])
PY
)

  declare -A seen=()
  local missing=()
  local library canonical owner package
  while IFS= read -r library; do
    [[ -n "$library" ]] || continue
    canonical=$(readlink -f "$library")
    owner=$(dpkg-query -S "$canonical" 2>/dev/null | head -n 1 | cut -d: -f1 || true)
    if [[ -z "$owner" ]]; then
      owner=$(dpkg-query -S "$library" 2>/dev/null | head -n 1 | cut -d: -f1 || true)
    fi
    [[ -n "$owner" ]] || {
      echo "no Debian package owns runtime library: $library" >&2
      exit 1
    }
    package=${owner%%:*}
    [[ -n "${seen[$package]:-}" ]] && continue
    seen[$package]=1
    if ! grep -Fxq "$package" <<<"$declared"; then
      missing+=("$package")
    fi
  done < <(
    ldd "$executable" >"$ldd_log" 2>&1
    awk '
      /=> \/[^ ]+/ { print $3 }
      /^[[:space:]]*\/[^ ]+/ { print $1 }
    ' "$ldd_log" | sort -u
  )

  if [[ ${#missing[@]} -ne 0 ]]; then
    echo "Debian package does not declare bundled MAME shared-library packages: ${missing[*]}" >&2
    echo "Depends: ${deb_depends:-<none>}" >&2
    echo "--- bundled MAME ldd ---" >&2
    cat "$ldd_log" >&2
    exit 1
  fi
  printf 'Debian package declares bundled MAME shared-library dependencies\n'
}

assert_runtime_tree() {
  local runtime_root=$1
  local runtime_bin="$runtime_root/bin/mame"
  [[ -x "$runtime_bin" ]] || { echo "bundled MAME executable is missing or non-executable: $runtime_bin" >&2; exit 1; }
  require_any_file_under "$runtime_root/hash" '-maxdepth 1 -type f -name *.xml' 'MAME hash XML resources'
  require_any_file_under "$runtime_root/bgfx" '-type f' 'MAME BGFX runtime resources'
  require_file_under "$runtime_root" 'licenses/COPYING' 'MAME COPYING file'
  require_any_file_under "$runtime_root/licenses/legal" '-type f' 'MAME legal/license resources'
  require_file_under "$runtime_root" 'runtime-provenance.txt' 'runtime provenance'
  assert_real_provenance "$runtime_root/runtime-provenance.txt"
  assert_real_executable_type "$runtime_bin"
  "$validator" "$runtime_bin"
  if [[ "${MAME_TAURI_REQUIRE_ARCADE_DRIVERS:-}" == "1" ]]; then
    "$repo_root/scripts/tauri/validate-release-driver-coverage.sh" "$runtime_bin"
  fi
  assert_no_missing_shared_libraries "$runtime_bin" "$smoke_tmp/ldd.log"
}

refresh_installed_paths() {
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
}

assert_backend_resolves_installed_bundled_runtime() {
  local runtime_root=$1
  local runtime_bin="$runtime_root/bin/mame"
  local resource_dir=${runtime_root%/mame-runtime}
  local probe_root="$smoke_tmp/backend-bundled-runtime"
  local probe_json="$probe_root/runtime-identity.json"
  mkdir -p "$probe_root/home" "$probe_root/config" "$probe_root/data" "$probe_root/cache"

  env \
    HOME="$probe_root/home" \
    XDG_CONFIG_HOME="$probe_root/config" \
    XDG_DATA_HOME="$probe_root/data" \
    XDG_CACHE_HOME="$probe_root/cache" \
    "$binary" --mame-tauri-verify-bundled-runtime "$resource_dir" >"$probe_json"

  python3 - "$probe_json" "$runtime_bin" <<'PY'
import json
import os
import sys

probe_path, expected_runtime = sys.argv[1], os.path.realpath(sys.argv[2])
with open(probe_path, encoding='utf-8') as handle:
    identity = json.load(handle)

errors = []
if identity.get('source') != 'bundled':
    errors.append(f"source={identity.get('source')!r}")
if identity.get('trust') != 'qualifiedBundled':
    errors.append(f"trust={identity.get('trust')!r}")
actual_path = os.path.realpath(identity.get('path', ''))
if actual_path != expected_runtime:
    errors.append(f"path={actual_path!r} expected={expected_runtime!r}")
if not identity.get('version'):
    errors.append('missing version')
if not identity.get('rawVersionLine'):
    errors.append('missing rawVersionLine')

if errors:
    raise SystemExit('installed backend bundled-runtime identity mismatch: ' + ', '.join(errors))
PY

  printf 'Installed backend resolves default runtime as bundled: %s\n' "$runtime_bin"
}

frontend_rel=$(require_payload_match '^(\./)?usr/bin/[^/]+$' 'frontend executable')
runtime_rel=$(require_payload_match '/mame-runtime/bin/mame$' 'bundled MAME executable')
desktop_rel=$(require_payload_match '^(\./)?usr/share/applications/.*\.desktop$' 'desktop entry')
runtime_root_rel=${runtime_rel%/bin/mame}

for pattern in \
  "$runtime_root_rel/hash/.*\.xml" \
  "$runtime_root_rel/bgfx/.+" \
  "$runtime_root_rel/licenses/COPYING" \
  "$runtime_root_rel/licenses/legal/.+" \
  "$runtime_root_rel/runtime-provenance.txt"; do
  if ! grep -Eq "^${pattern}$" "$payload"; then
    echo "Debian package is missing required bundled runtime payload matching: $pattern" >&2
    dump_payload
    exit 1
  fi
done

printf 'Real-runtime Debian payload root: %s\n' "${runtime_root_rel#./}"
printf 'Real-runtime Debian package dependencies: %s\n' "${depends:-<none>}"

sudo apt-get install -y "$DEB"

refresh_installed_paths

assert_runtime_tree "$runtime_root"
assert_backend_resolves_installed_bundled_runtime "$runtime_root"
assert_deb_declares_runtime_library_dependencies "$runtime_bin" "$depends" "$smoke_tmp/declared-runtime-deps-ldd.log"

readonly_probe_root="$smoke_tmp/readonly-runtime"
mkdir -p "$readonly_probe_root/cwd" "$readonly_probe_root/home" "$readonly_probe_root/config" "$readonly_probe_root/data" "$readonly_probe_root/cache"
sudo chmod -R a-w "$runtime_root"
(
  cd "$readonly_probe_root/cwd"
  env \
    HOME="$readonly_probe_root/home" \
    XDG_CONFIG_HOME="$readonly_probe_root/config" \
    XDG_DATA_HOME="$readonly_probe_root/data" \
    XDG_CACHE_HOME="$readonly_probe_root/cache" \
    "$runtime_bin" -noreadconfig -version >"$smoke_tmp/readonly-version.log" 2>&1
)
grep -Eq '^[0-9]+\.[0-9]+' "$smoke_tmp/readonly-version.log" || {
  echo "read-only runtime -version smoke did not report real MAME identity:" >&2
  cat "$smoke_tmp/readonly-version.log" >&2
  exit 1
}
(
  cd "$readonly_probe_root/cwd"
  env \
    HOME="$readonly_probe_root/home" \
    XDG_CONFIG_HOME="$readonly_probe_root/config" \
    XDG_DATA_HOME="$readonly_probe_root/data" \
    XDG_CACHE_HOME="$readonly_probe_root/cache" \
    "$runtime_bin" -noreadconfig -listxml "$smoke_machine" >"$smoke_tmp/readonly-listxml.xml" 2>"$smoke_tmp/readonly-listxml.err"
)
grep -Eq "<machine[^>]+name=\"$smoke_machine\"" "$smoke_tmp/readonly-listxml.xml" || {
  echo "read-only runtime listxml smoke did not return $smoke_machine metadata" >&2
  cat "$smoke_tmp/readonly-listxml.err" >&2 || true
  exit 1
}

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

    for _ in $(seq 1 100); do
      window_id=$(xdotool search --pid "$pid" 2>/dev/null | head -n 1 || true)
      if [[ -z "$window_id" ]]; then
        window_id=$(xdotool search --name "^MAME Tauri Frontend$" 2>/dev/null | head -n 1 || true)
      fi
      if [[ -z "$window_id" ]]; then
        window_id=$(xdotool search --name "^mame-tauri$" 2>/dev/null | head -n 1 || true)
      fi
      if [[ -n "$window_id" ]]; then
        window_pid=$(xdotool getwindowpid "$window_id" 2>/dev/null || true)
        window_name=$(xdotool getwindowname "$window_id" 2>/dev/null || true)
        printf "Real-runtime X11 window created: id=%s pid=%s name=%s\n" \
          "$window_id" "${window_pid:-unknown}" "${window_name:-<unnamed>}"
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
        exit 0
      fi
      if ! kill -0 "$pid" 2>/dev/null; then
        echo "application exited before creating an X11 window" >&2
        cat "$log_path" >&2
        exit 1
      fi
      sleep 0.25
    done

    echo "application did not create the expected X11 window before timeout" >&2
    echo "--- application log ---" >&2
    cat "$log_path" >&2
    exit 1
  ' bash "$log_path" "$@"
}

assert_package_reinstall_preserves_user_state() {
  local probe_root="$smoke_tmp/package-reinstall"
  local sentinel
  mkdir -p "$probe_root/home" "$probe_root/config" "$probe_root/data" "$probe_root/cache"

  for sentinel in \
    "$probe_root/home/user-content.sentinel" \
    "$probe_root/config/user-settings.sentinel" \
    "$probe_root/data/user-library.sentinel" \
    "$probe_root/cache/user-cache.sentinel"; do
    printf 'preserve across package reinstall: %s\n' "$(basename "$sentinel")" >"$sentinel"
  done

  x11_smoke "$smoke_tmp/deb-reinstall-before.log" env \
    HOME="$probe_root/home" \
    XDG_CONFIG_HOME="$probe_root/config" \
    XDG_DATA_HOME="$probe_root/data" \
    XDG_CACHE_HOME="$probe_root/cache" \
    "$binary"

  sudo apt-get install --reinstall -y "$DEB"

  refresh_installed_paths
  assert_runtime_tree "$runtime_root"
  assert_backend_resolves_installed_bundled_runtime "$runtime_root"
  assert_deb_declares_runtime_library_dependencies "$runtime_bin" "$depends" "$smoke_tmp/reinstall-declared-runtime-deps-ldd.log"

  "$runtime_bin" -noreadconfig -version >"$smoke_tmp/reinstall-version.log" 2>&1
  grep -Eq '^[0-9]+\.[0-9]+' "$smoke_tmp/reinstall-version.log" || {
    echo "reinstalled package runtime -version smoke did not report real MAME identity:" >&2
    cat "$smoke_tmp/reinstall-version.log" >&2
    exit 1
  }

  for sentinel in \
    "$probe_root/home/user-content.sentinel" \
    "$probe_root/config/user-settings.sentinel" \
    "$probe_root/data/user-library.sentinel" \
    "$probe_root/cache/user-cache.sentinel"; do
    [[ -s "$sentinel" ]] || {
      echo "user-state sentinel missing after Debian package reinstall: $sentinel" >&2
      find "$probe_root" -maxdepth 3 -type f -print >&2 || true
      exit 1
    }
  done

  x11_smoke "$smoke_tmp/deb-reinstall-after.log" env \
    HOME="$probe_root/home" \
    XDG_CONFIG_HOME="$probe_root/config" \
    XDG_DATA_HOME="$probe_root/data" \
    XDG_CACHE_HOME="$probe_root/cache" \
    "$binary"

  printf 'Debian package reinstall preserved user state and restored package-owned bundled runtime\n'
}

x11_smoke "$smoke_tmp/deb-launch.log" "$binary"
assert_package_reinstall_preserves_user_state

sudo apt-get remove -y "$package"
[[ ! -e "$binary" ]] || { echo "frontend executable remains after Debian uninstall: $binary" >&2; exit 1; }
[[ ! -e "$runtime_bin" ]] || { echo "bundled runtime remains after Debian uninstall: $runtime_bin" >&2; exit 1; }

if [[ -n "$APPIMAGE" ]]; then
  chmod +x "$APPIMAGE"
  x11_smoke "$smoke_tmp/appimage-launch.log" env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE"

  app_extract="$extract_tmp/appimage"
  mkdir -p "$app_extract"
  (
    cd "$app_extract"
    "$APPIMAGE" --appimage-extract >/dev/null
  )
  app_root="$app_extract/squashfs-root"
  app_runtime_bin=$(find "$app_root" -path '*/mame-runtime/bin/mame' -type f -print -quit)
  [[ -n "$app_runtime_bin" ]] || { echo "AppImage bundled MAME executable is missing" >&2; find "$app_root" -maxdepth 6 -type f -print >&2; exit 1; }
  assert_runtime_tree "${app_runtime_bin%/bin/mame}"
fi

printf 'Real-runtime Debian install/uninstall, bundled MAME execution, backend bundled-source resolution, read-only runtime, declared shared-library dependencies, package reinstall/user-state preservation, desktop launch, and package dependency qualification passed\n'
