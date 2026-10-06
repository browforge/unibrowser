#!/usr/bin/env bash
# Bundled mods must not depend on the desktop shell or Tauri internals.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

fail=0
for toml in mods/*/Cargo.toml; do
  if grep -qE 'unibrowser-desktop|tauri' "$toml"; then
    echo "forbidden dependency in $toml"
    fail=1
  fi
done

for rs in mods/*/src/*.rs mods/*/src/**/*.rs; do
  [[ -f "$rs" ]] || continue
  if grep -qE 'unibrowser_desktop|unibrowser-desktop|tauri::' "$rs"; then
    echo "forbidden import in $rs"
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
echo "Bundled mod API boundaries OK"
