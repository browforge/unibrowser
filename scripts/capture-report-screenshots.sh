#!/usr/bin/env bash
# Wednesday report screenshot helper (manual scenes + scrot/import).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${UB_REPORT_SCREENSHOT_DIR:-$ROOT/report-screenshots}"

print_guide() {
  cat <<GUIDE
Unibrowser report screenshots
==============================
Docs: docs/report-visuals.md

Prerequisites
  - cargo test --workspace && cd crates/unibrowser-desktop && npm run build
  - Graphical session with DISPLAY (for interactive capture only)

Launch the app
  cd crates/unibrowser-desktop && npm run tauri dev

Shot list (save as 1920x1080 or native, consistent theme)
  1. unibrowser-01-ntp.png      — New tab (NTP hero + recent)
  2. unibrowser-02-files.png    — Folder tab (breadcrumbs + list)
  3. unibrowser-03-palette.png  — Ctrl+Shift+P palette
  4. unibrowser-04-settings.png — unibrowser://settings
  5. unibrowser-05-unibrow.png  — Notes .unibrow board + timeline
  6. unibrowser-06-viewer.png   — EPUB, DOCX, or PDF highlight

Output directory (gitignored): ${OUT}

Tools (Linux): sudo apt-get install -y scrot   # or imagemagick (import)

Non-interactive: pass --guide-only to print this list and exit (no capture).
GUIDE
}

if [[ "${1:-}" == "--guide-only" ]]; then
  print_guide
  exit 0
fi

mkdir -p "$OUT"

if [[ -z "${DISPLAY:-}" ]]; then
  echo "No DISPLAY set. Use --guide-only for instructions, or run in a graphical session."
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

print_guide
echo ""
echo "Interactive capture — press Enter between shots."
echo ""

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

echo "Done. Files in $OUT (do not commit large binaries)."
