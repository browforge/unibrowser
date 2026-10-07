#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"
OUT_DIR="$ROOT/docs/screenshots/ui-batch-3"
BIN="$ROOT/target/release/unibrowser-desktop"
FIX="$ROOT/e2e/fixtures"
MOCK="$ROOT/docs/design/mockups/quiet-utility/document-problems-ai.png"
mkdir -p "$OUT_DIR"

capture() {
  local name="$1"
  local demo="$2"
  local open_path="${3:-}"
  local tmp="/tmp/ub-batch3-$$-${name}.png"
  local out="$OUT_DIR/${name}.png"
  local -a args=(--doc-capture "$tmp")
  if [[ -n "$open_path" ]]; then
    args+=(--open "$open_path")
  fi
  echo "Capturing $out demo=$demo open=${open_path:-none}"
  capture_config_prepare
  xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DEMO_STATE="$demo" \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    timeout 120 "$BIN" "${args[@]}"
  capture_config_cleanup
  convert "$tmp" -resize 1280x800 -strip "$out"
  rm -f "$tmp"
  local bytes
  bytes=$(wc -c <"$out" | tr -d " ")
  if [[ "$bytes" -lt 8000 ]]; then
    echo "FAIL: $out only ${bytes} bytes" >&2
    exit 1
  fi
}

# UI-04 / UI-07: title bar + tabs + omnibox (text document, compact CM)
capture "ui-04-07-document-compact" "theme-light,cm-compact,rail-collapsed,capture-ready" "$FIX/spell-test.txt"

# UI-05: grouped command manager (problems for visual parity with mockup)
capture "ui-05-command-manager" "theme-light,problems,cm-compact,rail-collapsed,capture-ready" "$FIX/spell-test.txt"

# Activity rail expanded
capture "ui-rail-expanded" "theme-light,rail-expanded,cm-compact,capture-ready" "$FIX/spell-test.txt"

# Chrome collapsed (title bar only)
capture "ui-chrome-collapsed" "theme-light,cm-compact,cm-collapsed,capture-ready" "$FIX/spell-test.txt"

# 150% text scaling (approximate Windows display scaling)
capture "ui-chrome-scale-150" "theme-light,cm-compact,scale-150,capture-ready" "$FIX/spell-test.txt"

if [[ -f "$MOCK" ]]; then
  convert "$OUT_DIR/ui-04-07-document-compact.png" \
    \( "$MOCK" -resize 1280x800 \) \
    +append -strip "$OUT_DIR/ui-document-mockup-side-by-side.png"
fi

echo "Batch 3 screenshots in $OUT_DIR"
