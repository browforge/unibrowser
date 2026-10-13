#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/doc-features"
BASE="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-collapsed,doc-visual-md-docx"
capture_vqa "$OUT" "doc-visual-docx-desktop-v6" "$BASE" "examples/demo-pack/sample.md"
