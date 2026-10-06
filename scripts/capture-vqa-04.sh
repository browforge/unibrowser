#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
MD="/workspace/examples/demo-pack/sample.md"
for w in 800 1024 1280; do
  capture_vqa "$OUT" "vqa-04-text-${w}" "capture-ready,theme-light,ai-sidebar" "$MD" "${w}x800"
done
capture_vqa "$OUT" "vqa-04-pdf-1280" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.pdf"
capture_vqa "$OUT" "vqa-04-image-1280" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.svg"
echo "VQA-04 captures: $OUT"
