#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"

OUT="$ROOT/docs/screenshots/vqa-final"
MOCK="$ROOT/docs/design/mockups/quiet-utility"
DEMO="$ROOT/examples/demo-pack"
mkdir -p "$OUT"

capture_pair() {
  local scenario="$1"
  local state="$2"
  local open_path="${3:-}"
  for theme in light dark; do
    capture_vqa \
      "$OUT" \
      "${scenario}-${theme}" \
      "capture-ready,theme-${theme},${state}" \
      "$open_path" \
      "1280x800"
    convert "$OUT/${scenario}-${theme}.png" \
      -resize 1280x800! -strip "$OUT/${scenario}-${theme}.png"
  done
}

capture_new_tab_pair() {
  for theme in light dark; do
    local tmp="$OUT/.new-tab-${theme}.png"
    capture_config_prepare
  xvfb-run -a -s "-screen 0 1280x800x24" env \
      WEBKIT_DISABLE_DMABUF_RENDERER=1 \
      WEBKIT_DISABLE_COMPOSITING_MODE=1 \
      UNIBROWSER_REPO_ROOT="$ROOT" \
      UNIBROWSER_SAFE_MODE=1 \
      UNIBROWSER_DEMO_STATE="theme-${theme},rail-collapsed,capture-ready" \
      "$BIN" --doc-capture "$tmp"
  capture_config_cleanup
    mv "$tmp" "$OUT/new-tab-${theme}.png"
    convert "$OUT/new-tab-${theme}.png" \
      -resize 1280x800! -strip "$OUT/new-tab-${theme}.png"
  done
}

compare_pair() {
  local scenario="$1"
  local light_mock="$2"
  local dark_mock="${3:-$light_mock}"
  convert "$OUT/${scenario}-light.png" -resize 1280x800! \
    -gravity NorthWest -pointsize 14 -fill '#1f2326' -annotate +12+8 'App' \
    \( "$MOCK/$light_mock" -resize 1280x800! \
      -gravity NorthWest -pointsize 14 -fill '#1f2326' -annotate +12+8 'Mockup' \) \
    +append -strip "$OUT/${scenario}-side-by-side-light.png"
  convert "$OUT/${scenario}-dark.png" -resize 1280x800! \
    -gravity NorthWest -pointsize 14 -fill '#e8eef2' -annotate +12+8 'App' \
    \( "$MOCK/$dark_mock" -resize 1280x800! \
      -gravity NorthWest -pointsize 14 -fill '#e8eef2' -annotate +12+8 'Mockup' \) \
    +append -strip "$OUT/${scenario}-side-by-side-dark.png"
}

capture_pair \
  "document-problems-ai" \
  "ai-sidebar,document-mockup-problems,rail-collapsed" \
  "$DEMO/sample.ts"
capture_pair \
  "pdf-annotation" \
  "rail-collapsed" \
  "$DEMO/sample.pdf"
capture_pair \
  "image-svg" \
  "rail-collapsed" \
  "$DEMO/sample.svg"
capture_pair \
  "rich-doc" \
  "rail-collapsed" \
  "$DEMO/sample-rich.docx"
capture_new_tab_pair
capture_pair \
  "rail-collapsed" \
  "ai-sidebar,rail-collapsed" \
  "$DEMO/sample.ts"
capture_pair \
  "rail-expanded" \
  "ai-sidebar,rail-expanded" \
  "$DEMO/sample.ts"
capture_pair \
  "chrome-collapsed" \
  "ai-sidebar,rail-collapsed,cm-collapsed" \
  "$DEMO/sample.ts"

compare_pair \
  "document-problems-ai" \
  "document-problems-ai.png" \
  "document-problems-ai-dark.png"
compare_pair "pdf-annotation" "pdf-annotation.png"
compare_pair \
  "image-svg" \
  "command-system-gallery-light.png" \
  "command-system-gallery-dark.png"
compare_pair \
  "rich-doc" \
  "document-command-large.png" \
  "command-system-gallery-dark.png"
compare_pair "new-tab" "new-tab.png"
compare_pair "rail-collapsed" "document-command-compact.png"
compare_pair "rail-expanded" "document-command-large.png"
compare_pair "chrome-collapsed" "chrome-before-after.png"

echo "VQA final captures: $OUT"
