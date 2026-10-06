#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"

OUT="$ROOT/docs/screenshots/vqa-3"
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
    \( "$MOCK/$light_mock" -resize 1280x800! \) \
    +append -strip "$OUT/${ticket}-${scenario}-side-by-side-light.png"
  convert "$OUT/${ticket}-${scenario}-dark.png" -resize 1280x800! \
    \( "$MOCK/$dark_mock" -resize 1280x800! \) \
    +append -strip "$OUT/${ticket}-${scenario}-side-by-side-dark.png"
}

capture_ticket "vqa-16" "new-tab" "rail-collapsed" "" "new-tab.png" "new-tab.png"
capture_ticket "vqa-17" "chrome-collapsed" "rail-collapsed,cm-collapsed" "$DEMO/sample.ts" "chrome-before-after.png" "chrome-before-after.png"
capture_ticket "vqa-24" "pdf-annotation" "rail-collapsed" "$DEMO/sample.pdf" "pdf-annotation.png" "pdf-annotation.png"
capture_ticket "vqa-24" "new-tab" "rail-collapsed" "" "new-tab.png" "new-tab.png"
capture_ticket "vqa-24" "image-svg" "rail-collapsed" "$DEMO/sample.svg" "command-system-gallery-light.png" "command-system-gallery-dark.png"
capture_ticket "vqa-18" "pdf-annotation" "rail-collapsed" "$DEMO/sample.pdf" "pdf-annotation.png" "pdf-annotation.png"

echo "Wrote captures under $OUT"
