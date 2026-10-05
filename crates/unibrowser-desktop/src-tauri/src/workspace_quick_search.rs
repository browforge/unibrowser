//! Fuzzy quick-open file search under the pinned workspace root.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use unibrowser_engine::palette::fuzzy_score;

use crate::workspace_tree::{DEFAULT_MAX_DEPTH, DEFAULT_MAX_ENTRIES};

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceQuickHitDto {
    pub path: String,
    pub name: String,
    pub score: i32,
}

pub fn fuzzy_find_workspace_files(
    root: &Path,
    query: &str,
    max_results: usize,
) -> Result<Vec<WorkspaceQuickHitDto>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        return Err("workspace root is not a directory".into());
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("workspace root invalid: {}", e))?;
    let cap = max_results.min(DEFAULT_MAX_ENTRIES);
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(&root, &root, 0, DEFAULT_MAX_DEPTH, cap, &mut files)?;
    let mut hits: Vec<WorkspaceQuickHitDto> = files
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().to_string();
            let score = fuzzy_score(q, &name);
            if score < 0 {
                return None;
            }
            Some(WorkspaceQuickHitDto {
                path: path.to_string_lossy().into_owned(),
                name,
                score,
            })
        })
        .collect();
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    hits.truncate(max_results);
    Ok(hits)
}

fn collect_files(
    root: &Path,
    dir: &Path,
    depth: u32,
    max_depth: u32,
    cap: usize,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if out.len() >= cap {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        if out.len() >= cap {
            return Ok(());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.starts_with(root) {
            continue;
        }
        let name = entry.file_name();
        if name == ".git" || name == "node_modules" || name == "target" {
            continue;
        }
        if path.is_file() {
            out.push(path);
        } else if path.is_dir() && depth < max_depth {
            collect_files(root, &path, depth + 1, max_depth, cap, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matches_basename() {
        let dir = std::env::temp_dir().join(format!("ub-ws-quick-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.ts"), b"x").unwrap();
        fs::write(dir.join("README.md"), b"y").unwrap();
        let hits = fuzzy_find_workspace_files(&dir, "main", 10).expect("search");
        assert!(hits.iter().any(|h| h.name == "main.ts"));
        let _ = fs::remove_dir_all(&dir);
    }
}
