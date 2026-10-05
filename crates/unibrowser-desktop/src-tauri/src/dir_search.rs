//! Filename and text-content search under a root with depth and size limits.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

/// Max depth passed from the shell (hard cap).
pub const HARD_MAX_DEPTH: u32 = 16;
/// Default max hits returned.
pub const DEFAULT_MAX_RESULTS: usize = 200;
/// Hard cap on hits.
pub const HARD_MAX_RESULTS: usize = 500;
/// Skip content scan above this size (bytes).
pub const MAX_TEXT_SCAN_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct FilenameHitDto {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub match_kind: String,
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
        max_depth.min(HARD_MAX_DEPTH),
        max_results.min(HARD_MAX_RESULTS),
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

    #[test]
    fn finds_text_content_line() {
        let dir = std::env::temp_dir().join(format!("ub-find-line-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("alpha.txt"), b"line one\nneedle here\n").unwrap();
        let cancel = AtomicBool::new(false);
        let hits = search_filenames(&dir, "needle", 2, 10, &cancel).expect("search");
        let hit = hits.iter().find(|h| h.path.ends_with("alpha.txt")).expect("hit");
        assert_eq!(hit.line, Some(2));
        assert_eq!(hit.match_kind, "content");
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
        let path = entry.path();
        if path.is_dir() {
            if depth < max_depth {
                walk(root, &path, query, depth + 1, max_depth, max_results, cancel, out)?;
            }
            continue;
        }
        if !path.starts_with(root) {
            continue;
        }
        let name_match = name.to_lowercase().contains(query);
        if name_match {
            let line = first_content_line(&path, query);
            out.push(FilenameHitDto {
                path: path.to_string_lossy().into_owned(),
                name,
                line,
                match_kind: if line.is_some() {
                    "filename+content".into()
                } else {
                    "filename".into()
                },
            });
            continue;
        }
        if is_text_search_candidate(&path) {
            if let Some(line) = first_content_line(&path, query) {
                out.push(FilenameHitDto {
                    path: path.to_string_lossy().into_owned(),
                    name,
                    line: Some(line),
                    match_kind: "content".into(),
                });
            }
        }
    }
    Ok(())
}

fn is_text_search_candidate(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if matches!(
        ext.as_str(),
        "txt" | "md" | "markdown" | "json" | "rs" | "toml" | "yaml" | "yml" | "html" | "htm"
            | "css" | "scss" | "xml" | "sh" | "bash" | "log" | "ini" | "cfg" | "conf" | "c"
            | "h" | "cpp" | "hpp" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "py" | "go" | "java"
            | "kt" | "sql" | "lua" | "csv" | "tsv"
    ) {
        return !is_probably_binary(path);
    }
    false
}

fn is_probably_binary(path: &Path) -> bool {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return true,
    };
    if meta.len() > MAX_TEXT_SCAN_BYTES {
        return true;
    }
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut buf = [0u8; 4096];
    let n = std::io::Read::read(&mut file, &mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}

fn first_content_line(path: &Path, query: &str) -> Option<u32> {
    if !is_text_search_candidate(path) && is_probably_binary(path) {
        return None;
    }
    let meta = fs::metadata(path).ok()?;
    if meta.len() > MAX_TEXT_SCAN_BYTES {
        return None;
    }
    if is_probably_binary(path) {
        return None;
    }
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let q = query.to_lowercase();
    for (idx, line) in reader.lines().map_while(Result::ok).enumerate() {
        if line.to_lowercase().contains(&q) {
            return Some((idx + 1) as u32);
        }
    }
    None
}
