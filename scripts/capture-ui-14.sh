#!/usr/bin/env bash
# UI-14 PDF, image, and DXF context shell evidence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"

OUT="$ROOT/docs/screenshots/ui-14"
BIN="$ROOT/target/release/unibrowser-desktop"
PACK="$ROOT/examples/demo-pack"
mkdir -p "$OUT"

capture() {
  local name="$1"
  local open_path="$2"
  local extra_state="${3:-}"
  local theme="${4:-light}"
  local contrast="standard-contrast"
  if [[ ",${extra_state}," == *",high-contrast,"* ]] || [[ "$extra_state" == "high-contrast" ]]; then
    contrast="high-contrast"
  fi
  local tmp="$OUT/.${name}.png"
  rm -f "$tmp"
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-${theme},${contrast},scale-100${extra_state:+,$extra_state}" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"
  mv "$tmp" "$OUT/${name}.png"
  local actual
  actual="$(identify -format '%wx%h' "$OUT/${name}.png")"
  if [[ "$actual" != "1280x800" ]]; then
    echo "capture dimensions differ: ${name} is ${actual}, expected 1280x800" >&2
    exit 1
  fi
}

capture "pdf-annotation-light-1280" "$PACK/sample.pdf"
capture "pdf-annotation-100-light-1280" "$PACK/sample.pdf" "ui14-pdf-100"
capture "pdf-annotation-overflow-light-1280" "$PACK/sample.pdf" "ui14-open-command-overflow"
capture "image-light-1280" "$PACK/sample.svg"
capture "image-raster-light-1280" "$PACK/sample.png"
capture "image-raster-dark-1280" "$PACK/sample.png" "" "dark"
capture "image-raster-hc-1280" "$PACK/sample.png" "high-contrast" "light"
capture "dxf-proving-light-1280" "$PACK/sample.dxf" "ui14-dxf-measure-reason"

declare -A seen_hashes=()
for png in "$OUT"/*.png; do
  hash="$(sha256sum "$png" | cut -d' ' -f1)"
  name="$(basename "$png")"
  if [[ -n "${seen_hashes[$hash]:-}" ]]; then
    echo "duplicate capture hash: ${seen_hashes[$hash]} and ${name} (${hash})" >&2
    exit 1
  fi
  seen_hashes[$hash]="$name"
done

echo "UI-14 captures in $OUT"
