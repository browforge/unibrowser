//! Shared download event emission and platform notes ([docs/browser-shell.md](../../../docs/browser-shell.md)).

use tauri::{AppHandle, Emitter};

use crate::downloads::DownloadEventPayload;

/// Emit the same payload the Linux `on_download` hook uses so `main.ts` can disposition.
pub fn emit_unibrowser_download(app: &AppHandle, payload: DownloadEventPayload) {
    let _ = app.emit("unibrowser-download", payload);
}

/// Human-readable platform status for diagnostics / docs.
pub fn platform_download_hook_status() -> (&'static str, bool) {
    #[cfg(target_os = "linux")]
    {
        ("WebKitGTK on_download → unibrowser-download", true)
    }
    #[cfg(target_os = "windows")]
    {
        (
            "WebView2 ICoreWebView2_4::DownloadStarting → unibrowser-download (compiles; not verified on hardware)",
            true,
        )
    }
    #[cfg(target_os = "macos")]
    {
        ("WKWebView download delegate not wired", false)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        ("Embedded webview download hook unknown", false)
    }
}

