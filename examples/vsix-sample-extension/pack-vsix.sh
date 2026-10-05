#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
OUT="$ROOT/unibrowser.hello-sample-0.1.0.vsix"
rm -f "$OUT"
(cd "$ROOT" && zip -r "$OUT" extension -x "*.DS_Store")
echo "Wrote $OUT"
