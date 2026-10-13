#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
CHROME="${CHROME_BIN:-$(command -v chromium || command -v chromium-browser || command -v google-chrome-stable || command -v google-chrome)}"

render() {
  local source="$1"
  local output="$2"
  local size="$3"
  local profile
  profile="$(mktemp -d)"
  timeout 30 "$CHROME" \
    --headless \
    --disable-gpu \
    --disable-dev-shm-usage \
    --no-sandbox \
    --hide-scrollbars \
    --force-device-scale-factor=1 \
    --user-data-dir="$profile" \
    --window-size="$size" \
    --screenshot="$ROOT/$output" \
    "file://$ROOT/$source"
  if [[ -d "$profile" && ! -L "$profile" ]]; then
    rm -rf "$profile"
  fi
}

render "document-problems-ai.html" "document-problems-ai.png" "1280,800"
render "document-problems-ai.html?theme=dark" "document-problems-ai-dark.png" "1280,800"
render "pdf-annotation.html" "pdf-annotation.png" "1280,800"
render "unibrow-create.html" "unibrow-create.png" "1280,800"
render "new-tab.html" "new-tab.png" "1280,800"
render "multimodal-workshop.html" "multimodal-workshop.png" "1280,800"
render "multimodal-workshop.html?wide=1" "multimodal-workshop-wide.png" "1920,1080"
render "chrome-before-after.html" "chrome-before-after.png" "1280,800"

# Command Manager density studies.
render "document-problems-ai.html?density=compact" "document-command-compact.png" "1280,800"
render "document-problems-ai.html?density=large" "document-command-large.png" "1280,800"
render "pdf-annotation.html?density=compact" "pdf-command-compact.png" "1280,800"
render "pdf-annotation.html?density=large" "pdf-command-large.png" "1280,800"
render "multimodal-workshop.html?wide=1&density=compact" "cad-command-compact-wide.png" "1920,1080"
render "multimodal-workshop.html?wide=1&density=large" "cad-command-large-wide.png" "1920,1080"
render "command-system-gallery.html" "command-system-gallery-light.png" "1920,1080"
render "command-system-gallery.html?theme=dark" "command-system-gallery-dark.png" "1920,1080"

# Text compact toolbar chrome (title + 32 px row + CM) for compare-at-1280 captures.
render "text-toolbar-compact-chrome.html" "text-toolbar-compact-chrome-markdown.png" "1280,112"
render "text-toolbar-compact-chrome.html?docx=1" "text-toolbar-compact-chrome-docx.png" "1280,112"

echo "Rendered Quiet Utility mockups in $ROOT"
