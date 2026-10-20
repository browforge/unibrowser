#!/usr/bin/env bash
# PKG-03..06 UI evidence at 1280x800 (web WASM captures; desktop binary built for parity).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
OUT="$ROOT/docs/screenshots/pkg-v1"
ART="/opt/cursor/artifacts"
mkdir -p "$OUT" "$ART"
rm -f "$OUT"/*-v1-1280.png "$OUT"/*-v1-1280.cm.json

(
  cd "$ROOT/crates/unibrowser-desktop"
  npm run build:web
  PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH:-/usr/bin/google-chrome-stable}" \
    npx playwright test tests/pkg-v1-capture.spec.ts --config playwright.config.ts
)

python3 "$ROOT/scripts/check-pkg-v1-capture-pixels.py"
