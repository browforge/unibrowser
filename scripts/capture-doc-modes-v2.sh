#!/usr/bin/env bash
# Document mode evidence (desktop + web) at 1280x800.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="${ROOT}/docs/screenshots/doc-modes"
ART="${ART:-/opt/cursor/artifacts}"
mkdir -p "$OUT" "$ART"

BASE_DEMO="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h2"

capture_vqa "$OUT" "doc-modes-wysiwyg-desktop-v2" "${BASE_DEMO},doc-mode-wysiwyg" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "doc-modes-source-desktop-v2" "${BASE_DEMO},doc-mode-source" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "doc-modes-split-desktop-v2" "${BASE_DEMO},doc-mode-split" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "doc-modes-picker-desktop-v2" "${BASE_DEMO},doc-mode-wysiwyg,txt-block-picker-open" "examples/demo-pack/sample.md"

echo "=== capture doc-modes web (Playwright) ==="
(
  cd "$ROOT/crates/unibrowser-desktop"
  npm run build:web
  npx playwright test tests/document-modes-capture.spec.ts --config playwright.config.ts
)

for name in \
  doc-modes-wysiwyg-desktop-v2.png \
  doc-modes-source-desktop-v2.png \
  doc-modes-split-desktop-v2.png \
  doc-modes-picker-desktop-v2.png \
  doc-modes-wysiwyg-web-v2.png \
  doc-modes-source-web-v2.png \
  doc-modes-split-web-v2.png \
  doc-modes-picker-web-v2.png; do
  if [[ -f "$OUT/$name" ]]; then
    cp -f "$OUT/$name" "$ART/$name"
  elif [[ -f "$ART/$name" ]]; then
    :
  else
    echo "warn: missing $name" >&2
  fi
done

echo "Document mode v2 captures in $OUT and $ART"
