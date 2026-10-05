#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo test --workspace
cd "$ROOT/crates/unibrowser-desktop"
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
npm run build
echo "check-workspace: ok"
