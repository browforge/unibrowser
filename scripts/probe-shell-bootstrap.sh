#!/usr/bin/env bash
# Normal startup (no --doc-capture): must reach shell-bootstrap-done before timeout.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=ensure-release-desktop.sh
source "$ROOT/scripts/ensure-release-desktop.sh"

BIN="$ROOT/target/release/unibrowser-desktop"
FIXTURE="$ROOT/e2e/fixtures/spell-test.txt"
TIMEOUT_SECS="${SHELL_BOOTSTRAP_PROBE_TIMEOUT_SECS:-120}"
LOG="$(mktemp -t ub-shell-bootstrap.XXXXXX.log)"

seed_recovery_journal() {
  UNIBROWSER_REPO_ROOT="$ROOT" cargo test -p unibrowser-desktop --lib \
    edit_recovery::shell_bootstrap_probe::shell_bootstrap_probe_seed_journal -- \
    --exact --nocapture
}

probe_one() {
  local label="$1"
  shift
  echo "=== shell bootstrap probe: ${label} (timeout ${TIMEOUT_SECS}s) ==="
  : >"$LOG"
  set +e
  timeout "${TIMEOUT_SECS}" xvfb-run -a -s "-screen 0 1280x800x24" env \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    UNIBROWSER_REPO_ROOT="$ROOT" \
    UNIBROWSER_SAFE_MODE=1 \
    "$@" \
    "$BIN" --open "$FIXTURE" 2>&1 | tee "$LOG"
  local status=${PIPESTATUS[0]}
  set -e
  if grep -q '\[self-test\] ping shell-bootstrap-done' "$LOG"; then
    echo "OK: ${label} reached shell-bootstrap-done"
    return 0
  fi
  echo "FAIL: ${label} did not reach shell-bootstrap-done (exit ${status})" >&2
  tail -n 40 "$LOG" >&2 || true
  return 1
}

FAIL=0
probe_one "open-only" || FAIL=1

seed_recovery_journal
probe_one "open-with-pending-recovery-journal" || FAIL=1

rm -f "$LOG"
exit "$FAIL"
