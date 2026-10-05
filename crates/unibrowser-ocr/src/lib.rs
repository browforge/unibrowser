//! OCR via **tesseract** on PATH (no bundled engine).

use std::path::Path;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("tesseract not found on PATH")]
    NotFound,
    #[error("tesseract failed: {0}")]
    Failed(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

pub fn tesseract_available() -> bool {
    find_tesseract().is_some()
}

pub fn find_tesseract() -> Option<String> {
    if let Ok(p) = std::env::var("UNIBROWSER_TESSERACT_PATH") {
        if Path::new(&p).is_file() {
            return Some(p);
        }
    }
    which_on_path("tesseract")
}

fn which_on_path(name: &str) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{}.exe", name));
            if exe.is_file() {
                return Some(exe.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Run OCR on a raster image path (PNG/JPEG/TIFF, etc.).
pub fn ocr_image_file(path: &Path) -> Result<String, OcrError> {
    let tesseract = find_tesseract().ok_or(OcrError::NotFound)?;
    let output = Command::new(&tesseract)
        .arg(path)
        .arg("stdout")
        .arg("-l")
        .arg("eng")
        .output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(OcrError::Failed(err.trim().into()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_smoke_when_tesseract_present() {
        if !tesseract_available() {
            eprintln!("skip ocr_smoke_when_tesseract_present: tesseract not on PATH");
            return;
        }
        let dir = std::env::temp_dir().join("unibrowser-ocr-smoke");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Minimal 1x1 PNG
        let png: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9c, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let path = dir.join("tiny.png");
        std::fs::write(&path, png).unwrap();
        let _text = ocr_image_file(&path).expect("ocr should run");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
