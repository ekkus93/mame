#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo 'Usage: finalize-appimage-runtime-provenance.sh <package.AppImage> <staged-runtime-root>' >&2
  exit 64
fi
appimage=$(realpath "$1")
staged=$(realpath "$2")
repo_root=$(cd "$(dirname "$0")/../.." && pwd)
verifier="$repo_root/scripts/tauri/verify-runtime-provenance.sh"
[[ -f "$appimage" && -x "$appimage" ]] || { echo "AppImage missing or not executable: $appimage" >&2; exit 1; }
"$verifier" "$staged"
for tool in mksquashfs objcopy readelf; do
  command -v "$tool" >/dev/null || { echo "Required AppImage provenance tool missing: $tool" >&2; exit 1; }
done

scratch=$(mktemp -d "$(dirname "$appimage")/.appimage-provenance.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/extract" "$scratch/check"
(cd "$scratch/extract" && "$appimage" --appimage-extract >/dev/null)
mapfile -d '' -t candidates < <(find "$scratch/extract/squashfs-root" -type f -path '*/mame-runtime/bin/mame' -print0)
[[ ${#candidates[@]} -eq 1 ]] || { echo 'AppImage must contain exactly one MAME runtime' >&2; exit 1; }
runtime=${candidates[0]%/bin/mame}
staged_hash=$(sha256sum "$staged/bin/mame" | awk '{print $1}')
app_hash=$(sha256sum "$runtime/bin/mame" | awk '{print $1}')
if [[ "$app_hash" == "$staged_hash" ]]; then
  "$verifier" "$runtime"
  echo 'AppImage executable already matches staged provenance.'
  exit 0
fi

# linuxdeploy may rewrite ELF RPATH metadata. Verify unchanged GNU build ID
# and .text machine code before recording the final packaged executable digest.
for binary in "$staged/bin/mame" "$runtime/bin/mame"; do
  [[ "$(file -b "$binary")" == *ELF* ]] || { echo "Not an ELF executable: $binary" >&2; exit 1; }
done
staged_id=$(readelf -n "$staged/bin/mame" 2>/dev/null | sed -n 's/^[[:space:]]*Build ID: //p' | head -n 1)
app_id=$(readelf -n "$runtime/bin/mame" 2>/dev/null | sed -n 's/^[[:space:]]*Build ID: //p' | head -n 1)
[[ -n "$staged_id" && "$staged_id" == "$app_id" ]] || {
  echo 'AppImage MAME build ID differs from staged executable; refusing provenance rewrite' >&2
  exit 1
}
objcopy --dump-section .text="$scratch/staged.text" "$staged/bin/mame"
objcopy --dump-section .text="$scratch/app.text" "$runtime/bin/mame"
cmp -s "$scratch/staged.text" "$scratch/app.text" || {
  echo 'AppImage MAME machine code differs from staged executable; refusing provenance rewrite' >&2
  exit 1
}
python3 - "$runtime/runtime-provenance.txt" "$staged_hash" "$app_hash" <<'PY'
from pathlib import Path
import sys
manifest = Path(sys.argv[1])
original, relocated = sys.argv[2:]
lines = manifest.read_text().splitlines()
matches = [i for i, line in enumerate(lines) if line.startswith("mame_sha256=")]
if len(matches) != 1 or lines[matches[0]] != f"mame_sha256={original}":
    raise SystemExit("AppImage provenance does not match staged runtime; refusing rewrite")
lines[matches[0]] = f"mame_sha256={relocated}"
lines.append(f"mame_prepack_sha256={original}")
lines.append("mame_packaging_transform=linuxdeploy-elf-relocation-verified")
manifest.write_text("\n".join(lines) + "\n")
PY
"$verifier" "$runtime"
offset=$("$appimage" --appimage-offset)
[[ "$offset" =~ ^[0-9]+$ && "$offset" -gt 0 ]] || { echo "Invalid AppImage offset: $offset" >&2; exit 1; }
mksquashfs "$scratch/extract/squashfs-root" "$scratch/payload.squashfs" -noappend -comp xz -processors 2 >/dev/null
head -c "$offset" "$appimage" >"$scratch/repacked.AppImage"
cat "$scratch/payload.squashfs" >>"$scratch/repacked.AppImage"
chmod --reference="$appimage" "$scratch/repacked.AppImage"
[[ "$("$scratch/repacked.AppImage" --appimage-offset)" == "$offset" ]] || {
  echo 'Repacked AppImage offset mismatch' >&2
  exit 1
}
(cd "$scratch/check" && "$scratch/repacked.AppImage" --appimage-extract >/dev/null)
mapfile -d '' -t checked < <(find "$scratch/check/squashfs-root" -type f -path '*/mame-runtime/bin/mame' -print0)
[[ ${#checked[@]} -eq 1 ]] || { echo 'Repacked AppImage missing MAME runtime' >&2; exit 1; }
"$verifier" "${checked[0]%/bin/mame}"
[[ "$(sha256sum "${checked[0]}" | awk '{print $1}')" == "$app_hash" ]] || {
  echo 'Repacked AppImage changed MAME executable bytes' >&2
  exit 1
}
mv -f "$scratch/repacked.AppImage" "$appimage"
printf 'AppImage relocation provenance verified: staged=%s packaged=%s\n' "$staged_hash" "$app_hash"
