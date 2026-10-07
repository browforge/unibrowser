#!/usr/bin/env bash
# Light-theme responsive shell evidence at the two review viewports.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
(cd "$ROOT/crates/unibrowser-desktop" && npm run test:contrast)
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"

OUT="$ROOT/docs/screenshots/responsive-and-states"
PACK="$ROOT/examples/demo-pack"
mkdir -p "$OUT"
rm -f "$OUT"/*-light-1024.png "$OUT"/*-light-1024.cm.json
rm -f "$OUT"/*-light-1920.png "$OUT"/*-light-1920.cm.json

capture_new_tab() {
  local suffix="$1"
  local geometry="$2"
  local tmp="$OUT/.new-tab-light-${suffix}.png"
  capture_config_prepare
  xvfb-run -a -s "-screen 0 ${geometry}x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="$geometry" \
    UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-light,standard-contrast,scale-100" \
    "$BIN" --doc-capture "$tmp"
  capture_config_cleanup
  mv "$tmp" "$OUT/new-tab-light-${suffix}.png"
  finalise_cm_capture "$OUT" "new-tab-light-${suffix}"
}

capture_surface() {
  local name="$1"
  local demo="$2"
  local path="$3"
  local suffix="$4"
  local geometry="$5"
  capture_vqa \
    "$OUT" \
    "${name}-light-${suffix}" \
    "capture-ready,cm-compact,rail-collapsed,theme-light,standard-contrast,scale-100,${demo}" \
    "$path" \
    "$geometry"
}

for viewport in "1024:1024x700" "1920:1920x1080"; do
  suffix="${viewport%%:*}"
  geometry="${viewport#*:}"
  capture_new_tab "$suffix" "$geometry"
  capture_surface "file-browser" "ui26-file-preview" "$PACK" "$suffix" "$geometry"
  capture_surface "settings" "settings" "$PACK/sample.txt" "$suffix" "$geometry"
  capture_surface "markdown" "document-mockup-problems,ai" "$PACK/sample.md" "$suffix" "$geometry"
  capture_surface "pdf" "" "$PACK/sample.pdf" "$suffix" "$geometry"
  capture_surface "image" "" "$PACK/sample.png" "$suffix" "$geometry"
  capture_surface "svg" "" "$PACK/sample.svg" "$suffix" "$geometry"
  capture_surface "dxf" "" "$PACK/sample.dxf" "$suffix" "$geometry"
  capture_surface "workshop" "" "$PACK/sample.unibrow" "$suffix" "$geometry"
  capture_surface "mod-portal" "ui17-mod-portal" "$PACK/sample.step" "$suffix" "$geometry"
  capture_surface "audio" "" "$PACK/sample.wav" "$suffix" "$geometry"
  capture_surface "video" "" "$PACK/derived/live-record.mp4" "$suffix" "$geometry"
done

echo "Responsive shell captures in $OUT"
