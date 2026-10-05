//! Zip export and import for `.unibrow` packages (directory bundle or single zip file).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{ZipWriter, write::FileOptions};

use serde::{Deserialize, Serialize};

use crate::{ProjectError, PROJECT_FILE};

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
}

fn zip_has_root_manifest(path: &Path) -> Result<bool, ProjectError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?;
        let norm = entry.name().replace('\\', "/");
        if norm == PROJECT_FILE || norm.ends_with(&format!("/{}", PROJECT_FILE)) {
            return Ok(true);
        }
    }
    Ok(false)
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
    zip_has_root_manifest(path).unwrap_or(false)
}

pub fn export_directory_to_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    if !root.is_dir() || !root.join(PROJECT_FILE).is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    if let Some(parent) = dest_zip.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(dest_zip)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<'_, ()> = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    walk_export(root, root, &mut zip, options)?;
    zip.finish().map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

fn walk_export(
    root: &Path,
    current: &Path,
    zip: &mut ZipWriter<File>,
    options: FileOptions<'_, ()>,
) -> Result<(), ProjectError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|_| ProjectError::Msg("strip_prefix failed".into()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            if !rel_str.is_empty() {
                zip.add_directory(format!("{}/", rel_str), options)
                    .map_err(|e| ProjectError::Msg(e.to_string()))?;
            }
            walk_export(root, &path, zip, options)?;
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
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    fs::create_dir_all(&dest)?;
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| ProjectError::Msg(e.to_string()))?;
        let name = entry.name().replace('\\', "/");
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
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    Ok(dest)
}
