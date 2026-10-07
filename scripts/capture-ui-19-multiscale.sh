#!/usr/bin/env bash
# UI-19 automated multi-scale high-contrast captures (doc-capture aborts on contrast failure).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/ui-19"
mkdir -p "$OUT"
BIN="$ROOT/target/release/unibrowser-desktop"
FIXTURE="$ROOT/examples/demo-pack/sample.txt"
[[ -f "$FIXTURE" ]]

for scale in 100 125 150 200; do
  for theme in light dark; do
    name="high-contrast-${theme}-${scale}-1280"
    tmp="$OUT/.${name}.png"
    capture_config_prepare
  xvfb-run -a -s "-screen 0 1280x800x24" env \
      WEBKIT_DISABLE_DMABUF_RENDERER=1 \
      WEBKIT_DISABLE_COMPOSITING_MODE=1 \
      UNIBROWSER_REPO_ROOT="$ROOT" \
      UNIBROWSER_SAFE_MODE=1 \
      UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
      UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-${theme},high-contrast,scale-${scale},ui19-focus-visible" \
      "$BIN" --open "$FIXTURE" --doc-capture "$tmp"
  capture_config_cleanup
    mv "$tmp" "$OUT/${name}.png"
    finalise_cm_capture "$OUT" "$name"
    [[ "$(identify -format '%wx%h' "$OUT/${name}.png")" == "1280x800" ]]
  done
done

echo "UI-19 multi-scale captures in $OUT"
