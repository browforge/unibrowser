#!/usr/bin/env bash
# UI-15 .unibrow Create workspace evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/ui-15"
mkdir -p "$OUT"
rm -f "$OUT"/*.png
BIN="$ROOT/target/release/unibrowser-desktop"
PACK="$ROOT/examples/demo-pack"

capture() {
  local name="$1"
  local theme="$2"
  local dimensions="$3"
  local tmp="$OUT/.${name}.png"
  rm -f "$tmp"
  xvfb-run -a -s "-screen 0 ${dimensions}x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="$dimensions" \
    UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-${theme},standard-contrast,scale-100,ui15-unibrow-title,ui16-live-preview" \
    "$BIN" --open "$PACK/sample.unibrow" --doc-capture "$tmp"
  mv "$tmp" "$OUT/${name}.png"
  finalise_cm_capture "$OUT" "$name"
  local actual
  actual="$(identify -format '%wx%h' "$OUT/${name}.png")"
  if [[ "$actual" != "$dimensions" ]]; then
    echo "capture dimensions differ: ${name} is ${actual}, expected ${dimensions}" >&2
    exit 1
  fi
}

capture "unibrow-create-light-1600" "light" "1600x1000"
capture "unibrow-create-dark-1600" "dark" "1600x1000"
capture "unibrow-create-light-1280" "light" "1280x800"
capture "unibrow-create-dark-1280" "dark" "1280x800"

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

echo "UI-15 captures in $OUT"
