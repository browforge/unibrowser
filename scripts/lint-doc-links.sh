#!/usr/bin/env bash
# Check relative markdown links from docs/index.md (and optional extra files).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

check_file() {
  local md="$1"
  local dir
  dir="$(dirname "$md")"
  python3 - "$md" "$dir" <<'PY'
import re, sys
from pathlib import Path
md_path, base = sys.argv[1], Path(sys.argv[2])
text = Path(md_path).read_text(encoding="utf-8")
broken = []
for link in re.findall(r'\]\(([^)]+)\)', text):
    if link.startswith(("http://", "https://", "#", "mailto:")):
        continue
    target = (base / link.split("#", 1)[0]).resolve()
    if not target.exists():
        broken.append(link)
if broken:
    print(f"{md_path}: {len(broken)} broken link(s)", file=sys.stderr)
    for b in broken:
        print(f"  - {b}", file=sys.stderr)
    sys.exit(1)
print(f"{md_path}: ok")
PY
}

check_file "docs/index.md"
if [[ "${1:-}" == "--all" ]]; then
  while IFS= read -r -d '' f; do
    check_file "$f"
  done < <(find docs -name '*.md' -print0)
fi
