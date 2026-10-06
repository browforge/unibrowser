# End-to-end UI tests

Unibrowser ships browser automation tests that drive the **real Tauri shell** via the embedded W3C WebDriver server (`tauri-plugin-wdio-webdriver`, enabled with the `e2e` Cargo feature on release builds).

## Stack

- **Runner:** [WebdriverIO](https://webdriver.io/) connecting to `http://127.0.0.1:4445` (embedded plugin server).
- **App:** `target/release/unibrowser-desktop` built with `--features e2e`.
- **Frontend:** `vite preview` on **http://127.0.0.1:1420** serves `dist/` (the embedded config still references `:1420` for dev assets).
- **Display:** `xvfb-run` at **1600x1000** on headless Linux.

## Run locally

```bash
./scripts/e2e.sh
```

The script kills stale listeners on **1420/4445**, builds UI + release binary, starts preview, launches Unibrowser, then runs WDIO specs.

Capture doc screenshots:

```bash
CAPTURE_SCREENSHOTS=1 ./scripts/e2e.sh
```

## What is covered

| Scenario | Spec |
| --- | --- |
| Text + **Increase font size** | `e2e/specs/shell.e2e.ts` |
| SVG + **Increase font size** | same |
| Unknown extension **banner** (not modal) | same |
| Close dirty tab + **Undo** | same |
| Spell proofing + **workspace dictionary** | same |
| **Ctrl+Shift+P** palette command | same |
| Command Manager **disabled** chips across tab types | same |
| Blank `.unibrow` **autosave** (zero clips) | same |

Helpers call `window.__TAURI__.core.invoke` for setup (`open_path`, proofing, commands).

## Troubleshooting

- **Stale `/status` on 4445:** a zombie `unibrowser-desktop` can make WebDriver report `waiting for webview initialization` forever. `scripts/e2e.sh` runs `fuser -k` on 4445/1420 before tests.
- **Empty shell in WebDriver:** ensure preview is up (`curl -s http://127.0.0.1:1420/ | rg omnibox`).
- **Session `No window could be found`:** still under investigation on some agents; retry after freeing ports or increase warm-up in `e2e/specs/shell.e2e.ts` `before` hook.

## CI note

E2E is not wired into GitHub Actions yet. Run `./scripts/e2e.sh` on Linux before release candidates.
