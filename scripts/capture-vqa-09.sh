#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/vqa"
capture_vqa "$OUT" "vqa-09-rail-collapsed-light" "capture-ready,theme-light,ai-sidebar,rail-collapsed" "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "vqa-09-rail-expanded-light" "capture-ready,theme-light,ai-sidebar,rail-expanded" "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "vqa-09-rail-collapsed-dark" "capture-ready,theme-dark,ai-sidebar,rail-collapsed" "/workspace/examples/demo-pack/sample.md"
capture_vqa "$OUT" "vqa-09-rail-expanded-dark" "capture-ready,theme-dark,ai-sidebar,rail-expanded" "/workspace/examples/demo-pack/sample.md"
echo "VQA-09 captures: $OUT"
