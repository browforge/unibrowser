#!/usr/bin/env bash
# CMD-11 worked-context Command Manager evidence (Monaco + CM visible).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
OUT="$ROOT/docs/screenshots/cmd-11"
mkdir -p "$OUT"
BIN="$ROOT/target/release/unibrowser-desktop"
PACK="$ROOT/examples/demo-pack"

capture() {
  local name="$1"
  local demo="$2"
  local open_path="$3"
  local tmp="$OUT/.${name}.png"
  rm -f "$tmp"
  echo "=== ${name} ==="
  xvfb-run -a -s "-screen 0 1600x1000x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DEMO_STATE="$demo" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"
  mv "$tmp" "$OUT/${name}.png"
  echo "wrote $OUT/${name}.png"
}

base="capture-ready"
declare -a CONTEXTS=(
  "plain-text|${PACK}/sample.txt"
  "markdown|${PACK}/sample.md"
  "rich-document|${PACK}/sample.docx"
  "code|${PACK}/sample.ts"
  "pdf-annotation|${PACK}/sample.pdf"
  "image|${PACK}/sample.svg"
)

for row in "${CONTEXTS[@]}"; do
  slug="${row%%|*}"
  path="${row#*|}"
  capture "${slug}-light-100" "${base},theme-light" "$path"
  capture "${slug}-dark-100" "${base},theme-dark" "$path"
  capture "${slug}-hc-100" "${base},theme-light,high-contrast" "$path"
  capture "${slug}-light-150" "${base},theme-light,scale-150" "$path"
done

echo "CMD-11 captures in $OUT"
