#!/usr/bin/env bash
# UI-19 high-contrast and 150% scale evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/ui-19"
mkdir -p "$OUT"
rm -f "$OUT"/*.png
BIN="$ROOT/target/release/unibrowser-desktop"
FIXTURE="$ROOT/examples/demo-pack/sample.txt"
[[ -f "$FIXTURE" ]]

capture() {
  local theme="$1"
  local name="high-contrast-${theme}-150-1280"
  local tmp="$OUT/.high-contrast-${theme}-150-1280.png"
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-${theme},high-contrast,scale-150,ui19-focus-visible" \
    "$BIN" --open "$FIXTURE" --doc-capture "$tmp"
  mv "$tmp" "$OUT/high-contrast-${theme}-150-1280.png"
  finalise_cm_capture "$OUT" "$name"
  [[ "$(identify -format '%wx%h' "$OUT/high-contrast-${theme}-150-1280.png")" == "1280x800" ]]
}

capture "light"
capture "dark"

if [[ "$(sha256sum "$OUT"/*.png | cut -d' ' -f1 | sort | uniq -d | wc -l)" != "0" ]]; then
  echo "duplicate UI-19 captures" >&2
  exit 1
fi

echo "UI-19 captures in $OUT"
