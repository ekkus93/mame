#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <mame-executable>" >&2
  exit 64
fi

mame=$1
[[ -f "$mame" && -x "$mame" ]] || {
  echo "real MAME executable is missing or non-executable: $mame" >&2
  exit 1
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

version_output=$(timeout 20 "$mame" -noreadconfig -version 2>&1) || {
  echo "real MAME -version probe failed" >&2
  printf '%s\n' "$version_output" >&2
  exit 1
}
version_line=$(printf '%s\n' "$version_output" | head -n 1 | tr -d '\r')
if ! grep -Eq '^[0-9]+\.[0-9]+' <<<"$version_line"; then
  echo "runtime does not report a real MAME version: $version_line" >&2
  exit 1
fi
if grep -Eiq 'synthetic|fixture payload|MT-1305' <<<"$version_output"; then
  echo "synthetic runtime cannot satisfy real MAME qualification" >&2
  exit 1
fi

(
  cd "$tmp"
  timeout 60 "$mame" -noreadconfig -listxml pacman >listxml.xml 2>listxml.err
) || {
  echo "real MAME bounded listxml smoke failed" >&2
  cat "$tmp/listxml.err" >&2 || true
  exit 1
}
grep -Eq '<mame[^>]+build=' "$tmp/listxml.xml" || {
  echo "real MAME listxml output has no build identity" >&2
  cat "$tmp/listxml.err" >&2 || true
  exit 1
}
grep -Eq '<machine[^>]+name="pacman"' "$tmp/listxml.xml" || {
  echo "real MAME listxml smoke did not contain pacman" >&2
  exit 1
}

printf 'Qualified real MAME runtime: %s\n' "$version_line"
