#!/usr/bin/env bash
# Diagnose release shell rendering under xvfb (AppImage vs bare binary, WebKit env).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/docs"
LOG="$OUT_DIR/release-shell-diagnosis-last.txt"
APPIMAGE="$ROOT/target/release/bundle/appimage/Unibrowser_0.1.0_amd64.AppImage"
BIN="$ROOT/target/release/unibrowser-desktop"
MIN_PNG_BYTES=8000

mkdir -p "$OUT_DIR"
: >"$LOG"

log() {
  echo "$*" | tee -a "$LOG"
}

check_png() {
  local label="$1"
  local png="$2"
  local bytes
  bytes=$(wc -c <"$png" | tr -d ' ')
  log "  screenshot ${label}: ${bytes} bytes (${png})"
  if [[ "$bytes" -lt "$MIN_PNG_BYTES" ]]; then
    log "  FAIL: ${label} looks blank (under ${MIN_PNG_BYTES} bytes)"
    return 1
  fi
  log "  OK: ${label} likely shows UI"
  return 0
}

run_capture() {
  local png="$2"
  xvfb-run -a -s "-screen 0 1280x800x24" env WEBKIT_DISABLE_DMABUF_RENDERER=1 UB_LAUNCH="$1" bash -c "
    set -euo pipefail
    eval \"\$UB_LAUNCH\" &
    APP_PID=\$!
    sleep 10
    if command -v import >/dev/null 2>&1; then
      import -window root \"${png}\"
    else
      scrot \"${png}\"
    fi
    kill \"\$APP_PID\" 2>/dev/null || true
    wait \"\$APP_PID\" 2>/dev/null || true
  "
}

if [[ ! -x "$BIN" ]]; then
  log "Build release first: cargo build -p unibrowser-desktop --release"
  exit 1
fi

if ! command -v import >/dev/null 2>&1 && ! command -v scrot >/dev/null 2>&1; then
  echo "install imagemagick (import) or scrot" >&2
  exit 1
fi

log "=== Unibrowser release shell diagnosis $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
log ""

FAIL=0

log "1) Bare release binary"
PNG_BARE="/tmp/unibrowser-diag-bare-binary.png"
run_capture "$BIN" "$PNG_BARE"
check_png bare-binary "$PNG_BARE" || FAIL=1

log ""
log "2) AppImage --appimage-extract-and-run"
if [[ -x "$APPIMAGE" ]]; then
  PNG_IMG="/tmp/unibrowser-diag-appimage.png"
  run_capture "$APPIMAGE --appimage-extract-and-run" "$PNG_IMG"
  check_png appimage "$PNG_IMG" || FAIL=1
else
  log "  SKIP: AppImage not found at $APPIMAGE"
fi

log ""
log "3) Wrong launch (squashfs binary without AppRun) — documented false blank"
SQUASH_BIN="$ROOT/squashfs-root/unibrowser-desktop"
if [[ -x "$SQUASH_BIN" ]]; then
  PNG_WRONG="/tmp/unibrowser-diag-wrong-layout.png"
  xvfb-run -a -s "-screen 0 1280x800x24" bash -c "
    set -euo pipefail
    \"$SQUASH_BIN\" &
    APP_PID=\$!
    sleep 10
    import -window root \"$PNG_WRONG\" 2>/dev/null || scrot \"$PNG_WRONG\"
    kill \"\$APP_PID\" 2>/dev/null || true
    wait \"\$APP_PID\" 2>/dev/null || true
  " || true
  wrong_bytes=$(wc -c <"$PNG_WRONG" | tr -d ' ')
  log "  wrong-layout screenshot: ${wrong_bytes} bytes"
else
  log "  SKIP: no squashfs-root/unibrowser-desktop"
fi

log ""
log "Findings:"
log "  - Release shell renders when launched as target/release/unibrowser-desktop or AppImage with extract-and-run."
log "  - Tiny captures: wrong AppImage layout (WebKitNetworkProcess) or screenshot before paint."
log "  - On xvfb/VMs use WEBKIT_DISABLE_DMABUF_RENDERER=1 for stable WebKitGTK paint."
log "  - E2E: embedded WebDriver + loopback dist; external driver often sees empty DOM."

if [[ "$FAIL" -ne 0 ]]; then
  log ""
  log "OVERALL: FAIL"
  exit 1
fi

log ""
log "OVERALL: PASS"
exit 0
