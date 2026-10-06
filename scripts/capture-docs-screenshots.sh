#!/usr/bin/env bash
# Capture docs/screenshots under xvfb (each output <400KB).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/docs/screenshots"
BIN="$ROOT/target/release/unibrowser-desktop"
mkdir -p "$OUT"

if [[ ! -x "$BIN" ]]; then
  echo "Build release desktop first: cargo build -p unibrowser-desktop --release" >&2
  exit 1
fi

if ! command -v convert >/dev/null 2>&1; then
  echo "ImageMagick convert required" >&2
  exit 1
fi

capture() {
  local name="$1"
  local tmp="/tmp/unibrowser-cap-$$.png"
  xvfb-run -a -s "-screen 0 1280x800x24" bash -c "
    set -euo pipefail
    \"$BIN\" &
    APP_PID=\$!
    sleep 8
    if command -v import >/dev/null 2>&1; then
      import -window root \"$tmp\"
    elif command -v scrot >/dev/null 2>&1; then
      scrot \"$tmp\"
    else
      echo 'install imagemagick or scrot' >&2
      exit 1
    fi
    kill \"\$APP_PID\" 2>/dev/null || true
    wait \"\$APP_PID\" 2>/dev/null || true
  "
  convert "$tmp" -resize 1200x -strip -quality 82 "$OUT/$name"
  rm -f "$tmp"
  local kb
  kb=$(du -k "$OUT/$name" | awk '{print $1}')
  echo "$OUT/$name (${kb}KB)"
  if [[ "$kb" -gt 400 ]]; then
    convert "$OUT/$name" -quality 75 "$OUT/$name"
    kb=$(du -k "$OUT/$name" | awk '{print $1}')
    echo "  recompressed to ${kb}KB"
  fi
}

capture "shell-desktop.png"
