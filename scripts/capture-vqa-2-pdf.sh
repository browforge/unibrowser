#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa-2"
mkdir -p "$OUT"
capture_vqa "$OUT" "vqa-07-pdf-annotation-light" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.pdf"
capture_vqa "$OUT" "vqa-07-pdf-annotation-dark" "capture-ready,theme-dark,ai-sidebar" "/workspace/examples/demo-pack/sample.pdf"
capture_vqa "$OUT" "vqa-07-chrome-tabs-light" "capture-ready,theme-light" "/workspace/examples/demo-pack/sample.md"
echo "VQA-2 PDF captures: $OUT"
