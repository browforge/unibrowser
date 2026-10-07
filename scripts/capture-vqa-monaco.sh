#!/usr/bin/env bash
# VQA-01/02/13: real Monaco light/dark TypeScript and Markdown evidence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"

OUT="$ROOT/docs/screenshots/vqa-monaco"
BIN="$ROOT/target/release/unibrowser-desktop"
mkdir -p "$OUT"

capture() {
  local name="$1"
  local theme="$2"
  local path="$3"
  local tmp="$OUT/.${name}.png"
  rm -f "$tmp"
  capture_config_prepare
  xvfb-run -a -s "-screen 0 1600x1000x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DEMO_STATE="capture-ready,theme-${theme}" \
    "$BIN" --open "$path" --doc-capture "$tmp"
  capture_config_cleanup
  mv "$tmp" "$OUT/${name}.png"
}

capture "typescript-light" "light" "$ROOT/examples/demo-pack/sample.ts"
capture "typescript-dark" "dark" "$ROOT/examples/demo-pack/sample.ts"
capture "markdown-light" "light" "$ROOT/examples/demo-pack/sample.md"
capture "markdown-dark" "dark" "$ROOT/examples/demo-pack/sample.md"

echo "VQA Monaco captures complete: $OUT"
