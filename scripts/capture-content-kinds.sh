#!/usr/bin/env bash
# Content-kind capability screenshots (docs/screenshots/content-kinds/).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"

OUT="$ROOT/docs/screenshots/content-kinds"
PACK="$ROOT/examples/demo-pack"
BIN="$ROOT/target/release/unibrowser-desktop"
mkdir -p "$OUT"

BASE_DEMO="capture-ready,cm-compact,rail-collapsed,theme-light,standard-contrast,scale-100,content-kind-grayscale-focus"

capture_kind() {
  local out_name="$1"
  local open_path="$2"
  local demo_extra="$3"
  local embedded="${4:-0}"
  local tmp="$OUT/.${out_name}.png"
  rm -f "$tmp"
  capture_config_prepare
  if ! xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_SELECT_EMBEDDED_IMAGE="$embedded" \
    UNIBROWSER_DEMO_STATE="${BASE_DEMO}${demo_extra}" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"; then
    capture_config_cleanup
    return 1
  fi
  capture_config_cleanup
  mv "$tmp" "$OUT/${out_name}.png"
  finalise_cm_capture "$OUT" "${out_name}"
  local actual
  actual="$(identify -format '%wx%h' "$OUT/${out_name}.png")"
  if [[ "$actual" != "1280x800" ]]; then
    echo "capture dimensions differ: ${out_name} is ${actual}, expected 1280x800" >&2
    exit 1
  fi
  echo "wrote $OUT/${out_name}.png"
}

capture_kind "grayscale-standalone-png-1280-light" "$PACK/sample.png" "" "0"
capture_kind "grayscale-embedded-docx-1280-light" "$PACK/sample-rich.docx" "" "1"
capture_kind "grayscale-embedded-markdown-1280-light" "$PACK/sample.md" "" "1"

echo "Content-kind captures in $OUT"
