//! Directory reads with cooperative cancel between entries.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use base_files::DirEntry;

pub fn list_dir_cancellable(path: &Path, cancel: &AtomicBool) -> Result<Vec<DirEntry>, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("listing cancelled".into());
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        if cancel.load(Ordering::Relaxed) {
            return Err("listing cancelled".into());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let modified_unix = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let name = entry.file_name().to_string_lossy().into_owned();
        let path_buf = entry.path();
        let is_dir = meta.is_dir();
        entries.push(DirEntry {
            name,
            path: path_buf.clone(),
            is_dir,
            size_bytes: if is_dir { 0 } else { meta.len() },
            modified_unix,
            is_image: !is_dir && is_image_ext(&path_buf),
            is_archive: !is_dir && is_archive_ext(&path_buf),
        });
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

fn is_image_ext(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
    )
}

fn is_archive_ext(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(ext.as_str(), "zip" | "unibrow")
}
