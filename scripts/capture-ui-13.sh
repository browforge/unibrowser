#!/usr/bin/env bash
# UI-13 file browser and settings layout evidence.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
OUT="$ROOT/docs/screenshots/ui-13"
mkdir -p "$OUT"
BIN="$ROOT/target/release/unibrowser-desktop"
PACK="$ROOT/examples/demo-pack"

capture() {
  local name="$1"
  local demo="$2"
  local open_path="$3"
  local dimensions="$4"
  local tmp="$OUT/.${name}.png"
  rm -f "$tmp"
  xvfb-run -a -s "-screen 0 ${dimensions}x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DEMO_STATE="$demo" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"
  mv "$tmp" "$OUT/${name}.png"
  local actual
  actual="$(identify -format '%wx%h' "$OUT/${name}.png")"
  if [[ "$actual" != "$dimensions" ]]; then
    echo "capture dimensions differ: ${name} is ${actual}, expected ${dimensions}" >&2
    exit 1
  fi
}

base="capture-ready,cm-compact,rail-collapsed"
capture "files-browser-preview-light-100" \
  "${base},theme-light,standard-contrast,scale-100,ui13-files-preview" \
  "$PACK" "1600x1000"
capture "files-browser-preview-dark-100" \
  "${base},theme-dark,standard-contrast,scale-100,ui13-files-preview" \
  "$PACK" "1600x1000"
capture "files-browser-preview-hc-100" \
  "${base},theme-light,high-contrast,scale-100,ui13-files-preview" \
  "$PACK" "1600x1000"
capture "files-browser-preview-light-150" \
  "${base},theme-light,standard-contrast,scale-150,ui13-files-preview" \
  "$PACK" "1600x1000"

capture "settings-developer-collapsed-light-800" \
  "${base},theme-light,standard-contrast,scale-100,settings,ui13-settings-developer-collapsed" \
  "unibrowser://settings" "800x800"
capture "settings-developer-expanded-light-800" \
  "${base},theme-light,standard-contrast,scale-100,settings,ui13-settings-developer-expanded" \
  "unibrowser://settings" "800x800"
capture "settings-developer-expanded-light-1600" \
  "${base},theme-light,standard-contrast,scale-100,settings,ui13-settings-developer-expanded" \
  "unibrowser://settings" "1600x1000"
capture "settings-developer-expanded-dark-1600" \
  "${base},theme-dark,standard-contrast,scale-100,settings,ui13-settings-developer-expanded" \
  "unibrowser://settings" "1600x1000"
capture "settings-developer-expanded-hc-1600" \
  "${base},theme-light,high-contrast,scale-100,settings,ui13-settings-developer-expanded" \
  "unibrowser://settings" "1600x1000"
capture "settings-developer-expanded-light-150" \
  "${base},theme-light,standard-contrast,scale-150,settings,ui13-settings-developer-expanded" \
  "unibrowser://settings" "1600x1000"

declare -A seen_hashes=()
for png in "$OUT"/*.png; do
  hash="$(sha256sum "$png" | cut -d' ' -f1)"
  name="$(basename "$png")"
  if [[ -n "${seen_hashes[$hash]:-}" ]]; then
    echo "duplicate capture hash: ${seen_hashes[$hash]} and ${name} (${hash})" >&2
    exit 1
  fi
  seen_hashes[$hash]="$name"
done

echo "UI-13 captures in $OUT"
