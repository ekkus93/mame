#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: test-real-linux-package-metadata-bootstrap.sh <package.deb>" >&2
  exit 64
fi
DEB=$(realpath "$1")
[[ -f "$DEB" ]] || { echo "missing Debian package: $DEB" >&2; exit 1; }
smoke_machine=${MAME_TAURI_REAL_RUNTIME_SMOKE_MACHINE:-gridlee}
package=$(dpkg-deb --field "$DEB" Package)
depends=$(dpkg-deb --field "$DEB" Depends || true)
if grep -Eq '(^|[ ,|])mame([ ,(]|$)' <<<"$depends"; then echo "Debian package must not depend on distro mame: $depends" >&2; exit 1; fi
smoke_tmp=$(mktemp -d)
cleanup() { sudo apt-get remove -y "$package" >/dev/null 2>&1 || true; rm -rf "$smoke_tmp"; }
trap cleanup EXIT
sudo apt-get install -y "$DEB"
installed=$(dpkg -L "$package")
binary=$(grep -m1 '^/usr/bin/[^/]*$' <<<"$installed" || true)
runtime_bin=$(grep -m1 '/mame-runtime/bin/mame$' <<<"$installed" || true)
[[ -n "$binary" && -x "$binary" ]] || { echo "installed frontend executable missing" >&2; exit 1; }
[[ -n "$runtime_bin" && -x "$runtime_bin" ]] || { echo "installed bundled MAME missing" >&2; exit 1; }
runtime_root=${runtime_bin%/bin/mame}
resource_dir=${runtime_root%/mame-runtime}
probe_root="$smoke_tmp/metadata-bootstrap"
mkdir -p "$probe_root/home" "$probe_root/config" "$probe_root/data" "$probe_root/cache" "$probe_root/external"
catalog="$probe_root/catalog.sqlite3"
report="$probe_root/report.json"
override_report="$probe_root/override-reset.json"
external_runtime="$probe_root/external/mame"
cp "$runtime_bin" "$external_runtime"
chmod +x "$external_runtime"
sudo chmod -R a-w "$runtime_root"

env HOME="$probe_root/home" XDG_CONFIG_HOME="$probe_root/config" XDG_DATA_HOME="$probe_root/data" XDG_CACHE_HOME="$probe_root/cache" \
  "$binary" --mame-tauri-verify-runtime-override-reset "$resource_dir" "$external_runtime" >"$override_report"
python3 - "$override_report" "$external_runtime" "$runtime_bin" <<'PY'
import json, os, sys
p, external, bundled = sys.argv[1:]
with open(p, encoding='utf-8') as f: r=json.load(f)
o=r.get('overrideIdentity') or {}; b=r.get('resetIdentity') or {}
errors=[]
if o.get('source')!='external': errors.append(f"override source={o.get('source')!r}")
if os.path.realpath(o.get('path') or '') != os.path.realpath(external): errors.append('override path mismatch')
if b.get('source')!='bundled': errors.append(f"reset source={b.get('source')!r}")
if b.get('trust')!='qualifiedBundled': errors.append(f"reset trust={b.get('trust')!r}")
if os.path.realpath(b.get('path') or '') != os.path.realpath(bundled): errors.append('reset bundled path mismatch')
if errors: raise SystemExit('installed override/reset mismatch: '+', '.join(errors))
PY

env HOME="$probe_root/home" XDG_CONFIG_HOME="$probe_root/config" XDG_DATA_HOME="$probe_root/data" XDG_CACHE_HOME="$probe_root/cache" \
  "$binary" --mame-tauri-verify-bundled-metadata-bootstrap "$resource_dir" "$catalog" "$smoke_machine" >"$report"
python3 - "$report" "$runtime_bin" "$catalog" "$smoke_machine" <<'PY'
import json, os, sys
report_path, expected_runtime, catalog_path, probe_machine = sys.argv[1:]
with open(report_path, encoding='utf-8') as f: report=json.load(f)
refresh=report.get('refresh') or {}; generation=refresh.get('generation') or {}; status=report.get('status') or {}; current=status.get('currentExecutable') or {}; query=report.get('query') or {}; items=query.get('items') or []
errors=[]
if generation.get('sourceKind')!='bundled': errors.append('generation source is not bundled')
if generation.get('trust')!='qualifiedBundled': errors.append('generation trust is not qualifiedBundled')
if int(generation.get('machineCount') or 0)<=0: errors.append('machineCount not positive')
if status.get('freshness')!='fresh': errors.append('metadata not fresh')
if not status.get('activeGeneration'): errors.append('missing activeGeneration')
if current.get('source')!='bundled' or current.get('trust')!='qualifiedBundled': errors.append('current runtime not qualified bundled')
if os.path.realpath(current.get('path') or '') != os.path.realpath(expected_runtime): errors.append('runtime path mismatch')
if int(query.get('total') or 0)<=0 or not items: errors.append('catalog query empty')
if not any(i.get('shortName')==probe_machine for i in items): errors.append('probe machine absent')
if not os.path.exists(catalog_path): errors.append('catalog missing')
if errors: raise SystemExit('metadata bootstrap report mismatch: '+', '.join(errors))
PY
printf 'Installed external override/reset and bundled metadata bootstrap qualification passed for %s\n' "$smoke_machine"
