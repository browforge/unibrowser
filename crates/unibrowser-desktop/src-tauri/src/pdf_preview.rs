//! Preview raster for PDF pages (proxy only; source PDF stays in assets per ADR 0011).

use std::path::{Path, PathBuf};
use std::process::Command;

use unibrowser_media::find_binary;

pub fn pdf_first_page_preview(path: &Path) -> Result<PathBuf, String> {
    pdf_page_preview(path, 1)
}

pub fn pdf_page_preview(path: &Path, page: u32) -> Result<PathBuf, String> {
    if !path.is_file() {
        return Err("not a file".into());
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "pdf" {
        return Err("not a pdf".into());
    }
    let page = page.max(1);
    let cache = std::env::temp_dir().join("unibrowser-pdf-preview");
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "page".into());
    let len = path.metadata().map(|m| m.len()).unwrap_or(0);
    let out = cache.join(format!("{}-{}-p{}.jpg", stem, len, page));
    if out.is_file() {
        return Ok(out);
    }

    if let Some(pdftoppm) = find_binary("pdftoppm") {
        let prefix = cache.join(format!("{}-{}-p{}", stem, len, page));
        let status = Command::new(&pdftoppm)
            .args([
                "-f",
                &page.to_string(),
                "-l",
                &page.to_string(),
                "-jpeg",
                "-r",
                "120",
                "-singlefile",
                path.to_string_lossy().as_ref(),
                prefix.to_string_lossy().as_ref(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        let jpg = prefix.with_extension("jpg");
        if status.success() && jpg.is_file() {
            return Ok(jpg);
        }
    }

    if page == 1 {
        if let Some(ffmpeg) = find_binary("ffmpeg") {
            let status = Command::new(&ffmpeg)
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-i",
                    path.to_string_lossy().as_ref(),
                    "-frames:v",
                    "1",
                    "-q:v",
                    "4",
                    "-y",
                    out.to_string_lossy().as_ref(),
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() && out.is_file() {
                return Ok(out);
            }
        }
    }

    Err("pdf preview needs pdftoppm or ffmpeg on PATH (or use pdf.js in the UI)".into())
}
