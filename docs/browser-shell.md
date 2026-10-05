# Browser shell: web content and downloads

Unibrowser embeds a per-window **content webview** for `https://` tabs ([ADR 0009](adr/0009-browser-as-unified-shell.md)).

## Download interception by platform

| Platform | WebView | Download hook | Status in repo |
| --- | --- | --- | --- |
| Linux | WebKitGTK (Tauri 2) | `WebviewBuilder::on_download` → `DownloadEvent::Requested` | **Wired** — emits `unibrowser-download` ([`webview.rs`](../crates/unibrowser-desktop/src-tauri/src/webview.rs)) |
| Windows | WebView2 | `ICoreWebView2::DownloadStarting` | **Not wired** — no native delegate yet |
| macOS | WKWebView | Platform download delegate | **Not wired** in this slice |

Until WebView2/macOS delegates exist, Windows users rely on:

- **Settings → Advanced → Open downloads in system browser** (interim): `unibrowser-download` opens the URL with the system browser instead of `handle_download_url`.
- **Demo download** / omnibox flows that call `handle_download_url` directly (still work).

## Frontend flow (Linux)

1. WebKit requests a download → Rust emits `unibrowser-download` with `url` + optional `destination`.
2. `main.ts` listener checks prefs:
   - `open_downloads_in_system_browser` → `open_url_externally` (opener plugin).
   - Else → `handle_download_url` (open in tab vs save to disk per [downloads.md](downloads.md)).

## Disposition logic

Shared Rust helper: `downloads::apply_download_disposition` (see integration tests in `download_open_tab.rs`).

## Related

- [downloads.md](downloads.md)
- [multi-window.md](multi-window.md)
