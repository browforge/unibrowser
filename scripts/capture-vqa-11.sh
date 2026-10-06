#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
capture_vqa "$OUT" "vqa-11-problems-ai-light" "capture-ready,theme-light,ai-sidebar,document-mockup-problems" "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "vqa-11-problems-ai-dark" "capture-ready,theme-dark,ai-sidebar,document-mockup-problems" "/workspace/examples/demo-pack/sample.md"
echo "VQA-11 captures: $OUT"
