//! Folder browse mutations: copy, move, create, rename, duplicate.

use std::fs;
use std::path::{Path, PathBuf};

pub fn validate_entry_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("name cannot be empty".into());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("invalid name".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
        return Err("name cannot contain path separators".into());
    }
    Ok(trimmed.to_string())
}

pub fn duplicate_destination(parent: &Path, source: &Path) -> Result<PathBuf, String> {
    let file_name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or("missing file name")?;
    Ok(unique_sibling_path(parent, &file_name, " copy"))
}

fn unique_sibling_path(parent: &Path, file_name: &str, suffix: &str) -> PathBuf {
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.to_string());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let mut n = 0u32;
    loop {
        let candidate = if n == 0 {
            format!("{}{}{}", stem, suffix, ext)
        } else {
            format!("{}{} {}{}", stem, suffix, n, ext)
        };
        let dest = parent.join(&candidate);
        if !dest.exists() {
            return dest;
        }
        n += 1;
    }
}

pub fn guard_mutating(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("not found: {}", path.display()));
    }
    if let Some(cfg) = crate::config::config_dir() {
        let cfg = cfg.canonicalize().map_err(|e| e.to_string())?;
        let canon = path.canonicalize().map_err(|e| e.to_string())?;
        if canon == cfg {
            return Err("cannot modify application config directory".into());
        }
    }
    Ok(())
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|e| e.to_string())
}

pub fn is_same_or_descendant(ancestor: &Path, candidate: &Path) -> bool {
    let Ok(a) = canonical(ancestor) else {
        return false;
    };
    let Ok(c) = canonical(candidate) else {
        return false;
    };
    c == a || c.starts_with(&a)
}

pub fn copy_path_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        return Err(format!("already exists: {}", dest.display()));
    }
    if src.is_dir() {
        copy_dir_all(src, dest)?;
    } else if src.is_file() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(src, dest).map_err(|e| e.to_string())?;
    } else {
        return Err("not a file or directory".into());
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = dest.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn move_path(src: &Path, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        return Err(format!("already exists: {}", dest.display()));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    copy_path_recursive(src, dest)?;
    if src.is_dir() {
        fs::remove_dir_all(src).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(src).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn transfer_paths(
    sources: &[PathBuf],
    dest_dir: &Path,
    copy: bool,
) -> Result<(), String> {
    if !dest_dir.is_dir() {
        return Err("destination is not a directory".into());
    }
    let dest_canon = dest_dir.canonicalize().map_err(|e| e.to_string())?;
    for src in sources {
        guard_mutating(src)?;
        let name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or("missing source name")?;
        let target = dest_canon.join(&name);
        if is_same_or_descendant(src, &dest_canon) && !copy {
            return Err("cannot move a folder into itself".into());
        }
        if is_same_or_descendant(src, &target) {
            return Err("destination is inside the source".into());
        }
        if copy {
            copy_path_recursive(src, &target)?;
        } else {
            move_path(src, &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_names() {
        assert!(validate_entry_name("").is_err());
        assert!(validate_entry_name("../x").is_err());
        assert!(validate_entry_name("ok.txt").is_ok());
    }

    #[test]
    fn unique_sibling_adds_copy_suffix() {
        let dir = std::env::temp_dir().join(format!("ub-unique-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), b"x").unwrap();
        let p = unique_sibling_path(&dir, "a.txt", " copy");
        assert_eq!(p.file_name().unwrap().to_str(), Some("a copy.txt"));
        let _ = fs::remove_dir_all(&dir);
    }
}
