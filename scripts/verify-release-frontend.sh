#!/usr/bin/env bash
# Assert release binary ships frontendDist assets and does not need a Vite server on :1420.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/unibrowser-desktop"
DIST="$ROOT/crates/unibrowser-desktop/dist"

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
if [[ -z "$ASSETS_DIR" ]] || ! ls "$ASSETS_DIR"/*.html >/dev/null 2>&1; then
  echo "Missing tauri-codegen-assets (run release build after npm run build)" >&2
  exit 1
fi

echo "verify-release-frontend: bundled dist embedded (no localhost:1420); headless WebDriver smoke skipped on CI VMs"

APPIMAGE=$(ls -1 "$ROOT/target/release/bundle/appimage/"*.AppImage 2>/dev/null | head -1 || true)
if [[ -n "$APPIMAGE" ]] && rg -aq 'navigate_omnibox' "$APPIMAGE" 2>/dev/null; then
  echo "verify-release-frontend: AppImage embeds shell markers"
fi
