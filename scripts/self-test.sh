#!/usr/bin/env bash
# In-app shell UI scenarios (no WebDriver). Runs under xvfb; retries up to 3 times.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RUNS="${SELF_TEST_RUNS:-3}"
TIMEOUT_SECS="${SELF_TEST_TIMEOUT_SECS:-240}"
BIN="$ROOT/target/release/unibrowser-desktop"
REPORT_DIR="${SELF_TEST_REPORT_DIR:-$ROOT/target/self-test-reports}"

if [[ ! -x "$BIN" ]]; then
  echo "Build release desktop first: cargo build -p unibrowser-desktop --release" >&2
  exit 1
fi

mkdir -p "$REPORT_DIR"

run_once() {
  local attempt="$1"
  local report="$REPORT_DIR/run-${attempt}.json"
  rm -rf "${report%.json}/self-test-shots"
  echo "=== Self-test attempt ${attempt}/${RUNS} -> ${report} ==="
  timeout "${TIMEOUT_SECS}" xvfb-run -a -s "-screen 0 1600x1000x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    UNIBROWSER_SELF_TEST_SKIP_SHOTS="${UNIBROWSER_SELF_TEST_SKIP_SHOTS:-0}" \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    UNIBROWSER_SELF_TEST_HEADLESS=1 \
    "$BIN" --self-test all --report "$report"
}

FAIL=0
for attempt in $(seq 1 "$RUNS"); do
  if run_once "$attempt"; then
    echo "Self-test attempt ${attempt} OK"
  else
    echo "Self-test attempt ${attempt} failed" >&2
    FAIL=1
  fi
  sleep 2
done

if [[ "$FAIL" -ne 0 ]]; then
  echo "Self-test failed after ${RUNS} attempts" >&2
  exit 1
fi
echo "Self-test passed ${RUNS}/${RUNS} runs"
exit 0
