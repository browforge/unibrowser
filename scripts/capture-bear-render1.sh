#!/usr/bin/env bash
# Bear concept Render 1 — desktop markdown/docx + web markdown evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="${ROOT}/docs/screenshots/txt-toolbar"
ART="${ART:-/opt/cursor/artifacts}"
mkdir -p "$OUT" "$ART"

MARKDOWN_DEMO="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h2,txt-block-picker-open"
DOCX_DEMO="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h1,txt-block-picker-open"

capture_vqa "$OUT" "bear-render-1-markdown-1280" "$MARKDOWN_DEMO" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "bear-render-1-docx-1280" "$DOCX_DEMO" "examples/demo-pack/sample-rich.docx"

echo "=== capture txt-web-markdown-picker-h2 (web build, Playwright) ==="
(
  cd "$ROOT/crates/unibrowser-desktop"
  npm run build:web
  UB_WEB_DEMO="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h2,txt-block-picker-open" \
    npx playwright test tests/txt-toolbar-web-capture.spec.ts --config playwright.config.ts
)

copy_artifact_retry() {
  local src="$1"
  local dst="$2"
  local attempt=1
  while [[ "$attempt" -le 8 ]]; do
    if cp -f "$src" "$dst" 2>/dev/null; then
      return 0
    fi
    sleep "$attempt"
    attempt=$((attempt + 1))
  done
  echo "warn: could not copy $src -> $dst after retries" >&2
  return 1
}

for pair in \
  "bear-render-1-markdown-1280.png:bear-render-1-markdown-1280-v2.png" \
  "bear-render-1-docx-1280.png:bear-render-1-docx-1280-v2.png" \
  "bear-render-1-markdown-1280.cm.json:bear-render-1-markdown-1280-v2.cm.json" \
  "bear-render-1-docx-1280.cm.json:bear-render-1-docx-1280-v2.cm.json" \
  "txt-web-markdown-picker-h2-v2.png:txt-web-markdown-picker-h2-v2.png"; do
  src="${pair%%:*}"
  dst="${pair##*:}"
  if [[ -f "$OUT/$src" ]]; then
    copy_artifact_retry "$OUT/$src" "$ART/$dst" || true
  fi
done

python3 "$ROOT/scripts/write-bear-defect-checklist.py" "$ART" || true

echo "Bear Render 1 captures in $OUT (artifacts in $ART)"
