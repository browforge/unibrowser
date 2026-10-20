//! ADR 0032 storage modes: view-only / archive / folder open paths.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::read::ZipArchive;

use crate::opc_bundle::{materialize_zip_to_dir, materialize_zip_to_dir_cancellable};
use crate::template::{memory_root_for_session, MEMORY_ROOT_PREFIX};
use crate::{
    open_package, export_package_to_zip, validate_manifest, ProjectError, ProjectManifest,
    ProjectPackage, PROJECT_FILE,
};
use crate::zip_bundle;

pub const DEFAULT_FOLDER_HINT_BYTES: u64 = 500 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageStorageMode {
    ViewOnly,
    Archive,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnibrowOpenEstimate {
    pub archive_path: String,
    pub unpacked_bytes: u64,
    pub suggest_folder: bool,
}

pub fn estimate_archive_unpacked_bytes(zip_path: &Path) -> Result<u64, ProjectError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut total: u64 = 0;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?;
        if !entry.is_dir() {
            total += entry.size();
        }
    }
    Ok(total)
}

pub fn unibrow_open_estimate(zip_path: &Path, hint_threshold_bytes: u64) -> Result<UnibrowOpenEstimate, ProjectError> {
    if !zip_bundle::is_unibrow_zip_file(zip_path) {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    let unpacked_bytes = estimate_archive_unpacked_bytes(zip_path)?;
    let suggest_folder = hint_threshold_bytes > 0 && unpacked_bytes > hint_threshold_bytes;
    Ok(UnibrowOpenEstimate {
        archive_path: zip_path.to_string_lossy().into_owned(),
        unpacked_bytes,
        suggest_folder,
    })
}

fn read_zip_entry_bytes(zip_path: &Path, entry_name: &str) -> Result<Vec<u8>, ProjectError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let norm = entry_name.replace('\\', "/");
    let mut entry = archive
        .by_name(&norm)
        .map_err(|_| ProjectError::NotPackage(entry_name.into()))?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read_archive_project_manifest(zip_path: &Path) -> Result<ProjectManifest, ProjectError> {
    let data = read_zip_entry_bytes(zip_path, PROJECT_FILE)?;
    let manifest: ProjectManifest = serde_json::from_slice(&data)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// In-memory package for view-only / pre-edit archive sessions (no disk writes).
pub fn open_archive_package_in_memory(
    zip_path: &Path,
    session_id: &str,
) -> Result<ProjectPackage, ProjectError> {
    let manifest = read_archive_project_manifest(zip_path)?;
    Ok(ProjectPackage {
        root: memory_root_for_session(session_id),
        manifest,
    })
}

pub fn is_memory_storage_session(root: &Path) -> bool {
    root.to_string_lossy().starts_with(MEMORY_ROOT_PREFIX)
}

/// Materialise archive interior to `cache_dir` for archive editing or folder unpack.
pub fn materialize_archive_bundle(zip_path: &Path, dest: &Path) -> Result<PathBuf, ProjectError> {
    materialize_zip_to_dir(zip_path, dest)?;
    let root = dest.to_path_buf();
    open_package(&root)?;
    Ok(root)
}

pub fn remove_unpack_destination(dest: &Path) {
    if dest.exists() {
        let _ = fs::remove_dir_all(dest);
    }
}

pub fn unpack_archive_to_folder(zip_path: &Path, dest_folder: &Path) -> Result<PathBuf, ProjectError> {
    unpack_archive_to_folder_cancellable(zip_path, dest_folder, &|| false)
}

pub fn unpack_archive_to_folder_cancellable(
    zip_path: &Path,
    dest_folder: &Path,
    should_cancel: &dyn Fn() -> bool,
) -> Result<PathBuf, ProjectError> {
    if dest_folder.exists() {
        let mut empty = true;
        if dest_folder.is_dir() {
            if let Ok(mut it) = fs::read_dir(dest_folder) {
                empty = it.next().is_none();
            }
        } else {
            return Err(ProjectError::Msg(format!(
                "destination exists and is not a directory: {}",
                dest_folder.display()
            )));
        }
        if !empty {
            return Err(ProjectError::Msg(format!(
                "destination folder is not empty: {}",
                dest_folder.display()
            )));
        }
    }
    materialize_zip_to_dir_cancellable(zip_path, dest_folder, should_cancel)?;
    let root = dest_folder.to_path_buf();
    open_package(&root)?;
    Ok(root)
}

pub fn pack_folder_to_archive(bundle_root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    export_package_to_zip(bundle_root, dest_zip)
}

/// Count bytes in all files under `root` (recursive). Used by tests.
pub fn dir_tree_byte_size(root: &Path) -> u64 {
    if !root.exists() {
        return 0;
    }
    let mut total = 0u64;
    if root.is_file() {
        return fs::metadata(root).map(|m| m.len()).unwrap_or(0);
    }
    let Ok(entries) = fs::read_dir(root) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            total += dir_tree_byte_size(&path);
        } else if let Ok(meta) = fs::metadata(&path) {
            total += meta.len();
        }
    }
    total
}
