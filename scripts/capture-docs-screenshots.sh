#!/usr/bin/env bash
# Capture docs/screenshots under xvfb (each output <400KB). Uses --open, UNIBROWSER_DEMO_STATE, --doc-capture.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/docs/screenshots"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
BIN="$ROOT/target/release/unibrowser-desktop"
DEMO="$ROOT/examples/demo-pack"
FIX="$ROOT/e2e/fixtures"
mkdir -p "$OUT"

if ! command -v convert >/dev/null 2>&1; then
  echo "ImageMagick convert required" >&2
  exit 1
fi

compress_png() {
  local file="$1"
  local kb
  kb=$(du -k "$file" | awk '{print $1}')
  if [[ "$kb" -gt 400 ]]; then
    convert "$file" -strip -quality 78 "$file"
    kb=$(du -k "$file" | awk '{print $1}')
    if [[ "$kb" -gt 400 ]]; then
      convert "$file" -resize 1100x -strip -quality 72 "$file"
    fi
  fi
  kb=$(du -k "$file" | awk '{print $1}')
  echo "  ${file} (${kb}KB)"
}

capture_scene() {
  local name="$1"
  local demo_state="${2:-}"
  local open_path="${3:-}"
  local tmp="/tmp/unibrowser-cap-$$-${name}.png"
  local out="$OUT/${name}.png"

  local -a args=(--doc-capture "$tmp")
  if [[ -n "$open_path" ]]; then
    args+=(--open "$open_path")
  fi

  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE=1 \
    UNIBROWSER_DEMO_STATE="$demo_state" \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    timeout 120 "$BIN" "${args[@]}"

  convert "$tmp" -resize 1200x -strip -quality 82 "$out"
  rm -f "$tmp"
  local bytes
  bytes=$(wc -c <"$out" | tr -d " ")
  if [[ "$bytes" -lt 8000 ]]; then
    echo "FAIL: ${out} is only ${bytes} bytes (likely blank shell)" >&2
    exit 1
  fi
  compress_png "$out"
}

echo "Capturing docs screenshots to $OUT"

capture_scene "unibrowser-01-ntp" "ntp"
capture_scene "unibrowser-02-files" "" "$DEMO"
capture_scene "unibrowser-03-palette" "palette"
capture_scene "unibrowser-04-settings" "settings"
capture_scene "unibrowser-05-unibrow" "" "$DEMO/sample.unibrow"
capture_scene "unibrowser-06-viewer" "" "$FIX/minimal.docx"
capture_scene "unibrowser-07-problems" "problems" "$FIX/spell-test.txt"
capture_scene "unibrowser-08-ai-sidebar" "ai"
capture_scene "unibrowser-09-live-preview" "live" "$DEMO/sample.unibrow"
capture_scene "unibrowser-10-vsix-mods" "vsix"

# Prefer a shot that shows product UI, not an empty content webview on xvfb.
if [[ -f "$OUT/unibrowser-08-ai-sidebar.png" ]]; then
  cp -f "$OUT/unibrowser-08-ai-sidebar.png" "$OUT/shell-desktop.png"
else
  cp -f "$OUT/unibrowser-03-palette.png" "$OUT/shell-desktop.png"
fi
compress_png "$OUT/shell-desktop.png"

echo "Checking screenshot diversity (md5)…"
mapfile -t HASHES < <(md5sum "$OUT"/*.png | awk '{print $1}' | sort -u)
if [[ "${#HASHES[@]}" -lt 8 ]]; then
  echo "WARN: only ${#HASHES[@]} unique PNG hashes in $OUT (captures may be too similar)" >&2
fi

echo "Done."
