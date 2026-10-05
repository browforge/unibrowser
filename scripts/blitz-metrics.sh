#!/usr/bin/env bash
# Print workspace counts for docs/blitz-status.md (Wed report).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

rust_files() {
  find crates mods -name '*.rs' -not -path '*/target/*' 2>/dev/null | wc -l | tr -d ' '
}

rust_lines() {
  find crates mods -name '*.rs' -not -path '*/target/*' -print0 2>/dev/null \
    | xargs -0 wc -l 2>/dev/null | tail -n 1 | awk '{print $1}'
}

ts_files() {
  find crates/unibrowser-desktop/src -name '*.ts' 2>/dev/null | wc -l | tr -d ' '
}

ts_lines() {
  if ! find crates/unibrowser-desktop/src -name '*.ts' -print -quit 2>/dev/null | grep -q .; then
    echo 0
    return
  fi
  find crates/unibrowser-desktop/src -name '*.ts' -print0 2>/dev/null \
    | xargs -0 wc -l 2>/dev/null | tail -n 1 | awk '{print $1}'
}

mod_packages() {
  find mods -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' '
}

adr_count() {
  find docs/adr -name '*.md' 2>/dev/null | wc -l | tr -d ' '
}

workspace_members() {
  awk '/^members = \[/,/^\]/ { if ($0 ~ /"/) print }' Cargo.toml | wc -l | tr -d ' '
}

echo "Blitz metrics ($(date -u +%Y-%m-%dT%H:%MZ))"
echo "rust_files=$(rust_files)"
echo "rust_lines=$(rust_lines)"
echo "ts_files=$(ts_files)"
echo "ts_lines=$(ts_lines)"
echo "mod_packages=$(mod_packages)"
echo "adr_count=$(adr_count)"
echo "workspace_members=$(workspace_members)"
