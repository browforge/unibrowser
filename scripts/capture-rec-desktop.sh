#!/usr/bin/env bash
# REC-01 desktop evidence (recDocCapture fast path; no CLI --open).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
npm run build --prefix "$ROOT/crates/unibrowser-desktop"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="$ROOT/docs/screenshots/rec"
ART="/opt/cursor/artifacts"
mkdir -p "$OUT" "$ART"
rm -f "$OUT"/rec-*.png "$OUT"/rec-*.cm.json "$OUT"/.rec-*.png "$OUT"/.rec-*.cm.json

BASE="capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100,cm-collapsed"

FIXTURE="e2e/fixtures/spell-test.txt"

capture_rec() {
  local name="$1"
  local demo="$2"
  local tmp
  tmp=$(mktemp -p "$OUT" "${name}.XXXXXX.png")
  echo "=== capture ${name} (1280x800) ==="
  capture_config_prepare
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="1280x800" \
    UNIBROWSER_DEMO_STATE="${demo}" \
    "$BIN" --open "$FIXTURE" --doc-capture "$tmp"
  capture_config_cleanup
  local cm_sidecar="${tmp%.png}.cm.json"
  mv "$tmp" "$OUT/${name}.png"
  if [[ -f "$cm_sidecar" ]]; then
    mv "$cm_sidecar" "$OUT/${name}.cm.json"
  elif [[ ! -f "$OUT/${name}.cm.json" ]]; then
    echo "missing CM diagnostics: $cm_sidecar" >&2
    exit 1
  fi
  python3 "$ROOT/scripts/check-cm-pixels.py" "$OUT/${name}.png" "$OUT/${name}.cm.json"
}

capture_rec "rec-recovery-banner" "${BASE},rec-capture-banner"
capture_rec "rec-restored-document" "${BASE},rec-capture-restored"
# pkg fast path handles rec-capture-* tokens (see pkgDocCapture.ts).
cp "$OUT/rec-recovery-banner.png" "$ART/rec-recovery-banner.png" || true
cp "$OUT/rec-restored-document.png" "$ART/rec-restored-document.png" || true
python3 "${ROOT}/scripts/check-rec-capture-pixels.py"
