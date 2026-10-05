//! Non-destructive PDF highlight sidecar (`{file}.ub-annotations.json`).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfHighlightDto {
    pub page: u32,
    /// Normalised 0–1 relative to page viewport at creation time.
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "rgba(255, 230, 0, 0.35)".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SidecarFile {
    version: u32,
    pdf_path: String,
    highlights: Vec<PdfHighlightDto>,
}

impl Default for SidecarFile {
    fn default() -> Self {
        Self {
            version: 1,
            pdf_path: String::new(),
            highlights: Vec::new(),
        }
    }
}

pub fn sidecar_path(pdf: &Path) -> PathBuf {
    let name = pdf
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "document.pdf".into());
    pdf.with_file_name(format!("{}.ub-annotations.json", name))
}

fn load_sidecar(pdf: &Path) -> Result<SidecarFile, String> {
    let path = sidecar_path(pdf);
    if !path.is_file() {
        return Ok(SidecarFile {
            version: 1,
            pdf_path: pdf.to_string_lossy().into_owned(),
            highlights: Vec::new(),
        });
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file: SidecarFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(file)
}

fn save_sidecar(pdf: &Path, file: &SidecarFile) -> Result<(), String> {
    let path = sidecar_path(pdf);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_highlights(pdf: &Path) -> Result<Vec<PdfHighlightDto>, String> {
    Ok(load_sidecar(pdf)?.highlights)
}

pub fn add_highlight(pdf: &Path, highlight: PdfHighlightDto) -> Result<Vec<PdfHighlightDto>, String> {
    let mut file = load_sidecar(pdf)?;
    file.pdf_path = pdf.to_string_lossy().into_owned();
    file.highlights.push(highlight);
    save_sidecar(pdf, &file)?;
    Ok(file.highlights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn roundtrip_sidecar() {
        let dir = std::env::temp_dir().join("ub-pdf-ann-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = dir.join("sample.pdf");
        let mut f = std::fs::File::create(&pdf).unwrap();
        f.write_all(b"%PDF-1.4 stub").unwrap();
        let sc = sidecar_path(&pdf);
        assert!(sc.to_string_lossy().contains("sample.pdf.ub-annotations.json"));
        add_highlight(
            &pdf,
            PdfHighlightDto {
                page: 1,
                x: 0.1,
                y: 0.2,
                w: 0.3,
                h: 0.1,
                color: default_color(),
            },
        )
        .unwrap();
        let list = list_highlights(&pdf).unwrap();
        assert_eq!(list.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
