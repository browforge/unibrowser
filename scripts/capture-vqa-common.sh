#!/usr/bin/env bash
# Shared helpers for docs/screenshots/vqa/ evidence captures.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
BIN="$ROOT/target/release/unibrowser-desktop"

capture_vqa() {
  local outdir="$1"
  local name="$2"
  local demo="$3"
  local open_path="$4"
  local geometry="${5:-1280x800}"
  mkdir -p "$outdir"
  local tmp="$outdir/.${name}.png"
  rm -f "$tmp"
  echo "=== capture ${name} (${geometry}) ==="
  xvfb-run -a -s "-screen 0 ${geometry}x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DEMO_STATE="$demo" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"
  mv "$tmp" "$outdir/${name}.png"
  echo "wrote $outdir/${name}.png"
}
