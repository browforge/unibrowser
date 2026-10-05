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
}

pub fn list_directory(path: &Path) -> Result<Vec<DirEntry>, String> {
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        entries.push(DirEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path(),
            is_dir: meta.is_dir(),
        });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(entries)
}

pub fn handles_directory(path: &Path) -> bool {
    path.is_dir()
}

pub fn register(_registry: &mut CommandRegistry) {
    // Files mod contributes navigation only in Phase 1; commands come from format mods.
}
