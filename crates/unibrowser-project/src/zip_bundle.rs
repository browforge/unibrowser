//! Zip export and import for `.unibrow` packages (directory bundle or single zip file).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{ZipWriter, write::FileOptions};

use serde::{Deserialize, Serialize};

use crate::{opc_bundle, ProjectError, PROJECT_FILE, DIR_DERIVED};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZipEntrySummary {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

/// List entries inside a zip file (`.zip` or `.unibrow` package) without extracting.
pub fn list_zip_entries(path: &Path, max_entries: usize) -> Result<Vec<ZipEntrySummary>, ProjectError> {
    if !path.is_file() {
        return Err(ProjectError::Msg("not a file".into()));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "unibrow" && !is_unibrow_zip_file(path) {
        return Err(ProjectError::Msg("not a zip archive".into()));
    }
    if ext != "zip" && ext != "unibrow" {
        return Err(ProjectError::Msg("not a zip archive".into()));
    }
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let cap = max_entries.min(archive.len());
    let mut out = Vec::with_capacity(cap);
    for i in 0..cap {
        let entry = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?;
        let name = entry.name().replace('\\', "/");
        let is_dir = entry.is_dir() || name.ends_with('/');
        out.push(ZipEntrySummary {
            path: name,
            is_dir,
            size: entry.size(),
        });
    }
    Ok(out)
}

fn sanitize_zip_entry_path(raw: &str) -> Result<PathBuf, ProjectError> {
    let norm = raw.replace('\\', "/");
    if norm.is_empty() {
        return Err(ProjectError::Msg("empty zip entry".into()));
    }
    if norm.starts_with('/') || norm.contains("..") {
        return Err(ProjectError::Msg("invalid zip entry path".into()));
    }
    let path = PathBuf::from(norm);
    for comp in path.components() {
        if comp == std::path::Component::ParentDir {
            return Err(ProjectError::Msg("invalid zip entry path".into()));
        }
    }
    Ok(path)
}

/// Extract one file from a zip archive into `dest_root/<archive-stem>/`.
pub fn extract_zip_entry(
    archive_path: &Path,
    entry_path: &str,
    dest_root: &Path,
) -> Result<PathBuf, ProjectError> {
    if entry_path.ends_with('/') {
        return Err(ProjectError::Msg("cannot open a directory entry".into()));
    }
    let rel = sanitize_zip_entry_path(entry_path)?;
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "unibrow" && !is_unibrow_zip_file(archive_path) {
        return Err(ProjectError::Msg("not a zip archive".into()));
    }
    if ext != "zip" && ext != "unibrow" {
        return Err(ProjectError::Msg("not a zip archive".into()));
    }
    let stem = archive_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "archive".into());
    let out_path = dest_root.join(stem).join(&rel);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    let zip_name = entry_path.replace('\\', "/");
    let mut entry = archive
        .by_name(&zip_name)
        .map_err(|e| ProjectError::Msg(e.to_string()))?;
    if entry.is_dir() {
        return Err(ProjectError::Msg("cannot open a directory entry".into()));
    }
    let mut out = File::create(&out_path)?;
    std::io::copy(&mut entry, &mut out)?;
    Ok(out_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extract_zip_entry_writes_file() {
        let dir = std::env::temp_dir().join(format!("ub-zip-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let zip_path = dir.join("sample.zip");
        {
            let file = File::create(&zip_path).expect("create zip");
            let mut zip = ZipWriter::new(file);
            let options: FileOptions<'_, ()> =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("hello.txt", options)
                .expect("start file");
            zip.write_all(b"peek").expect("write");
            zip.finish().expect("finish");
        }
        let dest = dir.join("out");
        let out =
            extract_zip_entry(&zip_path, "hello.txt", &dest).expect("extract");
        assert!(out.is_file());
        let text = fs::read_to_string(&out).expect("read");
        assert_eq!(text, "peek");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sanitize_rejects_traversal() {
        assert!(sanitize_zip_entry_path("../x").is_err());
    }

    #[test]
    fn share_export_skips_derived() {
        let dir = std::env::temp_dir().join(format!("ub-share-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("assets")).expect("assets");
        fs::write(dir.join(crate::PROJECT_FILE), b"{\"format_version\":1,\"title\":\"S\",\"nodes\":[],\"timeline\":{\"clips\":[],\"transitions\":[],\"markers\":[]}}").expect("manifest");
        fs::create_dir_all(dir.join("derived")).expect("derived");
        fs::write(dir.join("derived/proxy.txt"), b"x").expect("proxy");
        let zip_path = dir.with_extension("share.zip");
        export_share_package_to_zip(&dir, &zip_path).expect("export");
        let listed = list_zip_entries(&zip_path, 50).expect("list");
        assert!(listed.iter().any(|e| e.path.contains("project.json")));
        assert!(!listed.iter().any(|e| e.path.contains("derived")));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(&zip_path);
    }
}

pub fn is_unibrow_zip_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !ext.eq_ignore_ascii_case("unibrow") {
        return false;
    }
    opc_bundle::is_opc_unibrow_zip(path) || opc_bundle::is_legacy_unibrow_zip(path)
}

pub fn export_directory_to_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    export_directory_to_zip_filtered(root, dest_zip, false)
}

/// FOSS share bundle: `assets/`, `refs/`, and `project.json` without local `derived/` proxies.
pub fn export_share_package_to_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    export_directory_to_zip_filtered(root, dest_zip, true)
}

fn export_directory_to_zip_filtered(
    root: &Path,
    dest_zip: &Path,
    skip_derived: bool,
) -> Result<(), ProjectError> {
    opc_bundle::write_opc_zip_from_root(root, dest_zip, skip_derived)
}

/// Import a share zip into `parent_dir/{name}.unibrow/` (directory bundle).
pub fn import_share_zip_into_parent(
    zip_path: &Path,
    parent_dir: &Path,
    folder_name: &str,
) -> Result<PathBuf, ProjectError> {
    if !zip_path.is_file() {
        return Err(ProjectError::Msg("zip missing".into()));
    }
    let ext = zip_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "zip" && ext != "unibrow" {
        return Err(ProjectError::Msg("not a zip share package".into()));
    }
    let dest = parent_dir.join(format!("{}.unibrow", folder_name));
    if dest.exists() {
        return Err(ProjectError::Msg(format!("already exists: {}", dest.display())));
    }
    fs::create_dir_all(&dest)?;
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?;
        let name = entry.name().replace('\\', "/");
        if name.contains("..") {
            return Err(ProjectError::Msg("invalid zip entry".into()));
        }
        if name.ends_with('/') {
            fs::create_dir_all(dest.join(&name))?;
            continue;
        }
        let out_path = dest.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    if !dest.join(PROJECT_FILE).is_file() {
        let _ = fs::remove_dir_all(&dest);
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    if let Ok(data) = fs::read_to_string(dest.join(PROJECT_FILE)) {
        if let Ok(mut manifest) = serde_json::from_str::<crate::ProjectManifest>(&data) {
            let _ = opc_bundle::ensure_unpacked_package_layout(&dest, &mut manifest);
        }
    }
    Ok(dest)
}

fn walk_export(
    root: &Path,
    current: &Path,
    zip: &mut ZipWriter<File>,
    options: FileOptions<'_, ()>,
    skip_derived: bool,
) -> Result<(), ProjectError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|_| ProjectError::Msg("strip_prefix failed".into()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if skip_derived && (rel_str == DIR_DERIVED || rel_str.starts_with(&format!("{}/", DIR_DERIVED))) {
            continue;
        }
        if path.is_dir() {
            if !rel_str.is_empty() {
                zip.add_directory(format!("{}/", rel_str), options)
                    .map_err(|e| ProjectError::Msg(e.to_string()))?;
            }
            walk_export(root, &path, zip, options, skip_derived)?;
        } else if path.is_file() {
            zip.start_file(rel_str, options)
                .map_err(|e| ProjectError::Msg(e.to_string()))?;
            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf).map_err(|e| ProjectError::Msg(e.to_string()))?;
        }
    }
    Ok(())
}

/// Extract a `.unibrow` zip into `cache_dir/<stem>/` and return that directory.
pub fn materialize_zip_package(zip_path: &Path, cache_dir: &Path) -> Result<PathBuf, ProjectError> {
    if !is_unibrow_zip_file(zip_path) {
        return Err(ProjectError::Msg("not a .unibrow zip package".into()));
    }
    fs::create_dir_all(cache_dir)?;
    let stem = zip_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "package".into());
    let dest = cache_dir.join(stem);
    opc_bundle::materialize_zip_to_dir(zip_path, &dest)?;
    if !dest.join(PROJECT_FILE).is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    Ok(dest)
}
