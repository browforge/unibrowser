#!/usr/bin/env bash
# Scaffold a WASM mod from examples/ub-mod-wasm.
set -euo pipefail
NAME="${1:-}"
OUT="${2:-}"
if [[ -z "$NAME" || -z "$OUT" ]]; then
  echo "Usage: $0 <mod-name> <output-directory>" >&2
  exit 1
fi
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TEMPLATE="$ROOT/examples/ub-mod-wasm"
if [[ ! -d "$TEMPLATE" ]]; then
  echo "template missing: $TEMPLATE" >&2
  exit 1
fi
mkdir -p "$OUT"
cp -r "$TEMPLATE/src" "$OUT/"
cp "$TEMPLATE/Cargo.toml" "$OUT/"
cp "$TEMPLATE/info.json" "$OUT/"
MOD_CRATE="${NAME//-/_}"
sed -i "s/ub-mod-wasm/$NAME/g; s/ub_mod_wasm/${MOD_CRATE}/g" "$OUT/Cargo.toml" "$OUT/info.json" "$OUT/src/lib.rs" 2>/dev/null || true
sed -i "s/ub-mod-wasm/$NAME/g; s/ub_mod_wasm/${MOD_CRATE}/g" "$OUT/Cargo.toml" "$OUT/info.json" "$OUT/src/lib.rs"
echo "Scaffolded WASM mod at $OUT"
echo "Build: (cd $OUT && cargo build --target wasm32-unknown-unknown)"
echo "Install: copy folder + ${MOD_CRATE}.wasm into user mods directory"
