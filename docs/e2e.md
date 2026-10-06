# End-to-end UI tests

**Default:** `./scripts/e2e.sh` runs the **in-app self-test** (no WebDriver). Optional WDIO: `E2E_WDIO=1 ./scripts/e2e.sh`.

## In-app self-test (recommended)

Runs inside the real Tauri shell on Linux under **xvfb**, using Rust command handlers (same code paths as `invoke`) plus optional shell PNG captures.

```bash
./scripts/self-test.sh          # 3 consecutive passes
./scripts/e2e.sh                # build + verify-release-frontend + self-test
```

CLI:

```bash
target/release/unibrowser-desktop --self-test <scenario|all> --report out.json
```

| Scenario id | What it checks |
| --- | --- |
| `proofing-dictionary` | `proofing_check` hit + workspace dictionary add |
| `problems-jump` | Problems list + text navigation target |
| `palette-command` | Command palette toggle (full desktop) |
| `font-text-svg` | Font command on text + SVG |
| `unknown-file-banner` | Non-modal unknown-file banner |
| `close-undo` | Close dirty tab + undo |
| `cm-grey-tab-switch` | Disabled commands on image tab |
| `unibrow-blank-save` | Blank `.unibrow` autosave |
| `live-preview` | Open demo `.unibrow`, enable live mode, assert `live_mode` |
| `ai-sidebar` | Mock AI endpoint, grants prefs, `test_ai_connection`, panel configured |
| `all` | All ten scenarios in order |

`scripts/self-test.sh` runs `all` **3×** under xvfb (`WEBKIT_DISABLE_DMABUF_RENDERER=1`). The Rust runner skips `browser-state-updated` events during self-test so file opens do not deadlock the UI thread. Tab mutex re-entrancy is rejected in debug builds; see [concurrency.md](concurrency.md).

Environment:

- `UNIBROWSER_REPO_ROOT` — repo path for fixtures (default: cwd)
- `WEBKIT_DISABLE_DMABUF_RENDERER=1` — set by scripts on xvfb
- `UNIBROWSER_SELF_TEST_SKIP_SHOTS=1` — skip PNG captures (faster)
- `SELF_TEST_RUNS` / `SELF_TEST_TIMEOUT_SECS` — harness tuning

Reports: JSON at `--report`; PNGs beside it in `self-test-shots/`.

## WebdriverIO (optional)

`E2E_WDIO=1 ./scripts/e2e.sh` builds `--features e2e`, serves loopback `dist/` on **18765**, and runs `e2e/specs/10-shell.e2e.ts` via `@wdio/tauri-service` (`driverProvider: embedded` by default). Still flaky on headless VMs (empty DOM or “No window”); use self-test for release gates.

See `e2e/wdio.conf.ts`, `docs/diagnose-release-shell.sh` (blank shell vs WDIO).

## Doc screenshots

```bash
./scripts/capture-docs-screenshots.sh
```

Uses `UNIBROWSER_DEMO_STATE` presets under xvfb (`WEBKIT_DISABLE_DMABUF_RENDERER=1`, `WEBKIT_DISABLE_COMPOSITING_MODE=1`). The shell waits for paint, then `--doc-capture` writes a composited X11 root crop (includes native child webviews; not shell-webview snapshot alone).

## CI note

Not wired into GitHub Actions yet. Run `./scripts/e2e.sh` on Linux before release candidates.
