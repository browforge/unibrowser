//! Download → new tab disposition (ADR 0009).
//!
//! **Hook points (platform):**
//! - **Windows (WebView2):** `DownloadStarting` on `ICoreWebView2` — not yet wired; call
//!   `apply_download_disposition` from the handler when added.
//! - **Linux (WebKitGTK):** `webkit_web_context_download_started` — not yet wired.
//!
//! Until those delegates exist, the shell exposes `handle_download_url` and `simulate_download`.

use std::path::{Path, PathBuf};

use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

use crate::path_security::resolve_open_path;

/// Resolve a download target to **open in new tab** disposition (ADR 0009).
pub fn apply_download_disposition(
    url: &str,
    suggested_path: Option<PathBuf>,
    cwd: &Path,
) -> Result<DownloadDisposition, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("empty download url".into());
    }
    if trimmed.starts_with("javascript:") || trimmed.starts_with("data:") {
        return Err("unsupported download scheme".into());
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::Http {
                url: trimmed.to_string(),
            },
        });
    }
    if trimmed.starts_with("file://") {
        let path = trimmed.trim_start_matches("file://");
        let resolved = resolve_open_path(path, cwd)?;
        return Ok(DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::File { path: resolved },
        });
    }
    if let Some(path) = suggested_path {
        let resolved = resolve_open_path(&path.to_string_lossy(), cwd)?;
        return Ok(DownloadDisposition::OpenInNewTab {
            suggested_location: BrowserLocation::File { path: resolved },
        });
    }
    let resolved = resolve_open_path(trimmed, cwd)?;
    Ok(DownloadDisposition::OpenInNewTab {
        suggested_location: BrowserLocation::File { path: resolved },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_download_opens_tab() {
        let d = apply_download_disposition("https://example.com/a.pdf", None, Path::new("/tmp"))
            .expect("disposition");
        assert!(matches!(
            d,
            DownloadDisposition::OpenInNewTab {
                suggested_location: BrowserLocation::Http { .. }
            }
        ));
    }

    #[test]
    fn rejects_traversal_in_file_download() {
        let err = apply_download_disposition("../etc/passwd", None, Path::new("/tmp")).unwrap_err();
        assert!(err.contains(".."));
    }

    #[test]
    fn rejects_javascript_scheme() {
        assert!(apply_download_disposition("javascript:alert(1)", None, Path::new("/tmp")).is_err());
    }
}
