#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
OUT_DIR="$ROOT/docs/screenshots/ui-batch-3c"
MOCK_DIR="$ROOT/docs/design/mockups/quiet-utility"
BIN="$ROOT/target/release/unibrowser-desktop"
FIX="$ROOT/e2e/fixtures"
PDF_FIX="$ROOT/examples/demo-pack/sample.pdf"
mkdir -p "$OUT_DIR"

DOCUMENT_DEMO="theme-light,cm-text-pin,title-breadcrumb,document-mockup-problems,ai,ai-mockup-composer,cm-compact,rail-collapsed,capture-ready"
DOCUMENT_DEMO_DARK="theme-dark,cm-text-pin,title-breadcrumb,document-mockup-problems,ai,ai-mockup-composer,cm-compact,rail-collapsed,capture-ready"

capture() {
  local name="$1"
  local demo="$2"
  local open1="${3:-}"
  local tmp="/tmp/ub-batch3c-$$-${name}.png"
  local out="$OUT_DIR/${name}.png"
  local -a args=(--doc-capture "$tmp")
  if [[ -n "$open1" ]]; then
    args+=(--open "$open1")
  fi
  echo "Capturing $out demo=$demo"
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE=1 \
    UNIBROWSER_DEMO_STATE="$demo" \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    timeout 180 "$BIN" "${args[@]}"
  convert "$tmp" -resize 1280x800! -strip "$out"
  rm -f "$tmp"
  local bytes
  bytes=$(wc -c <"$out" | tr -d " ")
  if [[ "$bytes" -lt 8000 ]]; then
    echo "FAIL: $out only ${bytes} bytes" >&2
    exit 1
  fi
}

side_by_side() {
  local app_png="$1"
  local mock_png="$2"
  local out_png="$3"
  if [[ ! -f "$mock_png" ]]; then
    echo "SKIP side-by-side (missing mockup): $mock_png" >&2
    return 0
  fi
  convert "$app_png" -resize 1280x800! \
    \( "$mock_png" -resize 1280x800! \) \
    +append -strip "$out_png"
}

capture "ui-document-mockup-light" "$DOCUMENT_DEMO" "$FIX/sync-notes.ts"
capture "ui-document-mockup-dark" "$DOCUMENT_DEMO_DARK" "$FIX/sync-notes.ts"

side_by_side \
  "$OUT_DIR/ui-document-mockup-light.png" \
  "$MOCK_DIR/document-problems-ai.png" \
  "$OUT_DIR/ui-document-mockup-side-by-side-light.png"

side_by_side \
  "$OUT_DIR/ui-document-mockup-dark.png" \
  "$MOCK_DIR/document-problems-ai-dark.png" \
  "$OUT_DIR/ui-document-mockup-side-by-side-dark.png"

echo "Batch 3c screenshots in $OUT_DIR"
