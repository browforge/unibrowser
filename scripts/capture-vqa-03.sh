#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
capture_vqa "$OUT" "vqa-03-pdf-context" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.pdf"
capture_vqa "$OUT" "vqa-03-svg-context" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.svg"
echo "VQA-03 captures: $OUT"
