#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"
OUT_DIR="$ROOT/docs/screenshots/ui-batch-2"
BIN="$ROOT/target/release/unibrowser-desktop"
FIX="$ROOT/e2e/fixtures"
mkdir -p "$OUT_DIR"

capture() {
  local name="$1"
  local demo="$2"
  local open_path="${3:-}"
  local tmp="/tmp/ub-batch2-$$-${name}.png"
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

# UI-06A: tokens / type / docked AI + status (chrome frame unchanged)
capture "ui-06a-light" "theme-light,ai" "$FIX/spell-test.txt"
capture "ui-06a-dark" "theme-dark,ai" "$FIX/spell-test.txt"

# UI-08 palette
capture "ui-08-palette" "theme-light,palette" "$FIX/minimal.docx"

# UI-09 dock + problems
capture "ui-09-dock-problems" "theme-light,problems,ai" "$FIX/spell-test.txt"

# UI-10 problems table
capture "ui-10-problems" "theme-light,problems" "$FIX/spell-test.txt"

# UI-11 feedback ladder
capture "ui-11-feedback" "theme-light,feedback-demo" "$FIX/spell-test.txt"

rm -f "$OUT_DIR/ui-07-tabs.png"
echo "Batch 2 screenshots in $OUT_DIR (UI-07 deferred)"
