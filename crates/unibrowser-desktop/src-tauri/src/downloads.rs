//! Download → new tab disposition (ADR 0009).
//!
//! **Hook points (platform):**
//! - **Windows (WebView2):** `DownloadStarting` on `ICoreWebView2` — not yet wired; call
//!   `apply_download_disposition` from the handler when added.
//! - **Linux (WebKitGTK):** `webkit_web_context_download_started` — not yet wired.
//!
//! Until those delegates exist, the shell exposes `handle_download_url` and `simulate_download`.

use std::path::PathBuf;

use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

pub fn apply_download_disposition(url: &str, suggested_path: Option<PathBuf>) -> DownloadDisposition {
    if url.starts_with("http://") || url.starts_with("https://") {
        return DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::Http {
                url: url.to_string(),
            },
        };
    }
    if let Some(path) = suggested_path {
        return DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::File { path },
        };
    }
    DownloadDisposition::OpenInNewTab {
        suggested_location: BrowserLocation::File {
            path: PathBuf::from(url),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_download_opens_tab() {
        let d = apply_download_disposition("https://example.com/a.pdf", None);
        assert!(matches!(
            d,
            DownloadDisposition::OpenInNewTab {
                suggested_location: BrowserLocation::Http { .. }
            }
        ));
    }
}
