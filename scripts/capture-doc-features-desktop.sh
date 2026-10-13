#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
ART="${ART:-/opt/cursor/artifacts}"
mkdir -p "$ART"
OUT="$ROOT/docs/screenshots/doc-features"
mkdir -p "$OUT"
BASE="theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-collapsed"

capture_vqa "$OUT" "doc-wysiwyg-blockquote-desktop-v5" "${BASE},doc-mode-wysiwyg,txt-apply-h2" "examples/demo-pack/sample.md" || true
capture_vqa "$OUT" "doc-split-apply-banner-desktop-v5" "${BASE},doc-mode-split,doc-merge-banner" "examples/demo-pack/sample.md" || true
capture_vqa "$OUT" "doc-opaque-card-desktop-v5" "${BASE},doc-opaque-card" "examples/demo-pack/sample-opaque.md" || true
capture_vqa "$OUT" "doc-visual-docx-desktop-v6" "$BASE" "examples/demo-pack/sample.md" || true
capture_vqa "$OUT" "doc-history-panel-desktop-v5" "${BASE},doc-history-panel,txt-apply-h2" "examples/demo-pack/sample.md" || true

for f in doc-wysiwyg-blockquote-desktop-v5.png doc-split-apply-banner-desktop-v5.png doc-opaque-card-desktop-v5.png doc-visual-docx-desktop-v6.png doc-history-panel-desktop-v5.png; do
  cp -f "$OUT/$f" "$ART/$f"
  cp -f "$OUT/${f%.png}.cm.json" "$ART/${f%.png}.cm.json" 2>/dev/null || true
done

python3 "$ROOT/scripts/check-doc-features-capture-pixels.py" "$OUT" "$ART" || true

echo "Desktop doc feature captures in $ART"
