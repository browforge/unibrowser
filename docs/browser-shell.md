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

## Issue: WebView2 `DownloadStarting` (Windows)

**Tracking:** use GitHub issue template [WebView2 download hook](../.github/ISSUE_TEMPLATE/webview2-download.yml) when opening a follow-up.

**Spike in repo:** `webview_download.rs` exposes `emit_unibrowser_download` (shared with Linux) and `webview2_download_starting_stub` on Windows. Wire the stub from a future WRY/Tauri `DownloadStarting` callback so behaviour matches Linux.

### Acceptance criteria

1. Navigating an `https://` tab and triggering a file download (Content-Disposition attachment or WebView2 download UI) emits **`unibrowser-download`** on the owning window with a real `url` and optional `destination` path when the platform provides one.
2. The existing `main.ts` listener continues to honour **download preference** (open in tab vs save) and **Open downloads in system browser** without duplicate tabs or silent failures.
3. `apply_download_disposition` receives the same shape of inputs as Linux (`url`, suggested path, cwd, preference); integration tests in `download_open_tab.rs` stay green.
4. `unibrowser://diagnostics` reports `web_download_hook_wired: true` on Windows after the hook ships.
5. Documented manual QA: one small HTTPS test file, one redirect download, cancel path does not corrupt download history.

### Interim (today)

- Linux: `WebviewBuilder::on_download` in `webview.rs` → `emit_unibrowser_download`.
- Windows / macOS: hook not wired; Advanced setting **Open web downloads in system browser** remains the supported workaround.

## Related

- [downloads.md](downloads.md)
- [multi-window.md](multi-window.md)
