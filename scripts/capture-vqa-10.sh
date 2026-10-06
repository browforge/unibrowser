#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
capture_vqa "$OUT" "vqa-10-ai-light" "capture-ready,theme-light,ai-sidebar" "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "vqa-10-ai-dark" "capture-ready,theme-dark,ai-sidebar" "/workspace/examples/demo-pack/sample.md"
echo "VQA-10 captures: $OUT"
