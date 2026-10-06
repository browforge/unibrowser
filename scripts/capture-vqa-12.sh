#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
capture_vqa "$OUT" "vqa-12-text-100" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.md" "1280x800"
capture_vqa "$OUT" "vqa-12-text-150" "capture-ready,theme-light,ai-sidebar,scale-150" "/workspace/examples/demo-pack/sample.md" "1280x800"
echo "VQA-12 captures: $OUT"
