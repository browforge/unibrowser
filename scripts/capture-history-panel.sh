#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/history"
capture_vqa "$OUT" "history-panel-v3-1280" "doc-history-panel,txt-apply-h2" "examples/demo-pack/sample.md" "1280x800"
capture_vqa "$OUT" "history-jump-v3-1280" "doc-history-jump,txt-apply-h2" "examples/demo-pack/sample.md" "1280x800"
python3 "$ROOT/scripts/check-history-capture-pixels.py"
