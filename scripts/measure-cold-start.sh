#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== unibrowser cold-start probe ==="
echo "date: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo ""

echo "--- cargo test -p unibrowser-engine ---"
/usr/bin/time -f "wall %e s" cargo test -p unibrowser-engine --quiet

echo ""
echo "--- npm run build (desktop UI) ---"
cd "$ROOT/crates/unibrowser-desktop"
if [[ -f package-lock.json ]]; then
  npm ci --silent
else
  npm install --silent
fi
/usr/bin/time -f "wall %e s" npm run build --silent

echo ""
echo "Done. Record these numbers in docs/perf.md comparisons."
