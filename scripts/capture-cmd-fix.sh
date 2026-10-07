#!/usr/bin/env bash
# CMD-fix evidence captures (validates CM + Monaco before write).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"
OUT="$ROOT/docs/screenshots/cmd-fix"
mkdir -p "$OUT"
BIN="$ROOT/target/release/unibrowser-desktop"

capture() {
  local name="$1"
  local demo="$2"
  local open_path="$3"
  local tmp="$OUT/.${name}.png"
  rm -f "$tmp"
  echo "=== capture ${name} ==="
  capture_config_prepare
  xvfb-run -a -s "-screen 0 1600x1000x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DEMO_STATE="$demo" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"
  capture_config_cleanup
  mv "$tmp" "$OUT/${name}.png"
  echo "wrote $OUT/${name}.png"
}

capture "text-family-plain-txt" "capture-ready,theme-light" "/workspace/examples/demo-pack/sample.txt"
capture "text-family-markdown" "capture-ready,theme-light" "/workspace/examples/demo-pack/sample.md"
capture "text-family-rich-docx" "capture-ready,theme-light" "/workspace/examples/demo-pack/sample-rich.docx"
capture "text-family-dark" "capture-ready,theme-dark" "/workspace/examples/demo-pack/sample.md"
capture "pdf-annotation-cm" "capture-ready,theme-light" "/workspace/examples/demo-pack/sample.pdf"
capture "image-cm" "capture-ready,theme-light" "/workspace/examples/demo-pack/sample.svg"

echo "CMD-fix captures complete: $OUT"
