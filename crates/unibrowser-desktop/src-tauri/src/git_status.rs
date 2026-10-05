//! Optional git dirty hints for the Files manager (no full SCM UI).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.canonicalize().ok()?;
    loop {
        if cur.join(".git").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Paths (absolute) that differ from HEAD according to `git status --porcelain`.
pub fn dirty_paths(repo_root: &Path) -> Result<HashSet<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("git not available: {}", e))?;
    if !output.status.success() {
        return Err("git status failed".into());
    }
    let mut dirty = HashSet::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.len() < 4 {
            continue;
        }
        let path_part = line[3..].trim();
        if path_part.is_empty() {
            continue;
        }
        let rel = if let Some((_from, to)) = path_part.split_once(" -> ") {
            to
        } else {
            path_part
        };
        let abs = repo_root.join(rel);
        if let Ok(canonical) = abs.canonicalize() {
            dirty.insert(canonical.to_string_lossy().into_owned());
        } else {
            dirty.insert(abs.to_string_lossy().into_owned());
        }
    }
    Ok(dirty)
}

pub fn status_for_folder(folder: &Path) -> (bool, HashSet<String>) {
    let root = find_git_root(folder);
    match root {
        None => (false, HashSet::new()),
        Some(repo) => match dirty_paths(&repo) {
            Ok(set) => (true, set),
            Err(_) => (true, HashSet::new()),
        },
    }
}
