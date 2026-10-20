#!/usr/bin/env bash
# ADR 0036 / NEW-10: New project flow captures at 1280x800 (desktop + web).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
(cd "$ROOT/crates/unibrowser-desktop" && npm run build)
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/new-project"
ART="/opt/cursor/artifacts"
mkdir -p "$OUT" "$ART"
rm -f "$OUT"/empty-project-tab-v6-1280.png
rm -f "$OUT"/empty-project-tab-v5-1280.png
rm -f "$OUT"/*-v5-1280.png "$OUT"/*-v5-web-1280.png
rm -f "$OUT"/*-v4-1280.png "$OUT"/*-v4-web-1280.png
rm -f "$OUT"/*-v3-1280.png "$OUT"/*-v3-web-1280.png
rm -f "$OUT"/*-v2-1280.png "$OUT"/*-v2-web-1280.png
rm -f "$OUT"/*-1280.png "$OUT"/*-web-1280.png

capture_desktop() {
  local name="$1"
  local demo="$2"
  local open_path="${3:-}"
  local version="${4:-v5}"
  local tmp="$OUT/.${name}.png"
  local demo_full="capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100,cm-collapsed,${demo}"
  capture_config_prepare
  local open_args=()
  if [[ -n "$open_path" ]]; then
    open_args=(--open "$open_path")
  fi
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="$demo_full" \
    "$BIN" "${open_args[@]}" --doc-capture "$tmp"
  capture_config_cleanup
  mv "$tmp" "$OUT/${name}-${version}-1280.png"
  cp "$OUT/${name}-${version}-1280.png" "$ART/${name}-${version}-1280.png" || true
  [[ "$(identify -format '%wx%h' "$OUT/${name}-${version}-1280.png")" == "1280x800" ]]
}

capture_desktop "new-tab-paper-card" "new-project-capture-ntp"
capture_desktop "template-gallery-pick" "new-project-capture-gallery"
capture_desktop "template-gallery-pick-office" "new-project-capture-gallery-office"
capture_desktop "template-gallery-configure" "new-project-capture-configure"
capture_desktop "empty-project-tab" "new-project-capture-empty" "" "v6"
capture_desktop "office-template-project" "new-project-capture-office-project,text-card-render1"
capture_desktop "presentation-template-project" "new-project-capture-presentation-project"
capture_desktop "save-as-template" "new-project-capture-save-template"
capture_desktop "docx-tab-breadcrumb" "text-card-render1,title-breadcrumb,rail-collapsed,doc-visual-md-docx" \
  "$ROOT/examples/demo-pack/sample.md"
capture_desktop "relationship-consent" "new-project-capture-relationship-consent" "" "v4"
capture_desktop "first-save-dialog" "new-project-capture-first-save" "" "v3"

(
  cd "$ROOT/crates/unibrowser-desktop"
  npm run build:web
  PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH:-/usr/bin/google-chrome-stable}" \
    npx playwright test tests/new-project-capture.spec.ts --config playwright.config.ts
)

python3 "$ROOT/scripts/check-new-project-capture-pixels.py"
