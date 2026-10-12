#!/usr/bin/env bash
# TXT-06 evidence: compact text toolbar at 1280x800 light.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="${ROOT}/docs/screenshots/txt-toolbar"
ART="${ART:-/opt/cursor/artifacts}"
mkdir -p "$OUT" "$ART"

echo "=== capture txt-web-markdown-picker-h2 (web build, Playwright) ==="
(
  cd "$ROOT/crates/unibrowser-desktop"
  npm run build:web
  npx playwright test tests/txt-toolbar-web-capture.spec.ts --config playwright.config.ts
)

capture_vqa "$OUT" "txt-desktop-markdown" "theme-light" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "txt-docx-emulated-h1" "theme-light,txt-block-picker-open" "examples/demo-pack/sample-rich.docx"
capture_vqa "$OUT" "txt-sample-mod-toolbar" "theme-light" "examples/demo-pack/sample.md"

cp -f "$OUT"/*.png "$ART"/
echo "Copied captures to $ART"
