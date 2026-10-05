//! Capped recursive listing for workspace folder trees in the Files manager.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use base_files::list_directory;

pub const DEFAULT_MAX_ENTRIES: usize = 1200;
pub const DEFAULT_MAX_DEPTH: u32 = 10;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceTreeEntryDto {
    pub path: String,
    pub name: String,
    pub depth: u32,
    pub is_dir: bool,
}

pub fn list_workspace_tree(
    root: &Path,
    max_entries: usize,
    max_depth: u32,
) -> Result<Vec<WorkspaceTreeEntryDto>, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("workspace root invalid: {}", e))?;
    if !root.is_dir() {
        return Err("workspace root is not a directory".into());
    }
    let cap = max_entries.clamp(50, 5000);
    let depth_cap = max_depth.clamp(1, 24);
    let mut out = Vec::new();
    out.push(WorkspaceTreeEntryDto {
        path: root.to_string_lossy().into_owned(),
        name: root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.to_string_lossy().into_owned()),
        depth: 0,
        is_dir: true,
    });
    let mut queue: VecDeque<(PathBuf, u32)> = VecDeque::new();
    queue.push_back((root.clone(), 0));
    while let Some((dir, depth)) = queue.pop_front() {
        if out.len() >= cap {
            break;
        }
        if depth >= depth_cap {
            continue;
        }
        let entries = list_directory(&dir).map_err(|e| e.to_string())?;
        for entry in entries {
            if out.len() >= cap {
                break;
            }
            let child = PathBuf::from(&entry.path);
            let is_dir = entry.is_dir;
            out.push(WorkspaceTreeEntryDto {
                path: entry.path.to_string_lossy().into_owned(),
                name: entry.name.clone(),
                depth: depth + 1,
                is_dir,
            });
            if is_dir {
                queue.push_back((child, depth + 1));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn caps_workspace_walk() {
        let dir = std::env::temp_dir().join("unibrow-workspace-tree-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("a/b")).expect("mkdir");
        fs::write(dir.join("a/b/file.txt"), b"x").expect("write");
        let entries = list_workspace_tree(&dir, 50, 5).expect("walk");
        assert!(entries.len() >= 3);
        let _ = fs::remove_dir_all(&dir);
    }
}
