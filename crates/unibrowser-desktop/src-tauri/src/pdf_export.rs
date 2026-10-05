//! Export printer-friendly HTML to PDF via a headless browser when available (Linux-first).

use std::path::{Path, PathBuf};
use std::process::Command;

const WRAPPER_HTML: &str = r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
body{font-family:Georgia,serif;font-size:11pt;line-height:1.5;margin:1.5rem;color:#111}
</style></head><body>"#;

pub fn export_html_to_pdf(html_body: &str, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = std::env::temp_dir().join(format!(
        "unibrowser-print-{}.html",
        std::process::id()
    ));
    let full = format!("{}{}</body></html>", WRAPPER_HTML, html_body);
    std::fs::write(&tmp, full).map_err(|e| e.to_string())?;

    let dest_str = dest.to_string_lossy();
    let tmp_str = tmp.to_string_lossy();

    let chromium_candidates = [
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
    ];
    for bin in chromium_candidates {
        if try_chromium_print(bin, &tmp_str, &dest_str) {
            let _ = std::fs::remove_file(&tmp);
            return Ok(());
        }
    }
    if try_wkhtmltopdf(&tmp_str, &dest_str) {
        let _ = std::fs::remove_file(&tmp);
        return Ok(());
    }
    let _ = std::fs::remove_file(&tmp);
    Err(
        "No PDF engine found. Install chromium or wkhtmltopdf, or use Print → Save as PDF in the browser window."
            .into(),
    )
}

fn try_chromium_print(bin: &str, input: &str, output: &str) -> bool {
    Command::new(bin)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--print-to-pdf",
            output,
            input,
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn try_wkhtmltopdf(input: &str, output: &str) -> bool {
    Command::new("wkhtmltopdf")
        .args(["--quiet", input, output])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn suggest_pdf_path(title: &str, cwd: &Path) -> PathBuf {
    let stem = title
        .split(|c: char| c == '/' || c == '\\')
        .last()
        .unwrap_or("document")
        .replace(' ', "-");
    let name = if stem.to_lowercase().ends_with(".pdf") {
        stem
    } else {
        format!("{}.pdf", stem.trim_end_matches(".md").trim_end_matches(".docx"))
    };
    cwd.join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_pdf_strips_md_suffix() {
        let p = suggest_pdf_path("/docs/readme.md", Path::new("/tmp"));
        assert_eq!(p, PathBuf::from("/tmp/readme.pdf"));
    }
}
