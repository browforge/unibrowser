#!/usr/bin/env bash
# Generate NOTICE.md for third-party dependencies.
# Uses cargo-license when installed; otherwise writes a maintained stub.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$ROOT/NOTICE.md}"

write_stub() {
  cat >"$OUT" <<'STUB'
# Third-party notices (generated stub)

This file is a placeholder. Re-run `./scripts/generate-notice.sh` after installing
`cargo-license` (`cargo install cargo-license`) for a full dependency listing.

## Key runtime dependencies (hand-maintained)

| Component | Licence | Notes |
| --- | --- | --- |
| Tauri 2 | MIT / Apache-2.0 | Desktop shell |
| Monaco Editor | MIT | Text editor UI |
| pdf.js | Apache-2.0 | PDF viewer (bundled assets) |
| serde / serde_json | MIT / Apache-2.0 | Serialization |
| ffmpeg (sidecar, when used) | LGPL 2.1+ | Media probing / transcode |

Unibrowser application code is licensed under Apache-2.0 (see LICENSE).
STUB
  echo "Wrote stub NOTICE to $OUT"
}

if command -v cargo-license >/dev/null 2>&1; then
  {
    echo "# Third-party notices"
    echo
    echo "Generated with \`cargo license\` on $(date -u +%Y-%m-%d)."
    echo
    cargo license --avoid-build-deps --avoid-dev-deps --json \
      | python3 -c '
import json, sys
rows = json.load(sys.stdin)
print("| Crate | Version | Licence |")
print("| --- | --- | --- |")
for r in sorted(rows, key=lambda x: x.get("name", "")):
    name = r.get("name", "?")
    ver = r.get("version", "?")
    lic = r.get("license", "?")
    print(f"| {name} | {ver} | {lic} |")
' 2>/dev/null || cargo license --avoid-build-deps --avoid-dev-deps
  } >"$OUT"
  echo "Wrote NOTICE to $OUT (cargo-license)"
else
  write_stub
fi
