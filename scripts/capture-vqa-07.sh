#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
capture_vqa "$OUT" "vqa-07-pdf-annotation" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.pdf"
echo "VQA-07 captures: $OUT"
