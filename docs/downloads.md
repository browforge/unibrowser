# Downloads and OpenInNewTab

Default disposition (ADR 0009): **open in a new tab**, not hand off to the OS default app.

## Implemented (Phase 1 MVP)

| Path | Behaviour |
| --- | --- |
| `handle_download_url` command | Opens `https://` (or file path) in a **new tab** via mod resolution |
| Demo toolbar button | Calls the same command with `https://example.com/` |
| `WebviewBuilder::on_download` (content webview) | Emits `unibrowser-download` to the shell; returns `false` to cancel default save when supported |

## Platform gaps

| Platform | Status |
| --- | --- |
| **Linux WebKitGTK** | `on_download` wired; verify `Requested` fires for your target sites |
| **Windows WebView2** | Hook point documented in `downloads.rs`; full `DownloadStarting` integration deferred |
| **macOS** | Non-target |

When `Finished` fires with an on-disk path, future work should open `file://` in a tab via `base-files` routing.
