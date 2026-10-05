//! Path normalisation and traversal guards for file opens and project assets.

use std::path::{Component, Path, PathBuf};

use url::Url;

pub fn reject_traversal(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    if path
        .to_str()
        .map(|s| s.contains('\0'))
        .unwrap_or(true)
    {
        return Err("invalid path".into());
    }
    for component in path.components() {
        match component {
            Component::ParentDir => return Err("path must not contain ..".into()),
            Component::Normal(seg) => {
                let s = seg.to_string_lossy();
                if s == ".." || s.contains('\0') {
                    return Err("invalid path segment".into());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Resolve omnibox-style `file://` input (percent-decoding, then traversal guard).
pub fn resolve_file_url_input(input: &str, cwd: &Path) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    let rest = trimmed
        .strip_prefix("file://")
        .ok_or_else(|| "not a file URL".to_string())?;
    let decoded = percent_decode_lossy(rest);
    let probe = PathBuf::from(decoded.trim());
    reject_traversal(&probe)?;
    let path = if probe.is_absolute() {
        probe
    } else {
        let as_url = format!("file:///{}", decoded.trim_start_matches('/'));
        let parsed = Url::parse(&as_url).map_err(|e| e.to_string())?;
        parsed
            .to_file_path()
            .map_err(|_| "invalid file URL path".to_string())?
    };
    reject_traversal(&path)?;
    resolve_open_path(&path.to_string_lossy(), cwd)
}

fn percent_decode_lossy(input: &str) -> String {
    let mut out = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(pair) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(pair, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Resolve a user-supplied path against `cwd` (absolute paths allowed after traversal check).
pub fn resolve_open_path(raw: &str, cwd: &Path) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("empty path".into());
    }
    let path = PathBuf::from(trimmed);
    reject_traversal(&path)?;
    let full = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };
    reject_traversal(&full)?;
    if full.exists() {
        full.canonicalize().map_err(|e| e.to_string())
    } else {
        Ok(full)
    }
}

/// Asset file name inside a `.unibrow` package (`assets/<name>` only; no subpaths).
pub fn resolve_unibrow_asset_file(project_root: &Path, asset_name: &str) -> Result<PathBuf, String> {
    let name = asset_name.trim();
    if name.is_empty() {
        return Err("empty asset name".into());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("invalid asset name".into());
    }
    let project = project_root
        .canonicalize()
        .map_err(|e| format!("project path invalid: {}", e))?;
    let assets_dir = project.join("assets");
    if !assets_dir.is_dir() {
        return Err("project has no assets folder".into());
    }
    let assets_base = assets_dir
        .canonicalize()
        .map_err(|e| format!("assets folder invalid: {}", e))?;
    let candidate = assets_base.join(name);
    reject_traversal(&candidate)?;
    let canon = candidate
        .canonicalize()
        .map_err(|e| format!("asset not found: {}", e))?;
    if !canon.starts_with(&assets_base) {
        return Err("path escapes project assets".into());
    }
    Ok(canon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_parent_dir() {
        assert!(reject_traversal(Path::new("../etc/passwd")).is_err());
    }

    #[test]
    fn rejects_embedded_null() {
        assert!(reject_traversal(Path::new("a\0b")).is_err());
    }

    #[test]
    fn file_url_rejects_parent_dir() {
        let dir = std::env::temp_dir().join("ub-file-url-guard");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(resolve_file_url_input("file://../../../etc/passwd", &dir).is_err());
        assert!(resolve_file_url_input("file://%2e%2e%2f%2e%2e%2fetc%2fpasswd", &dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_under_temp() {
        let dir = std::env::temp_dir().join("ub-path-guard");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("note.txt");
        fs::write(&file, b"hi").unwrap();
        let got = resolve_open_path(file.to_str().unwrap(), &dir).unwrap();
        assert_eq!(got, file.canonicalize().unwrap());
        let _ = fs::remove_dir_all(&dir);
    }
}
