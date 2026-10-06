#!/usr/bin/env bash
# Capture docs/screenshots under xvfb (each output <400KB). Uses CLI --open and UNIBROWSER_DEMO_STATE.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/docs/screenshots"
BIN="$ROOT/target/release/unibrowser-desktop"
DEMO="$ROOT/examples/demo-pack"
FIX="$ROOT/e2e/fixtures"
mkdir -p "$OUT"

if [[ ! -x "$BIN" ]]; then
  echo "Build release desktop first: cargo build -p unibrowser-desktop --release" >&2
  exit 1
fi

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

  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    UNIBROWSER_DEMO_STATE="$demo_state" \
    UB_CAPTURE_OPEN="$open_path" \
    UB_BIN="$BIN" \
    UB_TMP="$tmp" \
    bash -c '
    set -euo pipefail
    args=()
    if [[ -n "${UB_CAPTURE_OPEN:-}" ]]; then
      args+=(--open "$UB_CAPTURE_OPEN")
    fi
    "$UB_BIN" "${args[@]}" &
    APP_PID=$!
    sleep 12
    if command -v import >/dev/null 2>&1; then
      import -window root "$UB_TMP"
    elif command -v xwd >/dev/null 2>&1; then
      xwd -root -out /tmp/unibrowser-cap.xwd
      convert /tmp/unibrowser-cap.xwd "$UB_TMP"
      rm -f /tmp/unibrowser-cap.xwd
    elif command -v scrot >/dev/null 2>&1; then
      scrot "$UB_TMP"
    else
      echo "install imagemagick (import) or scrot" >&2
      exit 1
    fi
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  '
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
capture_scene "unibrowser-07-problems" "problems" "$DEMO/sample.txt"
capture_scene "unibrowser-08-ai-sidebar" "ai"
capture_scene "unibrowser-09-live-preview" "live" "$DEMO/sample.unibrow"
capture_scene "unibrowser-10-vsix-mods" "vsix"

cp -f "$OUT/unibrowser-01-ntp.png" "$OUT/shell-desktop.png"
compress_png "$OUT/shell-desktop.png"

echo "Checking screenshot diversity (md5)…"
mapfile -t HASHES < <(md5sum "$OUT"/*.png | awk '{print $1}' | sort -u)
if [[ "${#HASHES[@]}" -lt 3 ]]; then
  echo "WARN: only ${#HASHES[@]} unique PNG hashes in $OUT (captures may be identical frames)" >&2
fi

echo "Done."
