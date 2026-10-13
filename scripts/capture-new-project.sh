#!/usr/bin/env bash
# ADR 0036 / NEW-10: New tab page + template gallery at 1280x800.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/new-project"
ART="/opt/cursor/artifacts"
mkdir -p "$OUT" "$ART"
rm -f "$OUT"/*-1280.png

capture_one() {
  local name="$1"
  local demo="$2"
  local tmp="$OUT/.${name}.png"
  capture_config_prepare
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100,${demo}" \
    "$BIN" --doc-capture "$tmp"
  capture_config_cleanup
  mv "$tmp" "$OUT/${name}-1280.png"
  cp "$OUT/${name}-1280.png" "$ART/${name}-1280-v1.png"
  [[ "$(identify -format '%wx%h' "$OUT/${name}-1280.png")" == "1280x800" ]]
}

capture_one "new-tab-paper-card" "new-project-capture-ntp"
capture_one "template-gallery" "new-project-capture-gallery"
python3 "$ROOT/scripts/check-new-project-capture-pixels.py"
