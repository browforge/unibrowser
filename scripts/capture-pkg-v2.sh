#!/usr/bin/env bash
# PKG-03..06 UI evidence at 1280x800 (desktop doc-capture only).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/pkg-v2"
ART="/opt/cursor/artifacts"
mkdir -p "$OUT" "$ART"
rm -f "$OUT"/*-v2-1280.png "$OUT"/*-v2-1280.cm.json

capture_desktop() {
  local name="$1"
  local demo="$2"
  local open_path="${3:-}"
  local tmp="$OUT/.${name}.png"
  local demo_full="capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100,cm-collapsed,${demo}"
  capture_config_prepare
  local open_args=()
  if [[ -n "$open_path" ]]; then
    open_args=(--open "$open_path")
  fi
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="$demo_full" \
    "$BIN" "${open_args[@]}" --doc-capture "$tmp"
  capture_config_cleanup
  mv "$tmp" "$OUT/${name}-v2-1280.png"
  cp "$OUT/${name}-v2-1280.png" "$ART/${name}-v2-1280.png" || true
  local cm_tmp="$OUT/.${name}.cm.json"
  if [[ -f "$cm_tmp" ]]; then
    mv "$cm_tmp" "$OUT/${name}-v2-1280.cm.json"
  fi
  [[ "$(identify -format '%wx%h' "$OUT/${name}-v2-1280.png")" == "1280x800" ]]
}

capture_desktop "open-mode-dialog" "pkg-v2-open-mode"
capture_desktop "history-location-notice" "pkg-v2-history-notice"
capture_desktop "unpack-progress" "pkg-v2-unpack-progress"
capture_desktop "file-menu-pack-unpack" "pkg-v2-file-menu"

python3 "$ROOT/scripts/check-pkg-v2-capture-pixels.py"
