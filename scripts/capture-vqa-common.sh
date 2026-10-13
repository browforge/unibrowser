#!/usr/bin/env bash
# Shared helpers for docs/screenshots/vqa/ evidence captures.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"
# shellcheck source=capture-isolated-config.sh
source "$ROOT/scripts/capture-isolated-config.sh"
BIN="$ROOT/target/release/unibrowser-desktop"

finalise_cm_capture() {
  local outdir="$1"
  local name="$2"
  local tmp_json="$outdir/.${name}.cm.json"
  local final_json="$outdir/${name}.cm.json"
  if [[ ! -f "$tmp_json" ]]; then
    echo "missing CM diagnostics: $tmp_json" >&2
    return 1
  fi
  mv "$tmp_json" "$final_json"
  python3 "$ROOT/scripts/check-cm-pixels.py" "$outdir/${name}.png" "$final_json"
  if python3 -c "import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d.get('blockPicker') else 1)" "$final_json"; then
    python3 "$ROOT/scripts/check-block-picker-open-pixels.py" "$outdir/${name}.png" "$final_json"
  fi
}

capture_vqa() {
  local outdir="$1"
  local name="$2"
  local demo="$3"
  local open_path="$4"
  local geometry="${5:-1280x800}"
  if [[ -n "${UNIBROWSER_REPO_ROOT:-}" && -n "$open_path" && "$open_path" != /* && "$open_path" != *://* ]]; then
    open_path="${UNIBROWSER_REPO_ROOT%/}/$open_path"
  fi
  mkdir -p "$outdir"
  local tmp="$outdir/.${name}.png"
  rm -f "$tmp"
  rm -f "$outdir/.${name}.cm.json"
  echo "=== capture ${name} (${geometry}) ==="
  capture_config_prepare
  # Webview doc-capture clears CM tooltips and focus before writing the PNG.
  if ! xvfb-run -a -s "-screen 0 ${geometry}x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_DOC_CAPTURE_SIZE="${geometry}" \
    UNIBROWSER_DEMO_STATE="$demo" \
    "$BIN" --open "$open_path" --doc-capture "$tmp"; then
    capture_config_cleanup
    return 1
  fi
  capture_config_cleanup
  mv "$tmp" "$outdir/${name}.png"
  finalise_cm_capture "$outdir" "$name"
  echo "wrote $outdir/${name}.png"
}
