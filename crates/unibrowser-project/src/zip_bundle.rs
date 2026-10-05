//! Zip export and import for `.unibrow` packages (directory bundle or single zip file).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{ZipWriter, write::FileOptions};

use crate::{ProjectError, PROJECT_FILE};

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
