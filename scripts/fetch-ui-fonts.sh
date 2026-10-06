#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$ROOT/crates/unibrowser-desktop/public/fonts"
mkdir -p "$FONT_DIR"
cd "$ROOT/crates/unibrowser-desktop"
npm install --no-save @fontsource/source-sans-3@5.2.5 @fontsource/source-code-pro@5.2.5
cp node_modules/@fontsource/source-sans-3/files/source-sans-3-latin-{400,600,700}-normal.woff2 "$FONT_DIR/"
cp node_modules/@fontsource/source-code-pro/files/source-code-pro-latin-{400,600}-normal.woff2 "$FONT_DIR/"
echo "Fonts copied to $FONT_DIR"
