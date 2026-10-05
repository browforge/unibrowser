# Browser shell: web content and downloads

Unibrowser embeds a per-window **content webview** for `https://` tabs ([ADR 0009](adr/0009-browser-as-unified-shell.md)).

## Download interception by platform

| Platform | WebView | Download hook | Status in repo |
| --- | --- | --- | --- |
| Linux | WebKitGTK (Tauri 2) | `WebviewBuilder::on_download` → `DownloadEvent::Requested` | **Wired** — emits `unibrowser-download` ([`webview.rs`](../crates/unibrowser-desktop/src-tauri/src/webview.rs)) |
| Windows | WebView2 | `ICoreWebView2_4::DownloadStarting` | **Wired in code** (`webview_download_win.rs`); compiles on `windows-latest`; manual QA on hardware — [windows-testing.md](windows-testing.md) |
| macOS | WKWebView | Platform download delegate | **Not wired** in this slice |

On Windows, if the native hook misbehaves, **Settings → Advanced → Open downloads in system browser** still hands off to the system browser. **Demo download** / omnibox flows that call `handle_download_url` directly still work.

## Frontend flow (Linux)

1. WebKit requests a download → Rust emits `unibrowser-download` with `url` + optional `destination`.
2. `main.ts` listener checks prefs:
   - `open_downloads_in_system_browser` → `open_url_externally` (opener plugin).
   - Else → `handle_download_url` (open in tab vs save to disk per [downloads.md](downloads.md)).

## Disposition logic

Shared Rust helper: `downloads::apply_download_disposition` (see integration tests in `download_open_tab.rs`).

## Issue: WebView2 `DownloadStarting` (Windows)

**Tracking:** use GitHub issue template [WebView2 download hook](../.github/ISSUE_TEMPLATE/webview2-download.yml) when opening a follow-up.

**Implementation:** `webview_download_win.rs` registers `DownloadStarting` on the content webview via Tauri `with_webview` + `ICoreWebView2_4`, cancels the default WebView2 download, and calls `emit_unibrowser_download` (same as Linux `on_download`).

### Acceptance criteria

1. Navigating an `https://` tab and triggering a file download (Content-Disposition attachment or WebView2 download UI) emits **`unibrowser-download`** on the owning window with a real `url` and optional `destination` path when the platform provides one.
2. The existing `main.ts` listener continues to honour **download preference** (open in tab vs save) and **Open downloads in system browser** without duplicate tabs or silent failures.
3. `apply_download_disposition` receives the same shape of inputs as Linux (`url`, suggested path, cwd, preference); integration tests in `download_open_tab.rs` stay green.
4. `unibrowser://diagnostics` reports `web_download_hook_wired: true` on Windows after the hook ships.
5. Documented manual QA: one small HTTPS test file, one redirect download, cancel path does not corrupt download history.

### Interim (today)

- Linux: `WebviewBuilder::on_download` in `webview.rs` → `emit_unibrowser_download`.
- Windows: `webview_download_win.rs` → same event (verify on hardware).
- macOS: hook not wired; use system-browser workaround until WKWebView delegate exists.

## Related

- [downloads.md](downloads.md)
- [multi-window.md](multi-window.md)
