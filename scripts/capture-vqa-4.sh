#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"

OUT="$ROOT/docs/screenshots/vqa-4"
MOCK="$ROOT/docs/design/mockups/quiet-utility"
DEMO="$ROOT/examples/demo-pack"
mkdir -p "$OUT"

capture_ticket() {
  local ticket="$1"
  local scenario="$2"
  local state="$3"
  local open_path="${4:-}"
  local light_mock="${5:-}"
  local dark_mock="${6:-$light_mock}"
  for theme in light dark; do
    capture_vqa \
      "$OUT" \
      "${ticket}-${scenario}-${theme}" \
      "capture-ready,theme-${theme},${state}" \
      "$open_path" \
      "1280x800"
    convert "$OUT/${ticket}-${scenario}-${theme}.png" \
      -resize 1280x800! -strip "$OUT/${ticket}-${scenario}-${theme}.png"
  done
  convert "$OUT/${ticket}-${scenario}-light.png" -resize 1280x800! \
    -gravity NorthWest -pointsize 14 -fill '#1f2326' -annotate +12+8 'App' \
    \( "$MOCK/$light_mock" -resize 1280x800! \
      -gravity NorthWest -pointsize 14 -fill '#1f2326' -annotate +12+8 'Mockup' \) \
    +append -strip "$OUT/${ticket}-${scenario}-side-by-side-light.png"
  convert "$OUT/${ticket}-${scenario}-dark.png" -resize 1280x800! \
    -gravity NorthWest -pointsize 14 -fill '#e8eef2' -annotate +12+8 'App' \
    \( "$MOCK/$dark_mock" -resize 1280x800! \
      -gravity NorthWest -pointsize 14 -fill '#e8eef2' -annotate +12+8 'Mockup' \) \
    +append -strip "$OUT/${ticket}-${scenario}-side-by-side-dark.png"
}

capture_ticket "vqa-19" "pdf-workspace" "rail-collapsed" "$DEMO/sample.pdf" "pdf-annotation.png" "pdf-annotation.png"
capture_ticket "vqa-20" "image-svg" "rail-collapsed" "$DEMO/sample.svg" "command-system-gallery-light.png" "command-system-gallery-dark.png"
capture_ticket "vqa-21" "rich-document" "rail-collapsed" "$DEMO/sample-rich.docx" "document-problems-ai.png" "document-problems-ai-dark.png"
capture_ticket "vqa-22" "rail-expanded" "rail-expanded" "$DEMO/sample.pdf" "document-problems-ai.png" "document-problems-ai-dark.png"
capture_ticket "vqa-25" "document-shell" "rail-collapsed,cm-collapsed" "$DEMO/sample.ts" "document-problems-ai.png" "document-problems-ai-dark.png"

echo "Wrote captures under $OUT"
