#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/history"
capture_vqa "$OUT" "hist-missing-plugin-v1-1280" "doc-history-missing-plugin" "examples/demo-pack/sample.md" "1280x800"
