# Downloads and disposition (ADR 0009)

Default disposition: **open in a new tab** unless the user opts into **save to disk** in Recent → Downloads.

## Shell commands and events

| Entry point | Behaviour |
| --- | --- |
| `handle_download_url` | Resolves URL via `apply_download_disposition` and user `download_preference` |
| `unibrowser-download` event | Payload `{ url, destination? }` from the content webview |
| Demo toolbar button | Same as `handle_download_url` for `https://example.com/` |

### Preferences

- `open_in_new_tab` (default): HTTP(S) and safe file paths open in a new tab.
- `save_to_disk`: HTTP(S) downloads via `ureq` after a save dialog (or uses WebKit destination when provided); `file://` copies into the chosen path.

## Platform wiring

### Linux (WebKitGTK via Tauri `WebviewBuilder::on_download`)

**Implemented** in `crates/unibrowser-desktop/src-tauri/src/webview.rs`:

| `DownloadEvent` | Action |
| --- | --- |
| `Requested` | Emit `unibrowser-download` with URL + suggested destination path; return `false` to cancel WebKit’s default save when the shell handles disposition |
| `Finished` (success, path present) | Emit `unibrowser-download` with `file://` URL and on-disk path so **save-to-disk** preference can open or archive without a second fetch |

Verify `Requested` / `Finished` for your target sites; behaviour depends on WebKitGTK version and response headers.

### Windows (WebView2)

**Not wired** in-tree. Intended hook:

1. Register `ICoreWebView2DownloadStarting` on the content WebView2 instance.
2. Call `apply_download_disposition` with the download URI and user preference.
3. For `OpenInNewTab`, cancel the default download and invoke the same path as `handle_download_url`.
4. For `SaveToDisk`, allow the download or stream to the user-chosen path.

Until that delegate exists, use `handle_download_url`, the demo download button, or emit `unibrowser-download` from a future native layer.

### macOS

Non-target for this blitz.

## Security

`apply_download_disposition` rejects `javascript:` and `data:` URLs and applies `path_security` for file paths (no `..` traversal).

## Tests

- Unit tests in `downloads.rs` (schemes, preference branches).
- Integration tests in `src-tauri/tests/download_open_tab.rs`.
