#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build -p ub-mod-cdylib-stub
SO="$ROOT/target/debug/libub_mod_cdylib_stub.so"
if [[ ! -f "$SO" ]]; then
  echo "missing $SO" >&2
  exit 1
fi
STAGE="$(mktemp -d)"
cp "$ROOT/examples/hello-world-ubmod/info.json" "$STAGE/info.json"
cp "$SO" "$STAGE/libub_mod_cdylib_stub.so"
OUT="$ROOT/examples/hello-world.ubmod"
rm -f "$OUT"
(
  cd "$STAGE"
  zip -q -r "$OUT" .
)
rm -rf "$STAGE"
echo "wrote $OUT"
