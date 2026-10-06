# End-to-end UI tests

Unibrowser drives the **real Tauri shell** on Linux via `@wdio/tauri-service` with the **embedded** WebDriver server (`tauri-plugin-wdio-webdriver`, `driverProvider: 'embedded'`). Helpers use **`browser.tauri.execute`** (direct eval in the app webview). Run only under **xvfb** (`./scripts/e2e.sh`). **Shipping** builds use bundled `frontendDist` (no Vite on port 1420). **E2E release** (`--features e2e`) points the main window at the same `dist/` bytes on **http://127.0.0.1:18765** (static server in `scripts/e2e.sh`) with capability `e2e-loopback` so WebKitGTK + invoke work under xvfb.

## Stack

- **Runner:** WebdriverIO + `@wdio/tauri-service` (spawns `target/release/unibrowser-desktop` with `UNIBROWSER_E2E=1`).
- **App:** `cargo build -p unibrowser-desktop --release --features e2e` (WDIO plugins + loopback window URL; capability `capabilities/e2e-loopback.json`).
- **Display:** `xvfb-run` at **1600x1000**.

## Run locally

```bash
./scripts/e2e.sh
```

Doc screenshots (xvfb):

```bash
./scripts/capture-docs-screenshots.sh
```

## What is covered

| Scenario | Spec | Status |
| --- | --- | --- |
| Smoke (`get_browser_state`) | `e2e/specs/10-shell.e2e.ts` (first `it`) | Same session as shell |
| Full shell flows (font, banner, undo, proofing, palette, CM, `.unibrow`) | `e2e/specs/10-shell.e2e.ts` | **Default** (loopback `dist/` + `e2e-loopback` capability; flaky on slow VMs — retry harness) |
| Doc screenshots | `e2e/specs/screenshots.e2e.ts` | Optional (`CAPTURE_SCREENSHOTS=1`) |

Helpers use `browser.execute` → `window.__TAURI__.core.invoke` on the bundled webview.

## Reliability

`scripts/e2e.sh` retries WDIO up to **3** times; production release is checked with `scripts/verify-release-frontend.sh` (**no `localhost:1420`**). Run **three consecutive** full `./scripts/e2e.sh` passes before release candidates. `e2e/wdio.conf.ts` defaults to `driverProvider: 'embedded'` and `WEBKIT_DISABLE_DMABUF_RENDERER=1`. Override with `E2E_DRIVER_PROVIDER=external` if you use system `tauri-driver` + `webkit2gtk-driver`. On headless cloud VMs (Oct 2026), embedded often fails session create (`No window could be found`); external may connect but see an empty WebDriver page until the driver attaches to the shell webview — run `./scripts/diagnose-release-shell.sh` to separate **blank release** from **WDIO attachment** issues.

## Troubleshooting

- **Stale port 4444/4445:** zombie `unibrowser-desktop` or `tauri-driver` — `scripts/e2e.sh` kills listeners before each attempt.
- **Embedded WebDriver `ready:false`:** spawn with `WDIO_EMBEDDED_SERVER=true` (the external driver path does not need this).
- **No `import`/`scrot`:** install ImageMagick or scrot for `scripts/capture-docs-screenshots.sh`.

## CI note

E2E is not wired into GitHub Actions yet. Run `./scripts/e2e.sh` on Linux before release candidates.
