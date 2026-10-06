#!/usr/bin/env bash
# Rebuild release desktop when frontend dist is newer than the binary (Tauri embeds dist at compile time).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/unibrowser-desktop"
DIST="$ROOT/crates/unibrowser-desktop/dist"
INDEX="$DIST/index.html"
STAMP="$DIST/.ub-frontend-stamp"

if [[ ! -f "$INDEX" ]]; then
  echo "Missing $INDEX — run: (cd crates/unibrowser-desktop && npm run build)" >&2
  exit 1
fi
if [[ ! -f "$STAMP" ]]; then
  echo "Missing $STAMP — run: (cd crates/unibrowser-desktop && npm run build)" >&2
  exit 1
fi

need_build=0
if [[ ! -x "$BIN" ]]; then
  need_build=1
elif [[ "$BIN" -ot "$INDEX" ]] || [[ "$BIN" -ot "$STAMP" ]]; then
  need_build=1
else
  while IFS= read -r -d '' asset; do
    if [[ "$asset" -nt "$BIN" ]]; then
      need_build=1
      break
    fi
  done < <(find "$DIST" -type f -print0 2>/dev/null)
fi

if [[ "$need_build" -eq 1 ]]; then
  echo "ensure-release-desktop: rebuilding (binary older than dist or missing)"
  (cd "$ROOT/crates/unibrowser-desktop" && npm run build)
  (cd "$ROOT" && cargo build -p unibrowser-desktop --release)
fi

if [[ ! -x "$BIN" ]]; then
  echo "Release binary missing after build: $BIN" >&2
  exit 1
fi

DIST_SHA=$(tr -d '\n' <"$STAMP")
EMBED_STAMP_FILE=$(ls -t "$ROOT/target/release/build/unibrowser-desktop-"*/out/frontend-stamp.txt 2>/dev/null | head -1 || true)
if [[ -z "$EMBED_STAMP_FILE" ]] || [[ ! -f "$EMBED_STAMP_FILE" ]]; then
  echo "Missing embedded frontend stamp (cargo build did not record dist/.ub-frontend-stamp)" >&2
  exit 1
fi
EMBED_SHA=$(tr -d '\n' <"$EMBED_STAMP_FILE")
if [[ "$DIST_SHA" != "$EMBED_SHA" ]]; then
  echo "Frontend stamp mismatch: dist=$DIST_SHA embedded=$EMBED_SHA (stale release binary)" >&2
  exit 1
fi

echo "$DIST_SHA" >"$ROOT/target/release/.unibrowser-frontend-index-sha256"
echo "ensure-release-desktop: ok (index sha256 $DIST_SHA)"
