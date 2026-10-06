#!/usr/bin/env bash
# End-to-end UI tests: debug Unibrowser + @wdio/tauri-service (embedded WebDriver) under xvfb.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export UNIBROWSER_E2E=1

# Stale listeners make embedded WebDriver report "waiting for webview initialization" forever.
kill_stale_drivers() {
  killall -9 unibrowser-desktop 2>/dev/null || true
  killall -9 tauri-driver 2>/dev/null || true
  killall -9 WebKitWebDriver 2>/dev/null || true
  fuser -k 1420/tcp 2>/dev/null || true
  fuser -k 4445/tcp 2>/dev/null || true
  sleep 1
}

VITE_PID=""

stop_vite() {
  if [[ -n "${VITE_PID}" ]] && kill -0 "${VITE_PID}" 2>/dev/null; then
    kill "${VITE_PID}" 2>/dev/null || true
    wait "${VITE_PID}" 2>/dev/null || true
  fi
}

start_vite_preview() {
  # Tauri embeds devUrl (http://localhost:1420) for debug and often for icon paths in dev builds.
  # Serve the built dist on :1420 so WebDriver sees the real shell DOM.
  cd "$ROOT/crates/unibrowser-desktop"
  npm run preview -- --host 127.0.0.1 --port 1420 --strictPort >"$ROOT/e2e/.vite-preview.log" 2>&1 &
  VITE_PID=$!
  for _ in $(seq 1 90); do
    if curl -sf "http://127.0.0.1:1420/" 2>/dev/null | rg -q 'id="omnibox"'; then
      echo "Vite preview ready on :1420 (pid ${VITE_PID})"
      sleep 2
      return 0
    fi
    if ! kill -0 "${VITE_PID}" 2>/dev/null; then
      echo "Vite exited early; log:" >&2
      tail -40 "$ROOT/e2e/.vite-preview.log" >&2 || true
      exit 1
    fi
    sleep 0.5
  done
  echo "Timed out waiting for Vite on :1420" >&2
  tail -40 "$ROOT/e2e/.vite-preview.log" >&2 || true
  exit 1
}

trap stop_vite EXIT

run_tests() {

kill_stale_drivers

echo "=== Building UI ==="
cd "$ROOT/crates/unibrowser-desktop"
if [[ -f package-lock.json ]]; then npm ci; else npm install; fi
npm run build

echo "=== Building release desktop (e2e WebDriver plugins) ==="
cd "$ROOT"
cargo build -p unibrowser-media --bin unibrowser-media-sidecar --release
cargo build -p unibrowser-desktop --release --features e2e

mkdir -p "$ROOT/e2e/fixtures/projects"
python3 "$ROOT/e2e/scripts/make-fixtures.py"

echo "=== Installing e2e deps ==="
cd "$ROOT/e2e"
npm install --legacy-peer-deps

  sleep 2

  start_vite_preview

  echo "=== Starting Unibrowser (embedded WebDriver on :4445) ==="
  UNIBROWSER_E2E=1 "$ROOT/target/release/unibrowser-desktop" >/tmp/unibrowser-e2e-app.log 2>&1 &
  APP_PID=$!
  for _ in $(seq 1 90); do
    if curl -sf "http://127.0.0.1:4445/status" 2>/dev/null | rg -q '"ready":true'; then
      break
    fi
    if ! kill -0 "${APP_PID}" 2>/dev/null; then
      echo "Unibrowser exited during boot; log:" >&2
      tail -40 /tmp/unibrowser-e2e-app.log >&2 || true
      exit 1
    fi
    sleep 0.5
  done
  sleep 8

  if [[ "${CAPTURE_SCREENSHOTS:-0}" == "1" ]]; then
    echo "=== Capturing screenshots ==="
    (cd "$ROOT/e2e" && npm run screenshots)
  else
    echo "=== Running WebdriverIO specs ==="
    (cd "$ROOT/e2e" && npm test)
  fi
  TEST_EXIT=$?

  kill "${APP_PID}" 2>/dev/null || true
  wait "${APP_PID}" 2>/dev/null || true
  exit "${TEST_EXIT}"
}

if command -v xvfb-run >/dev/null 2>&1; then
  xvfb-run -a -s "-screen 0 1600x1000x24" env ROOT="$ROOT" UNIBROWSER_E2E=1 VITE_PID="" bash -c "
    $(declare -f kill_stale_drivers)
    $(declare -f stop_vite)
    $(declare -f start_vite_preview)
    $(declare -f run_tests)
    trap stop_vite EXIT
    cd \"\$ROOT\"
    run_tests
  "
else
  export DISPLAY="${DISPLAY:-:99}"
  run_tests
fi

echo "E2E finished OK"
exit 0
