//! Filename search under a root with depth limit.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FilenameHitDto {
    pub path: String,
    pub name: String,
}

pub fn search_filenames(
    root: &Path,
    query: &str,
    max_depth: u32,
    max_results: usize,
    cancel: &AtomicBool,
) -> Result<Vec<FilenameHitDto>, String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        return Err("root is not a directory".into());
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("root path invalid: {}", e))?;
    let mut out = Vec::new();
    walk(
        &root,
        &root,
        &q,
        0,
        max_depth,
        max_results,
        cancel,
        &mut out,
    )?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn finds_file_by_name() {
        let dir = std::env::temp_dir().join(format!("ub-find-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("needle.txt"), b"x").unwrap();
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/other.md"), b"y").unwrap();
        let cancel = AtomicBool::new(false);
        let hits = search_filenames(&dir, "needle", 2, 10, &cancel).expect("search");
        assert!(hits.iter().any(|h| h.name == "needle.txt"));
        let _ = fs::remove_dir_all(&dir);
    }
}

fn walk(
    root: &Path,
    dir: &Path,
    query: &str,
    depth: u32,
    max_depth: u32,
    max_results: usize,
    cancel: &AtomicBool,
    out: &mut Vec<FilenameHitDto>,
) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("search cancelled".into());
    }
    if out.len() >= max_results {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        if cancel.load(Ordering::Relaxed) {
            return Err("search cancelled".into());
        }
        if out.len() >= max_results {
            return Ok(());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_lowercase().contains(query) {
            let path = entry.path();
            if path.starts_with(root) {
                out.push(FilenameHitDto {
                    path: path.to_string_lossy().into_owned(),
                    name,
                });
            }
        }
        if depth < max_depth {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, query, depth + 1, max_depth, max_results, cancel, out)?;
            }
        }
    }
    Ok(())
}
