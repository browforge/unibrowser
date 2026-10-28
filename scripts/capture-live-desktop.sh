#!/usr/bin/env bash
# LIVE-01 desktop evidence (ProseMirror live block markers in Visual mode).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
npm run build --prefix "$ROOT/crates/unibrowser-desktop"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/live"
mkdir -p "$OUT"
BASE="capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100,cm-collapsed,text-card-render1,doc-mode-wysiwyg"
SAMPLE="examples/demo-pack/live-block-sample.md"

node --experimental-strip-types --test "$ROOT/crates/unibrowser-desktop/tests/markdownLiveBlock.test.mjs"

capture_vqa "$OUT" "live-cursor-outside" "${BASE},live-cursor-outside" "$SAMPLE" "1280x800"
capture_vqa "$OUT" "live-cursor-in-heading" "${BASE},live-cursor-in-heading" "$SAMPLE" "1280x800"
capture_vqa "$OUT" "live-cursor-in-bold" "${BASE},live-cursor-in-bold" "$SAMPLE" "1280x800"

python3 "$ROOT/scripts/check-live-capture-pixels.py" "$OUT"
echo "Live block captures in $OUT"
