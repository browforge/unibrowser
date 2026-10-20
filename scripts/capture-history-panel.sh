#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/history"
capture_vqa "$OUT" "history-panel-v4-1280" "doc-history-panel" "examples/demo-pack/sample.md" "1280x800"
capture_vqa "$OUT" "history-jump-v4-1280" "doc-history-jump" "examples/demo-pack/sample.md" "1280x800"
python3 "$ROOT/scripts/check-history-capture-pixels.py"
