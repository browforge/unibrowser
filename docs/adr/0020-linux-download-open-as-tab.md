# ADR 0020: Linux web download → fetch → file tab

- **Status:** Accepted (shipped on blitz `main`)
- **Date:** 2026-10-05
- **Related:** [ADR 0009](0009-browser-as-unified-shell.md), [browser-shell.md](../browser-shell.md)

## Context

WebKitGTK reports downloads via Tauri `DownloadEvent::Requested` with a suggested filesystem path. Default user preference is **open in new tab**, but opening the raw `https://` URL is wrong for file downloads.

## Decision

1. When preference is **open in new tab** and the platform provides a **destination path**, the shell **fetches** with `ureq` to that path (honouring **Content-Disposition** when the leaf name is generic), then opens a **file** tab.
2. Ignore `DownloadEvent::Finished` emissions to avoid duplicate handling.
3. **Windows WebView2** remains a separate gap (`DownloadStarting` not wired); Settings system-browser hand-off unchanged.

## Consequences

- Linux https downloads align with ADR 0009 tab semantics for local files.
- Double fetch if both shell and webview attempted download — mitigated by returning `false` on `Requested` (custom handling).
