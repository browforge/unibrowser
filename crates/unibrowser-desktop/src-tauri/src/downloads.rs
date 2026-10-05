//! Download → tab or disk disposition (ADR 0009).
//!
//! **Hook points (platform):**
//! - **Linux (WebKitGTK / Tauri content webview):** `WebviewBuilder::on_download` in `webview.rs`
//!   handles `DownloadEvent::Requested` and emits `unibrowser-download` (see `docs/downloads.md`).
//! - **Windows (WebView2):** `ICoreWebView2::DownloadStarting` is **not** wired in this repo yet;
//!   use `handle_download_url` / demo button until the native delegate calls `apply_download_disposition`.
//!
//! Until platform delegates pass a suggested path, the shell exposes `handle_download_url` and events.

use std::path::{Path, PathBuf};

use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

use crate::path_security::resolve_open_path;
use crate::preferences::DownloadPreference;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadEventPayload {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
}

pub fn suggested_download_filename(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url.trim()) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.filter(|s| !s.is_empty()).last() {
                return last.to_string();
            }
        }
    }
    "download".into()
}

pub fn default_download_path(cwd: &Path, url: &str) -> PathBuf {
    let name = suggested_download_filename(url);
    let home = std::env::var("HOME").unwrap_or_else(|_| cwd.to_string_lossy().into_owned());
    PathBuf::from(home).join("Downloads").join(name)
}

/// Resolve a download target disposition from URL, preference, and optional platform hint path.
pub fn apply_download_disposition(
    url: &str,
    suggested_path: Option<PathBuf>,
    cwd: &Path,
    preference: DownloadPreference,
) -> Result<DownloadDisposition, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("empty download url".into());
    }
    if trimmed.starts_with("javascript:") || trimmed.starts_with("data:") {
        return Err("unsupported download scheme".into());
    }

    if preference == DownloadPreference::SaveToDisk {
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            let dest = suggested_path
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| default_download_path(cwd, trimmed));
            return Ok(DownloadDisposition::SaveToDisk { path: dest });
        }
        if trimmed.starts_with("file://") {
            let path = trimmed.trim_start_matches("file://");
            let _resolved = resolve_open_path(path, cwd)?;
            let dest = suggested_path
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| default_download_path(cwd, trimmed));
            return Ok(DownloadDisposition::SaveToDisk { path: dest });
        }
        if let Some(path) = suggested_path {
            return Ok(DownloadDisposition::SaveToDisk {
                path: path.to_path_buf(),
            });
        }
        let _resolved = resolve_open_path(trimmed, cwd)?;
        let dest = default_download_path(cwd, trimmed);
        return Ok(DownloadDisposition::SaveToDisk { path: dest });
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

pub fn download_http_to_file(url: &str, dest: &Path) -> Result<(), String> {
    let trimmed = url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("not an http(s) url".into());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let response = ureq::get(trimmed)
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn copy_file_to_path(source: &Path, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(source, dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_download_opens_tab_by_default() {
        let d = apply_download_disposition(
            "https://example.com/a.pdf",
            None,
            Path::new("/tmp"),
            DownloadPreference::OpenInNewTab,
        )
        .expect("disposition");
        assert!(matches!(
            d,
            DownloadDisposition::OpenInNewTab {
                suggested_location: BrowserLocation::Http { .. }
            }
        ));
    }

    #[test]
    fn http_download_save_pref() {
        let d = apply_download_disposition(
            "https://example.com/a.pdf",
            None,
            Path::new("/tmp"),
            DownloadPreference::SaveToDisk,
        )
        .expect("disposition");
        assert!(matches!(d, DownloadDisposition::SaveToDisk { .. }));
    }

    #[test]
    fn rejects_traversal_in_file_download() {
        let err = apply_download_disposition(
            "../etc/passwd",
            None,
            Path::new("/tmp"),
            DownloadPreference::OpenInNewTab,
        )
        .unwrap_err();
        assert!(err.contains(".."));
    }

    #[test]
    fn rejects_javascript_scheme() {
        assert!(apply_download_disposition(
            "javascript:alert(1)",
            None,
            Path::new("/tmp"),
            DownloadPreference::OpenInNewTab,
        )
        .is_err());
    }

    #[test]
    fn filename_from_url_path() {
        assert_eq!(
            suggested_download_filename("https://example.org/files/report.pdf"),
            "report.pdf"
        );
    }

    #[test]
    fn platform_hint_still_opens_http_tab_without_fetch_helper() {
        let dest = PathBuf::from("/tmp/Downloads/report.pdf");
        let d = apply_download_disposition(
            "https://example.org/report.pdf",
            Some(dest),
            Path::new("/tmp"),
            DownloadPreference::OpenInNewTab,
        )
        .expect("disposition");
        assert!(matches!(
            d,
            DownloadDisposition::OpenInNewTab {
                suggested_location: BrowserLocation::Http { .. }
            }
        ));
    }
}
