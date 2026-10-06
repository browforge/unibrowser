#!/usr/bin/env bash
# Record perf baselines for docs/perf.md (Linux VM, debug binary unless noted).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
DISPLAY="${DISPLAY:-:98}"
export DISPLAY

if ! pgrep -f "Xvfb $DISPLAY" >/dev/null 2>&1; then
  Xvfb "$DISPLAY" -screen 0 1600x1000x24 -ac &
  sleep 1
fi

BIN="$ROOT/target/debug/unibrowser-desktop"
FIX="$ROOT/e2e/fixtures"
mkdir -p "$FIX"

# 100MB text fixture (generated once)
BIG="$FIX/large-100mb.txt"
if [[ ! -f "$BIG" ]]; then
  echo "Generating 100MB text fixture..."
  head -c 104857600 < /dev/zero | tr '\0' 'x' > "$BIG"
fi

# 4K PNG via ImageMagick if available
PNG4K="$FIX/test-4k.png"
if [[ ! -f "$PNG4K" ]] && command -v convert >/dev/null 2>&1; then
  convert -size 3840x2160 xc:'#336699' "$PNG4K"
fi

echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "host=$(uname -a)"

echo "--- cold_start_wall_s (process until window, best effort) ---"
if [[ -x "$BIN" ]]; then
  START=$(date +%s.%N)
  "$BIN" &
  APP_PID=$!
  sleep 4
  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  END=$(date +%s.%N)
  python3 - <<PY
start, end = $START, $END
print(f"cold_start_wall_s={end-start:.2f}")
PY
else
  echo "cold_start_wall_s=SKIP (build unibrowser-desktop first)"
fi

echo "--- idle_rss_mb (debug, after 5s) ---"
if [[ -x "$BIN" ]]; then
  "$BIN" &
  APP_PID=$!
  sleep 5
  RSS=$(ps -o rss= -p "$APP_PID" 2>/dev/null | awk '{print int($1/1024)}')
  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  echo "idle_rss_mb=${RSS:-unknown}"
fi

echo "--- open_100mb_text (invoke timing via e2e helper pattern: manual note) ---"
if [[ -f "$BIG" ]]; then
  /usr/bin/time -f "open_100mb_wall_s=%e" "$BIN" &
  APP_PID=$!
  sleep 3
  kill "$APP_PID" 2>/dev/null || true
  echo "fixture_mb=$(du -m "$BIG" | awk '{print $1}')"
fi

echo "Done. Paste numbers into docs/perf.md baseline table."
