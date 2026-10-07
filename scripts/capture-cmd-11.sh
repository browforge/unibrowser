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
  local dimensions width height
  dimensions="$(identify -format '%w %h' "$OUT/${name}.png")"
  read -r width height <<<"$dimensions"
  if (( width < 1200 || height < 700 )); then
    echo "capture too small: ${name} is ${width}x${height}" >&2
    exit 1
  fi
  echo "wrote $OUT/${name}.png"
}

base="capture-ready,cm-compact"
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
  proof=""
  if [[ "$slug" == "plain-text" ]]; then
    proof=",cmd11-plain-proof"
  elif [[ "$slug" == "markdown" ]]; then
    proof=",cmd11-markdown-proof"
  fi
  capture "${slug}-light-100" "${base},theme-light,standard-contrast,scale-100${proof}" "$path"
  capture "${slug}-dark-100" "${base},theme-dark,standard-contrast,scale-100${proof}" "$path"
  capture "${slug}-hc-100" "${base},theme-light,high-contrast,scale-100${proof}" "$path"
  capture "${slug}-light-150" "${base},theme-light,standard-contrast,scale-150${proof}" "$path"
done

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

echo "CMD-11 captures in $OUT"
