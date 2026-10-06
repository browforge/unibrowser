# End-to-end UI tests

Unibrowser drives the **real Tauri shell** on Linux via `@wdio/tauri-service` and **tauri-driver** + WebKitWebDriver (`driverProvider: 'external'`). Release builds use **bundled `frontendDist`** (no Vite on port 1420).

## Stack

- **Runner:** WebdriverIO + `@wdio/tauri-service` (spawns `target/release/unibrowser-desktop` with `UNIBROWSER_E2E=1`).
- **App:** `cargo build -p unibrowser-desktop --release --features e2e` (WDIO plugins + loopback shell URL for headless WebKit).
- **Frontend:** Same bytes as `crates/unibrowser-desktop/dist/`, served on **http://127.0.0.1:18765** during E2E only (static `python3 -m http.server`, not Vite).
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
| WebDriver session + window handle | `e2e/specs/smoke.e2e.ts` | **Green** (default in `e2e.sh`) |
| Full shell flows (font, banner, undo, proofing, palette, CM, `.unibrow`) | `e2e/specs/shell.e2e.ts` | **Quarantined** — shell HTML/invoke timing on loopback URL under xvfb (see below) |
| Doc screenshots | `e2e/specs/screenshots.e2e.ts` | Optional (`CAPTURE_SCREENSHOTS=1`) |

Helpers use `browser.execute` → `window.__TAURI__.core.invoke` where the webview exposes the Tauri bridge.

## Reliability (2026-10-06)

On this VM, `./scripts/e2e.sh` (smoke only) was run **three times in a row** with **1 passed / 0 failed** each run (recorded 2026-10-06 UTC).

Harness details:

- Poll **`getWindowHandles()`** in `e2e/helpers/tauri.ts` (`waitForWindowHandle`).
- Optional **`document.body.dataset.ubReady`** (set from `main.ts` when shell chrome is present).
- WDIO **`connectionRetryCount: 3`**; `scripts/e2e.sh` retries the WDIO run up to **3** times on failure.
- Production release verified by `scripts/verify-release-frontend.sh` (**no `localhost:1420`** in the shipping binary).

## Quarantined: `shell.e2e.ts`

Excluded in `e2e/wdio.conf.ts` until loopback E2E reliably loads the shell with Tauri invoke (WebDriver sometimes attaches before `http://127.0.0.1:18765/index.html` paints). Re-enable by removing the `exclude` entry after fixing navigation/ACL on the E2E URL.

## Troubleshooting

- **Stale port 4444/4445:** zombie `unibrowser-desktop` or `tauri-driver` — `scripts/e2e.sh` kills listeners before each attempt.
- **Embedded WebDriver `ready:false`:** spawn with `WDIO_EMBEDDED_SERVER=true` (the external driver path does not need this).
- **No `import`/`scrot`:** install ImageMagick or scrot for `scripts/capture-docs-screenshots.sh`.

## CI note

E2E is not wired into GitHub Actions yet. Run `./scripts/e2e.sh` on Linux before release candidates.
