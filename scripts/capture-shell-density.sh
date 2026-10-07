#!/usr/bin/env bash
# Quiet Utility shell density matrix (1280x800 light + dark).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
(cd "$ROOT/crates/unibrowser-desktop" && npm run test:contrast)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/shell-density"
DEMO="$ROOT/examples/demo-pack"
mkdir -p "$OUT"
rm -f "$OUT"/*.png

capture_scene() {
  local name="$1"
  local demo="$2"
  local open_path="$3"
  for theme in light dark; do
    capture_vqa \
      "$OUT" \
      "${name}-${theme}-1280" \
      "capture-ready,cm-compact,rail-collapsed,theme-${theme},standard-contrast,scale-100,${demo}" \
      "$open_path" \
      "1280x800"
    [[ "$(identify -format '%wx%h' "$OUT/${name}-${theme}-1280.png")" == "1280x800" ]]
  done
}

capture_new_tab() {
  for theme in light dark; do
    for contrast in standard high; do
      local suffix="${theme}-1280"
      local contrast_token="standard-contrast"
      if [[ "$contrast" == "high" ]]; then
        suffix="${theme}-hc-1280"
        contrast_token="high-contrast"
      fi
      local tmp="$OUT/.new-tab-${suffix}.png"
    capture_config_prepare
    xvfb-run -a -s "-screen 0 1280x800x24" env \
      WEBKIT_DISABLE_DMABUF_RENDERER=1 \
      WEBKIT_DISABLE_COMPOSITING_MODE=1 \
      UNIBROWSER_REPO_ROOT="$ROOT" \
      UNIBROWSER_SAFE_MODE=1 \
      UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
      UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-${theme},${contrast_token},scale-100" \
      "$BIN" --doc-capture "$tmp"
    capture_config_cleanup
      mv "$tmp" "$OUT/new-tab-${suffix}.png"
      finalise_cm_capture "$OUT" "new-tab-${suffix}"
      [[ "$(identify -format '%wx%h' "$OUT/new-tab-${suffix}.png")" == "1280x800" ]]
    done
  done
}

capture_scene_hc() {
  local name="$1"
  local demo="$2"
  local open_path="$3"
  for theme in light dark; do
    capture_vqa \
      "$OUT" \
      "${name}-${theme}-hc-1280" \
      "capture-ready,cm-compact,rail-collapsed,theme-${theme},high-contrast,scale-100,${demo}" \
      "$open_path" \
      "1280x800"
    [[ "$(identify -format '%wx%h' "$OUT/${name}-${theme}-hc-1280.png")" == "1280x800" ]]
  done
}

capture_new_tab
capture_scene "file-browser" "ui26-file-preview" "$DEMO"
capture_scene_hc "file-browser" "ui26-file-preview" "$DEMO"
capture_scene "markdown" "document-mockup-problems,ai" "$DEMO/sample.md"
capture_scene_hc "markdown" "document-mockup-problems,ai" "$DEMO/sample.md"
capture_scene "pdf" "" "$DEMO/sample.pdf"
capture_scene_hc "pdf" "" "$DEMO/sample.pdf"
capture_scene "image" "" "$DEMO/sample.png"
capture_scene_hc "image" "" "$DEMO/sample.png"
capture_scene "workshop" "" "$DEMO/sample.unibrow"
capture_scene_hc "workshop" "" "$DEMO/sample.unibrow"
capture_scene "settings" "settings" "$DEMO/sample.txt"
capture_scene_hc "settings" "settings" "$DEMO/sample.txt"
capture_scene "mod-portal" "ui17-mod-portal" "$DEMO/sample.step"
capture_scene_hc "mod-portal" "ui17-mod-portal" "$DEMO/sample.step"

echo "Shell density captures in $OUT"
