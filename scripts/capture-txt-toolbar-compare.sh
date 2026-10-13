#!/usr/bin/env bash
# Side-by-side Quiet Utility mockup | build for text chrome at 1280 px width.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MOCK_DIR="$ROOT/docs/design/mockups/quiet-utility"
OUT="${ROOT}/docs/screenshots/txt-toolbar"
ART="${ART:-/opt/cursor/artifacts}"
CHROME_H="${TXT_CHROME_CROP_H:-112}"
CHROME="${CHROME_BIN:-$(command -v chromium || command -v chromium-browser || command -v google-chrome-stable || command -v google-chrome)}"

mkdir -p "$OUT" "$ART"

render_mockups() {
  if [[ -x "$MOCK_DIR/render.sh" ]]; then
    (cd "$MOCK_DIR" && CHROME_BIN="$CHROME" ./render.sh) || true
  fi
}

require_mock() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "Rendering Quiet Utility text chrome mockups…" >&2
    render_mockups
  fi
  if [[ ! -f "$path" ]]; then
    echo "missing approved mockup: $path" >&2
    exit 1
  fi
}

compare_pair() {
  local app_png="$1"
  local mock_png="$2"
  local out_png="$3"
  if [[ ! -f "$app_png" ]]; then
    echo "missing app capture: $app_png" >&2
    exit 1
  fi
  require_mock "$mock_png"
  local app_crop="/tmp/ub-txt-chrome-app-$$.png"
  local mock_crop="/tmp/ub-txt-chrome-mock-$$.png"
  convert "$app_png" -crop "1280x${CHROME_H}+0+0" +repage -strip "$app_crop"
  convert "$mock_png" -resize "1280x${CHROME_H}!" -strip "$mock_crop"
  convert "$mock_crop" "$app_crop" +append -strip "$out_png"
  rm -f "$app_crop" "$mock_crop"
  local bytes
  bytes=$(wc -c <"$out_png" | tr -d " ")
  if [[ "$bytes" -lt 4000 ]]; then
    echo "FAIL: $out_png only ${bytes} bytes" >&2
    exit 1
  fi
  echo "Wrote $out_png (${bytes} bytes)"
}

compare_pair \
  "$OUT/txt-desktop-markdown.png" \
  "$MOCK_DIR/text-toolbar-compact-chrome-markdown.png" \
  "$OUT/compare-at-1280-markdown.png"

compare_pair \
  "$OUT/txt-docx-emulated-h1.png" \
  "$MOCK_DIR/text-toolbar-compact-chrome-docx.png" \
  "$OUT/compare-at-1280-docx.png"

cp -f "$OUT/compare-at-1280-markdown.png" "$OUT/compare-at-1280-docx.png" "$ART"/
echo "Copied compare-at-1280 PNGs to $ART"
