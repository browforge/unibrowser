#!/usr/bin/env bash
# Copy self-test PNGs into docs/screenshots when a full run produced them.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/target/self-test-reports/self-test-shots"
OUT="$ROOT/docs/screenshots"
if [[ ! -d "$SRC" ]]; then
  echo "Run ./scripts/self-test.sh first" >&2
  exit 1
fi
mkdir -p "$OUT"
map_shot() {
  local id="$1"
  local dest="$2"
  if [[ -f "$SRC/${id}.png" ]]; then
    cp -f "$SRC/${id}.png" "$OUT/${dest}"
    echo "  ${id} -> ${dest}"
  fi
}
map_shot "palette-command" "unibrowser-03-palette.png"
map_shot "unknown-file-banner" "unibrowser-02-files.png"
map_shot "font-text-svg" "unibrowser-06-viewer.png"
map_shot "cm-grey-tab-switch" "unibrowser-04-settings.png"
map_shot "close-undo" "unibrowser-01-ntp.png"
map_shot "unibrow-blank-save" "unibrowser-05-unibrow.png"
map_shot "proofing-dictionary" "unibrowser-07-problems.png"
map_shot "problems-jump" "unibrowser-07-problems.png"
echo "Synced self-test shots into $OUT"
