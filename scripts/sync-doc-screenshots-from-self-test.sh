#!/usr/bin/env bash
# Copy latest self-test PNGs into docs/screenshots for README embeds.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/target/self-test-reports/self-test-shots"
OUT="$ROOT/docs/screenshots"
if [[ ! -d "$SRC" ]]; then
  echo "Run ./scripts/self-test.sh first" >&2
  exit 1
fi
mkdir -p "$OUT"
cp -f "$SRC/problems-jump.png" "$OUT/unibrowser-07-problems.png"
cp -f "$SRC/proofing-dictionary.png" "$OUT/unibrowser-07-problems.png" 2>/dev/null || true
echo "Synced self-test shots into $OUT"
