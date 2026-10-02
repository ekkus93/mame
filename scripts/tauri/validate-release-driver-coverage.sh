#!/usr/bin/env bash
set -euo pipefail

[[ $# == 1 && -x "$1" ]] || {
  echo 'usage: validate-release-driver-coverage.sh <built-mame-executable>' >&2
  exit 64
}
runtime=$1
# A real ELF is insufficient: tiny is a real binary but lacks normal arcade drivers.
for machine in pacman sf2 galaga contra asteroid gauntlet robotron tapper rtype; do
  output=$("$runtime" -noreadconfig -listfull "$machine") || {
    echo "Release runtime does not support required arcade machine: $machine" >&2
    exit 1
  }
  grep -Eq "^${machine}[[:space:]]" <<<"$output" || {
    echo "Release runtime did not report exact machine: $machine" >&2
    exit 1
  }
done
echo 'Release runtime includes required arcade driver coverage'
