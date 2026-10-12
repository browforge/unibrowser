#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/crates/unibrowser-desktop/public/wasm"
mkdir -p "$OUT"
cd "$ROOT"
cargo build -p unibrowser-web-host --release --target wasm32-unknown-unknown
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  cargo install wasm-bindgen-cli --locked 2>/dev/null || true
fi
WASM_BINDGEN=wasm-bindgen
"$WASM_BINDGEN" \
  --target web \
  --out-dir "$OUT" \
  --no-typescript \
  "$ROOT/target/wasm32-unknown-unknown/release/unibrowser_web_host.wasm"
echo "WASM host written to $OUT"
