#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/live"
mkdir -p "$OUT"
BASE="theme-light,text-card-render1,title-breadcrumb,rail-collapsed,no-ai-sidebar,doc-mode-wysiwyg"
SAMPLE="examples/demo-pack/live-block-sample.md"

capture_vqa "$OUT" "live-all-rendered-desktop" "${BASE},live-caret-outside" "$SAMPLE" || true
capture_vqa "$OUT" "live-heading-marker-desktop" "${BASE},live-caret-heading" "$SAMPLE" || true
capture_vqa "$OUT" "live-bold-marker-desktop" "${BASE},live-caret-bold" "$SAMPLE" || true

python3 "$ROOT/scripts/check-live-capture-pixels.py" "$OUT"
echo "Live block captures in $OUT"
