#!/usr/bin/env bash
# Bear concept Render 1 — full 1280×800 markdown + docx evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="${ROOT}/docs/screenshots/txt-toolbar"
ART="${ART:-/opt/cursor/artifacts}"
mkdir -p "$OUT" "$ART"

MARKDOWN_DEMO="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,content-kind-md-preview"
DOCX_DEMO="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h1"

capture_vqa "$OUT" "bear-render-1-markdown-1280" "$MARKDOWN_DEMO" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "bear-render-1-docx-1280" "$DOCX_DEMO" "examples/demo-pack/sample-rich.docx"

cp -f "$OUT/bear-render-1-"*.png "$ART"/ 2>/dev/null || true
echo "Bear Render 1 captures in $OUT"
