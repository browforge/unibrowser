#!/usr/bin/env bash
# Wednesday report screenshot helper (manual scenes + scrot/import).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${UB_REPORT_SCREENSHOT_DIR:-$ROOT/report-screenshots}"
mkdir -p "$OUT"

if [[ -z "${DISPLAY:-}" ]]; then
  echo "No DISPLAY set. Start Unibrowser locally, then run this script in a graphical session."
  echo "See docs/report-visuals.md for the shot list."
  exit 1
fi

capture() {
  local name="$1"
  local path="$OUT/$name"
  if command -v scrot >/dev/null 2>&1; then
    scrot "$path"
  elif command -v import >/dev/null 2>&1; then
    import -window root "$path"
  else
    echo "Install scrot or imagemagick (import) to capture screenshots."
    exit 1
  fi
  echo "Saved $path"
}

cat <<'GUIDE'
Unibrowser report screenshots
==============================
Launch: cd crates/unibrowser-desktop && npm run tauri dev

For each prompt, arrange the window and press Enter.
GUIDE

shots=(
  "unibrowser-01-ntp.png:New tab — NTP hero and recent"
  "unibrowser-02-files.png:Folder tab — breadcrumbs and file list"
  "unibrowser-03-palette.png:Ctrl+Shift+P command palette"
  "unibrowser-04-settings.png:unibrowser://settings"
  "unibrowser-05-unibrow.png:Notes .unibrow board + timeline"
  "unibrowser-06-viewer.png:EPUB/DOCX/PDF viewer highlight"
)

for entry in "${shots[@]}"; do
  file="${entry%%:*}"
  hint="${entry#*:}"
  read -r -p "Ready for $hint? [Enter] " _
  capture "$file"
done

echo "Done. Files in $OUT (add to .gitignore; do not commit large binaries)."
