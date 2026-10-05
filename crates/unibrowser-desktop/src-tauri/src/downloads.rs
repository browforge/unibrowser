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

pub fn is_generic_download_filename(name: &str) -> bool {
    let n = name.trim();
    n.is_empty()
        || n.eq_ignore_ascii_case("download")
        || n.eq_ignore_ascii_case("download.bin")
        || n.starts_with('.')
        || n.ends_with(".crdownload")
        || n.ends_with(".part")
}

/// Parse `Content-Disposition` for a filename (RFC 5987 `filename*` or quoted `filename=`).
pub fn filename_from_content_disposition(header: &str) -> Option<String> {
    let h = header.trim();
    if let Some(star) = h
        .split(';')
        .map(str::trim)
        .find(|p| p.to_ascii_lowercase().starts_with("filename*="))
    {
        let val = star
            .split_once('=')
            .map(|(_, v)| v.trim())
            .unwrap_or("");
        let val = val.trim_matches('"');
        let name = if let Some((_enc, encoded)) = val.split_once("''") {
            percent_decode_str(encoded)
        } else {
            val.to_string()
        };
        let clean = sanitize_download_filename(&name);
        if !clean.is_empty() && clean != "download" {
            return Some(clean);
        }
    }
    for part in h.split(';').map(str::trim) {
        let lower = part.to_ascii_lowercase();
        if lower.starts_with("filename=") {
            let val = part
                .split_once('=')
                .map(|(_, v)| v.trim())
                .unwrap_or("");
            let unquoted = val.trim_matches('"').trim_matches('\'');
            let clean = sanitize_download_filename(unquoted);
            if !clean.is_empty() && clean != "download" {
                return Some(clean);
            }
        }
    }
    None
}

fn percent_decode_str(s: &str) -> String {
    let mut out: Vec<u8> = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn sanitize_download_filename(name: &str) -> String {
    let name = name.trim().replace(['\\', '/'], "_");
    Path::new(&name)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "download".into())
}

/// Prefer platform hint directory; replace generic leaf names with CD or URL filename.
pub fn merge_download_destination(
    hinted: &Path,
    url: &str,
    content_disposition: Option<&str>,
) -> PathBuf {
    let cd_name = content_disposition.and_then(filename_from_content_disposition);
    let leaf = hinted
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = hinted.parent().filter(|p| !p.as_os_str().is_empty());
    let pick_name = |fallback: &str| -> String {
        if let Some(n) = cd_name.as_ref() {
            if is_generic_download_filename(&leaf) {
                return n.clone();
            }
        }
        if is_generic_download_filename(&leaf) {
            return suggested_download_filename(url);
        }
        leaf.clone()
    };
    let name = pick_name(&leaf);
    if let Some(dir) = parent {
        dir.join(name)
    } else {
        PathBuf::from(name)
    }
}

pub fn download_http_to_file(url: &str, dest: &Path) -> Result<PathBuf, String> {
    let trimmed = url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("not an http(s) url".into());
    }
    let response = ureq::get(trimmed)
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let cd = response
        .header("content-disposition")
        .or_else(|| response.header("Content-Disposition"));
    let final_dest = merge_download_destination(dest, trimmed, cd);
    if let Some(parent) = final_dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(&final_dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    Ok(final_dest)
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
    fn content_disposition_filename() {
        assert_eq!(
            filename_from_content_disposition(r#"attachment; filename="report Q1.pdf""#),
            Some("report Q1.pdf".into())
        );
        assert_eq!(
            filename_from_content_disposition("attachment; filename*=UTF-8''caf%C3%A9.bin"),
            Some("café.bin".into())
        );
    }

    fn merge_uses_cd_when_hint_is_generic() {
        let dest = merge_download_destination(
            Path::new("/tmp/Downloads/download"),
            "https://example.org/foo",
            Some(r#"attachment; filename="real.zip""#),
        );
        assert_eq!(dest, PathBuf::from("/tmp/Downloads/real.zip"));
    }

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
