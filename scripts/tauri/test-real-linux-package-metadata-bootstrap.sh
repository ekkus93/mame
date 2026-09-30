#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: test-real-linux-package-metadata-bootstrap.sh <package.deb>

Release-grade metadata-bootstrap qualification for the installed bundled-MAME
Debian package. This test reinstalls the package, makes the installed bundled
runtime tree read-only, asks the installed backend to resolve the default
package-owned runtime, imports real MAME -listxml metadata into a user-writable
catalog, verifies the resulting machine catalog is queryable, then verifies an
explicit external override and reset back to the installed bundled runtime.
EOF
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 64
fi

DEB=$(realpath "$1")
[[ -f "$DEB" ]] || { echo "missing Debian package: $DEB" >&2; exit 1; }

smoke_machine=${MAME_TAURI_REAL_RUNTIME_SMOKE_MACHINE:-gridlee}
package=$(dpkg-deb --field "$DEB" Package)
depends=$(dpkg-deb --field "$DEB" Depends || true)

if grep -Eq '(^|[ ,|])mame([ ,(]|$)' <<<"$depends"; then
  echo "Debian package must not depend on the distro mame package: $depends" >&2
  exit 1
fi

smoke_tmp=$(mktemp -d)
cleanup() {
  sudo apt-get remove -y "$package" >/dev/null 2>&1 || true
  rm -rf "$smoke_tmp"
}
trap cleanup EXIT

sudo apt-get install -y "$DEB"

installed=$(dpkg -L "$package")
binary=$(grep -m1 '^/usr/bin/[^/]*$' <<<"$installed" || true)
runtime_bin=$(grep -m1 '/mame-runtime/bin/mame$' <<<"$installed" || true)
[[ -n "$binary" && -x "$binary" ]] || { echo "installed frontend executable is missing or non-executable: ${binary:-<not found>}" >&2; printf '%s\n' "$installed" >&2; exit 1; }
[[ -n "$runtime_bin" && -x "$runtime_bin" ]] || { echo "installed bundled MAME executable is missing or non-executable: ${runtime_bin:-<not found>}" >&2; printf '%s\n' "$installed" >&2; exit 1; }

runtime_root=${runtime_bin%/bin/mame}
resource_dir=${runtime_root%/mame-runtime}
[[ -d "$resource_dir" ]] || { echo "installed Tauri resource directory is missing: $resource_dir" >&2; exit 1; }

probe_root="$smoke_tmp/metadata-bootstrap"
mkdir -p "$probe_root/home" "$probe_root/config" "$probe_root/data" "$probe_root/cache"
catalog="$probe_root/catalog.sqlite3"
report="$probe_root/report.json"

sudo chmod -R a-w "$runtime_root"

env \
  HOME="$probe_root/home" \
  XDG_CONFIG_HOME="$probe_root/config" \
  XDG_DATA_HOME="$probe_root/data" \
  XDG_CACHE_HOME="$probe_root/cache" \
  "$binary" --mame-tauri-verify-bundled-metadata-bootstrap \
    "$resource_dir" \
    "$catalog" \
    "$smoke_machine" >"$report"

python3 - "$report" "$runtime_bin" "$catalog" "$smoke_machine" <<'PY'
import json
import os
import sys

report_path, expected_runtime, catalog_path, probe_machine = sys.argv[1:]
expected_runtime = os.path.realpath(expected_runtime)
with open(report_path, encoding='utf-8') as handle:
    report = json.load(handle)

errors = []
refresh = report.get('refresh') or {}
generation = refresh.get('generation') or {}
status = report.get('status') or {}
current = status.get('currentExecutable') or {}
query = report.get('query') or {}
items = query.get('items') or []

if generation.get('sourceKind') != 'bundled':
    errors.append(f"generation.sourceKind={generation.get('sourceKind')!r}")
if generation.get('trust') != 'qualifiedBundled':
    errors.append(f"generation.trust={generation.get('trust')!r}")
if int(generation.get('machineCount') or 0) <= 0:
    errors.append('generation.machineCount must be positive')
if status.get('freshness') != 'fresh':
    errors.append(f"status.freshness={status.get('freshness')!r}")
if not status.get('activeGeneration'):
    errors.append('missing activeGeneration')
if current.get('source') != 'bundled':
    errors.append(f"currentExecutable.source={current.get('source')!r}")
if current.get('trust') != 'qualifiedBundled':
    errors.append(f"currentExecutable.trust={current.get('trust')!r}")
actual_runtime = os.path.realpath(current.get('path') or '')
if actual_runtime != expected_runtime:
    errors.append(f"currentExecutable.path={actual_runtime!r} expected={expected_runtime!r}")
if int(query.get('total') or 0) <= 0:
    errors.append('query.total must be positive')
if not items:
    errors.append('query.items must be non-empty')
if not any(item.get('shortName') == probe_machine for item in items):
    errors.append(f"query.items does not contain probe machine {probe_machine!r}")
if not os.path.exists(catalog_path):
    errors.append(f"catalog was not created: {catalog_path}")

if errors:
    raise SystemExit('metadata bootstrap report mismatch: ' + ', '.join(errors))
PY

external_runtime="$probe_root/external-mame"
cp "$runtime_bin" "$external_runtime"
chmod u+w "$external_runtime"

cargo run --quiet \
  --manifest-path tauri/src-tauri/Cargo.toml \
  --example verify_runtime_override_reset \
  -- "$resource_dir" "$external_runtime" >"$probe_root/override-reset.json"

python3 - "$probe_root/override-reset.json" "$runtime_bin" "$external_runtime" <<'PY'
import json
import os
import sys

report_path, bundled_runtime, external_runtime = sys.argv[1:]
with open(report_path, encoding='utf-8') as handle:
    report = json.load(handle)

override = report.get('overrideIdentity') or {}
reset = report.get('resetIdentity') or {}
errors = []

if override.get('source') != 'external':
    errors.append(f"override.source={override.get('source')!r}")
if override.get('trust') != 'userConfigured':
    errors.append(f"override.trust={override.get('trust')!r}")
if os.path.realpath(override.get('path') or '') != os.path.realpath(external_runtime):
    errors.append(f"override.path={override.get('path')!r}")
if reset.get('source') != 'bundled':
    errors.append(f"reset.source={reset.get('source')!r}")
if reset.get('trust') != 'qualifiedBundled':
    errors.append(f"reset.trust={reset.get('trust')!r}")
if os.path.realpath(reset.get('path') or '') != os.path.realpath(bundled_runtime):
    errors.append(f"reset.path={reset.get('path')!r}")

if errors:
    raise SystemExit('override/reset report mismatch: ' + ', '.join(errors))
PY

printf 'Installed bundled-MAME metadata bootstrap and override/reset qualification passed for %s\n' "$smoke_machine"
