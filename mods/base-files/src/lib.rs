use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use unibrowser_engine::CommandRegistry;

pub const BACKEND_ID: &str = "base-files";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub modified_unix: i64,
    #[serde(default)]
    pub is_image: bool,
    #[serde(default)]
    pub is_archive: bool,
}

fn is_image_extension(path: &Path) -> bool {
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

fn is_archive_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(ext.as_str(), "zip" | "unibrow")
}

pub fn list_directory(path: &Path) -> Result<Vec<DirEntry>, String> {
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let modified_unix = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let name = entry.file_name().to_string_lossy().into_owned();
        entries.push(DirEntry {
            name: name.clone(),
            path: entry.path(),
            is_dir: meta.is_dir(),
            size_bytes: if meta.is_dir() { 0 } else { meta.len() },
            modified_unix,
            is_image: !meta.is_dir() && is_image_extension(&entry.path()),
            is_archive: !meta.is_dir() && is_archive_extension(&entry.path()),
        });
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

pub fn handles_directory(path: &Path) -> bool {
    path.is_dir()
}

pub fn register(_registry: &mut CommandRegistry) {
    // Files mod contributes navigation only in Phase 1; commands come from format mods.
}
