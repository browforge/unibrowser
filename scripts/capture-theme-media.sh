#!/usr/bin/env bash
# Audio/video dark and high-contrast theme evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"

OUT="$ROOT/docs/screenshots/theme-media"
PACK="$ROOT/examples/demo-pack"
mkdir -p "$OUT"
rm -f "$OUT"/*.png "$OUT"/*.cm.json

capture_media() {
  local kind="$1"
  local path="$2"
  local theme="$3"
  local contrast="$4"
  local suffix="${theme}-1280"
  local contrast_token="standard-contrast"
  if [[ "$contrast" == "high" ]]; then
    suffix="${theme}-hc-1280"
    contrast_token="high-contrast"
  fi
  capture_vqa \
    "$OUT" \
    "${kind}-${suffix}" \
    "capture-ready,cm-compact,rail-collapsed,theme-${theme},${contrast_token},scale-100" \
    "$path" \
    "1280x800"
}

for theme in light dark; do
  capture_media "audio" "$PACK/sample.wav" "$theme" "high"
  capture_media "video" "$PACK/derived/live-record.mp4" "$theme" "high"
done
capture_media "audio" "$PACK/sample.wav" "dark" "standard"
capture_media "video" "$PACK/derived/live-record.mp4" "dark" "standard"

echo "Theme media captures in $OUT"
