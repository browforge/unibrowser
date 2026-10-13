#!/usr/bin/env bash
# TXT-06 evidence: compact text toolbar at 1280x800 light.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=capture-vqa-common.sh
source "$ROOT/scripts/capture-vqa-common.sh"
OUT="${ROOT}/docs/screenshots/txt-toolbar"
ART="${ART:-/opt/cursor/artifacts}"
mkdir -p "$OUT" "$ART"

echo "=== capture txt-web-markdown-picker-h2 (web build, Playwright) ==="
(
  cd "$ROOT/crates/unibrowser-desktop"
  npm run build:web
  npx playwright test tests/txt-toolbar-web-capture.spec.ts --config playwright.config.ts
)

capture_vqa "$OUT" "txt-desktop-markdown" "theme-light" "examples/demo-pack/sample.md"
capture_vqa "$OUT" "txt-docx-emulated-h1" "theme-light,txt-apply-h1,txt-block-picker-open" "examples/demo-pack/sample-rich.docx"
capture_vqa "$OUT" "txt-sample-mod-toolbar" "theme-light,txt-mod-toolbar-visible" "examples/demo-pack/sample.md"

"$ROOT/scripts/capture-txt-toolbar-compare.sh"

cp -f "$OUT"/*.png "$ART"/
echo "Copied captures to $ART"

echo "=== assert unique capture hashes ==="
python3 - "$OUT" <<'PY'
import hashlib
import sys
from pathlib import Path

out = Path(sys.argv[1])
names = [
    "txt-web-markdown-picker-h2.png",
    "txt-desktop-markdown.png",
    "txt-docx-emulated-h1.png",
    "txt-sample-mod-toolbar.png",
    "compare-at-1280-markdown.png",
    "compare-at-1280-docx.png",
]
digests: dict[str, str] = {}
for name in names:
    path = out / name
    if not path.is_file():
        print(f"missing capture: {path}", file=sys.stderr)
        sys.exit(1)
    digest = hashlib.md5(path.read_bytes()).hexdigest()
    if digest in digests.values():
        clash = next(k for k, v in digests.items() if v == digest)
        print(f"duplicate md5 {digest}: {clash} and {name}", file=sys.stderr)
        sys.exit(1)
    digests[name] = digest
    print(f"{name}: md5={digest}")
PY
