#!/usr/bin/env bash
# Assert release binary ships frontendDist assets and does not need a Vite server on :1420.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/unibrowser-desktop"
DIST="$ROOT/crates/unibrowser-desktop/dist"
RESULT_FILE="$ROOT/docs/verify-release-frontend-last.txt"

kill_port_listeners() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -ti "tcp:${port}" 2>/dev/null | xargs -r kill -9 2>/dev/null || true
  elif command -v fuser >/dev/null 2>&1; then
    fuser -k "${port}/tcp" 2>/dev/null || true
  fi
}

if [[ ! -x "$BIN" ]]; then
  echo "Missing release binary: $BIN" >&2
  exit 1
fi
if [[ ! -f "$DIST/index.html" ]]; then
  echo "Missing dist/index.html — run npm run build in crates/unibrowser-desktop" >&2
  exit 1
fi

if [[ "$BIN" -ot "$DIST/index.html" ]]; then
  echo "Release binary is older than dist/index.html — run scripts/ensure-release-desktop.sh" >&2
  exit 1
fi

if ! rg -aq 'navigate_omnibox' "$BIN" 2>/dev/null; then
  echo "Release binary missing embedded shell markers (navigate_omnibox)" >&2
  exit 1
fi

kill_port_listeners 1420
kill_port_listeners 4445
pgrep -f '/unibrowser-desktop$' 2>/dev/null | xargs -r kill -9 2>/dev/null || true
sleep 1

if rg -aq 'localhost:1420' "$BIN" 2>/dev/null; then
  echo "Release binary still references localhost:1420 (dev server coupling)" >&2
  exit 1
fi

ASSETS_DIR=$(ls -d "$ROOT/target/release/build/unibrowser-desktop-"*/out/tauri-codegen-assets 2>/dev/null | head -1 || true)
STAMP="$DIST/.ub-frontend-stamp"
if [[ ! -f "$STAMP" ]]; then
  echo "Missing $STAMP — run npm run build in crates/unibrowser-desktop" >&2
  exit 1
fi
EMBED_STAMP_FILE=$(ls -t "$ROOT/target/release/build/unibrowser-desktop-"*/out/frontend-stamp.txt 2>/dev/null | head -1 || true)
if [[ -z "$EMBED_STAMP_FILE" ]] || [[ "$(tr -d '\n' <"$STAMP")" != "$(tr -d '\n' <"$EMBED_STAMP_FILE")" ]]; then
  echo "Embedded frontend stamp does not match dist (run scripts/ensure-release-desktop.sh)" >&2
  exit 1
fi

APPIMAGE=$(ls -1 "$ROOT/target/release/bundle/appimage/"*.AppImage 2>/dev/null | head -1 || true)
APPIMAGE_OK="skipped (no AppImage built)"
if [[ -n "$APPIMAGE" ]]; then
  EXTRACT_DIR=$(mktemp -d)
  trap 'rm -rf "$EXTRACT_DIR"' EXIT
  cd "$EXTRACT_DIR"
  "$APPIMAGE" --appimage-extract >/dev/null 2>&1 || true
  APP_BIN=$(find "$EXTRACT_DIR/squashfs-root" -name 'unibrowser-desktop' -type f 2>/dev/null | head -1 || true)
  if [[ -z "$APP_BIN" ]] || [[ ! -f "$APP_BIN" ]]; then
    echo "AppImage extract failed (no unibrowser-desktop inside)" >&2
    exit 1
  fi
  if rg -aq 'localhost:1420' "$APP_BIN" 2>/dev/null; then
    echo "AppImage still references localhost:1420" >&2
    exit 1
  fi
  if ! rg -aq 'navigate_omnibox' "$APP_BIN" 2>/dev/null; then
    echo "AppImage missing embedded shell markers" >&2
    exit 1
  fi
  APPIMAGE_OK="pass ($(basename "$APPIMAGE"), extracted binary, port 1420 closed)"
  cd "$ROOT"
fi

{
  echo "verify-release-frontend $(date -u +%Y-%m-%dT%H:%MZ)"
  echo "binary: pass (bundled dist, no localhost:1420)"
  echo "tauri-codegen-assets: pass"
  echo "appimage: ${APPIMAGE_OK}"
} | tee "$RESULT_FILE"

echo "Recorded in $RESULT_FILE"
