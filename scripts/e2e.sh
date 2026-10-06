#!/usr/bin/env bash
# End-to-end UI tests: release Unibrowser + embedded WebDriver under xvfb (bundled dist, no Vite).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export UNIBROWSER_E2E=1
export UNIBROWSER_E2E_ASSET_BASE="http://127.0.0.1:18765/index.html"

kill_port_listeners() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -ti "tcp:${port}" 2>/dev/null | xargs -r kill -9 2>/dev/null || true
  elif command -v fuser >/dev/null 2>&1; then
    fuser -k "${port}/tcp" 2>/dev/null || true
  fi
}

kill_stale_drivers() {
  if command -v killall >/dev/null 2>&1; then
    killall -9 unibrowser-desktop 2>/dev/null || true
    killall -9 tauri-driver 2>/dev/null || true
    killall -9 WebKitWebDriver 2>/dev/null || true
  fi
  pgrep -f '/unibrowser-desktop$' 2>/dev/null | xargs -r kill -9 2>/dev/null || true
  kill_port_listeners 1420
  kill_port_listeners 18765
  kill_port_listeners 4444
  kill_port_listeners 4445
  sleep 1
}

run_tests() {
  kill_stale_drivers

  echo "=== Building UI ==="
  cd "$ROOT/crates/unibrowser-desktop"
  if [[ -f package-lock.json ]]; then npm ci; else npm install; fi
  npm run build

  echo "=== Building production release (bundled frontendDist) ==="
  cd "$ROOT"
  cargo build -p unibrowser-media --bin unibrowser-media-sidecar --release
  cargo build -p unibrowser-desktop --release

  echo "=== Verifying production release (no localhost:1420) ==="
  "$ROOT/scripts/verify-release-frontend.sh"

  echo "=== Building release desktop (e2e WebDriver plugins) ==="
  cargo build -p unibrowser-desktop --release --features e2e

  mkdir -p "$ROOT/e2e/fixtures/projects"
  python3 "$ROOT/e2e/scripts/make-fixtures.py"

  echo "=== Installing e2e deps ==="
  cd "$ROOT/e2e"
  npm install --legacy-peer-deps

  sleep 2

  DIST="$ROOT/crates/unibrowser-desktop/dist"
  E2E_ASSET_PORT=18765
  kill_port_listeners "${E2E_ASSET_PORT}"
  python3 -m http.server "${E2E_ASSET_PORT}" --bind 127.0.0.1 --directory "${DIST}" >/tmp/unibrowser-e2e-assets.log 2>&1 &
  ASSET_PID=$!
  for _ in $(seq 1 40); do
    if curl -sf "http://127.0.0.1:${E2E_ASSET_PORT}/index.html" | rg -q 'id="omnibox"'; then
      break
    fi
    sleep 0.25
  done

  echo "=== Warming GTK/WebKit under xvfb (embedded WebDriver) ==="
  sleep 5

  echo "=== WebdriverIO (@wdio/tauri-service, dist/ on :${E2E_ASSET_PORT}, no Vite) ==="

  run_wdio() {
    if [[ "${CAPTURE_SCREENSHOTS:-0}" == "1" ]]; then
      (cd "$ROOT/e2e" && npm run screenshots)
    else
      (cd "$ROOT/e2e" && npm test)
    fi
  }

  TEST_EXIT=1
  for attempt in 1 2 3; do
    echo "=== WebdriverIO attempt ${attempt}/3 ==="
    kill_stale_drivers
    if run_wdio; then
      TEST_EXIT=0
      break
    fi
    echo "WDIO attempt ${attempt} failed" >&2
    sleep 3
  done

  kill "${ASSET_PID}" 2>/dev/null || true
  wait "${ASSET_PID}" 2>/dev/null || true
  kill_stale_drivers
  exit "${TEST_EXIT}"
}

E2E_EXIT=1
if command -v xvfb-run >/dev/null 2>&1; then
  xvfb-run -a -s "-screen 0 1600x1000x24" env ROOT="$ROOT" UNIBROWSER_E2E=1 UNIBROWSER_E2E_ASSET_BASE="http://127.0.0.1:18765/index.html" WEBKIT_DISABLE_DMABUF_RENDERER=1 bash -c "
    set -euo pipefail
    $(declare -f kill_port_listeners)
    $(declare -f kill_stale_drivers)
    $(declare -f run_tests)
    cd \"\$ROOT\"
    run_tests
  " && E2E_EXIT=0
else
  export DISPLAY="${DISPLAY:-:99}"
  run_tests && E2E_EXIT=0
fi

if [[ "$E2E_EXIT" -eq 0 ]]; then
  echo "E2E finished OK"
fi
exit "$E2E_EXIT"
