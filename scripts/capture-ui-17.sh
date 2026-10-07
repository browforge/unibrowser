#!/usr/bin/env bash
# UI-17 unknown-handler and Mod Portal evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
OUT="$ROOT/docs/screenshots/ui-17"
mkdir -p "$OUT"
rm -f "$OUT"/*.png
BIN="$ROOT/target/release/unibrowser-desktop"
FIXTURE="$ROOT/examples/demo-pack/unknown.ubunknown"

capture() {
  local name="$1"
  local extra="$2"
  local tmp="$OUT/.${name}.png"
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="capture-ready,cm-compact,rail-collapsed,theme-light,standard-contrast,scale-100${extra:+,$extra}" \
    "$BIN" --open "$FIXTURE" --doc-capture "$tmp"
  mv "$tmp" "$OUT/${name}.png"
  [[ "$(identify -format '%wx%h' "$OUT/${name}.png")" == "1280x800" ]]
}

capture "unknown-file-support-light-1280" ""
capture "mod-portal-discover-light-1280" "ui17-mod-portal"

if [[ "$(sha256sum "$OUT"/*.png | cut -d' ' -f1 | sort | uniq -d | wc -l)" != "0" ]]; then
  echo "duplicate UI-17 captures" >&2
  exit 1
fi

echo "UI-17 captures in $OUT"
