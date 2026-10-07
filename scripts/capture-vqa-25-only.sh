#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa-4"
MOCK="$ROOT/docs/design/mockups/quiet-utility"
DEMO="$ROOT/examples/demo-pack"
for theme in light dark; do
  capture_vqa "$OUT" "vqa-25-document-shell-${theme}" "capture-ready,theme-${theme},rail-collapsed,cm-collapsed" "$DEMO/sample.ts" "1280x800"
  convert "$OUT/vqa-25-document-shell-${theme}.png" -resize 1280x800! -strip "$OUT/vqa-25-document-shell-${theme}.png"
done
convert "$OUT/vqa-25-document-shell-light.png" -resize 1280x800! \
  -gravity NorthWest -pointsize 14 -fill '#1f2326' -annotate +12+8 'App' \
  \( "$MOCK/document-problems-ai.png" -resize 1280x800! \
    -gravity NorthWest -pointsize 14 -fill '#1f2326' -annotate +12+8 'Mockup' \) \
  +append -strip "$OUT/vqa-25-document-shell-side-by-side-light.png"
convert "$OUT/vqa-25-document-shell-dark.png" -resize 1280x800! \
  -gravity NorthWest -pointsize 14 -fill '#e8eef2' -annotate +12+8 'App' \
  \( "$MOCK/document-problems-ai-dark.png" -resize 1280x800! \
    -gravity NorthWest -pointsize 14 -fill '#e8eef2' -annotate +12+8 'Mockup' \) \
  +append -strip "$OUT/vqa-25-document-shell-side-by-side-dark.png"
echo "wrote vqa-25 under $OUT"
