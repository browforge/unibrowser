//! Per-tab `.unibrow` storage mode sessions (ADR 0032 PKG-03..06).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use unibrowser_project::{
    is_memory_storage_session, materialize_archive_bundle, open_archive_package_in_memory,
    open_package, pack_folder_to_archive, unpack_archive_to_folder, PackageStorageMode,
    UnibrowOpenEstimate,
};

use crate::project_templates::{MemoryProjectEntry, store_memory_project};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStorageSessionDto {
    pub archive_path: Option<String>,
    pub bundle_root: String,
    pub mode: String,
    pub history_path: String,
    pub edited: bool,
    pub history_notice_dismissed: bool,
    pub unpacked_bytes: u64,
    pub suggest_folder: bool,
    pub unpack_in_progress: bool,
    pub unpack_progress: u8,
}

#[derive(Debug, Clone)]
pub struct PackageStorageSession {
    pub tab_path: String,
    pub archive_path: Option<PathBuf>,
    pub bundle_root: PathBuf,
    pub mode: PackageStorageMode,
    pub edited: bool,
    pub history_notice_dismissed: bool,
    pub unpacked_bytes: u64,
    pub suggest_folder: bool,
    pub unpack_in_progress: bool,
    pub unpack_progress: u8,
}

pub struct PackageStorageStore {
    by_tab: Mutex<HashMap<String, PackageStorageSession>>,
}

impl PackageStorageStore {
    pub fn new() -> Self {
        Self {
            by_tab: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, tab_path: &str) -> Option<PackageStorageSession> {
        self.by_tab
            .lock()
            .expect("package_storage")
            .get(tab_path)
            .cloned()
    }

    pub fn insert(&self, session: PackageStorageSession) {
        self.by_tab
            .lock()
            .expect("package_storage")
            .insert(session.tab_path.clone(), session);
    }

    pub fn update<F>(&self, tab_path: &str, f: F) -> Option<PackageStorageSession>
    where
        F: FnOnce(&mut PackageStorageSession),
    {
        let mut map = self.by_tab.lock().expect("package_storage");
        let entry = map.get_mut(tab_path)?;
        f(entry);
        Some(entry.clone())
    }
}

fn mode_label(mode: PackageStorageMode) -> &'static str {
    match mode {
        PackageStorageMode::ViewOnly => "view_only",
        PackageStorageMode::Archive => "archive",
        PackageStorageMode::Folder => "folder",
    }
}

pub fn history_path_for_session(session: &PackageStorageSession) -> String {
    match session.mode {
        PackageStorageMode::ViewOnly | PackageStorageMode::Archive => session
            .archive_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| session.tab_path.clone()),
        PackageStorageMode::Folder => {
            let root = &session.bundle_root;
            root.join(unibrowser_project::HISTORY_FILE)
                .to_string_lossy()
                .into_owned()
        }
    }
}

pub fn session_to_dto(session: &PackageStorageSession) -> PackageStorageSessionDto {
    PackageStorageSessionDto {
        archive_path: session
            .archive_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned()),
        bundle_root: session.bundle_root.to_string_lossy().into_owned(),
        mode: mode_label(session.mode).into(),
        history_path: history_path_for_session(session),
        edited: session.edited,
        history_notice_dismissed: session.history_notice_dismissed,
        unpacked_bytes: session.unpacked_bytes,
        suggest_folder: session.suggest_folder,
        unpack_in_progress: session.unpack_in_progress,
        unpack_progress: session.unpack_progress,
    }
}

pub fn query_unibrow_open(
    zip_path: &Path,
    hint_threshold: u64,
) -> Result<UnibrowOpenEstimate, String> {
    unibrowser_project::unibrow_open_estimate(zip_path, hint_threshold).map_err(|e| e.to_string())
}

pub fn open_with_storage_mode(
    state: &AppState,
    tab_path: &Path,
    zip_path: &Path,
    mode: PackageStorageMode,
    folder_dest: Option<&Path>,
    hint_threshold: u64,
) -> Result<(unibrowser_project::ProjectPackage, PackageStorageSession), String> {
    let estimate = unibrowser_project::unibrow_open_estimate(zip_path, hint_threshold)
        .map_err(|e| e.to_string())?;
    let tab_key = tab_path.to_string_lossy().into_owned();
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;

    let (package, bundle_root) = match mode {
        PackageStorageMode::ViewOnly => {
            let session_id = format!(
                "view-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            );
            let pkg = open_archive_package_in_memory(zip_path, &session_id)
                .map_err(|e| e.to_string())?;
            let entry = MemoryProjectEntry {
                package: pkg.clone(),
                assets: HashMap::new(),
            };
            store_memory_project(state, entry);
            let root = pkg.root.clone();
            (pkg, root)
        }
        PackageStorageMode::Archive => {
            let session_id = format!(
                "arc-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            );
            let pkg = open_archive_package_in_memory(zip_path, &session_id)
                .map_err(|e| e.to_string())?;
            let entry = MemoryProjectEntry {
                package: pkg.clone(),
                assets: HashMap::new(),
            };
            store_memory_project(state, entry);
            let root = pkg.root.clone();
            (pkg, root)
        }
        PackageStorageMode::Folder => {
            let dest = folder_dest.ok_or("folder destination required")?;
            let root = unpack_archive_to_folder(zip_path, dest).map_err(|e| e.to_string())?;
            let pkg = open_package(&root).map_err(|e| e.to_string())?;
            (pkg, root)
        }
    };

    let session = PackageStorageSession {
        tab_path: tab_key,
        archive_path: Some(zip_path.to_path_buf()),
        bundle_root,
        mode,
        edited: false,
        history_notice_dismissed: false,
        unpacked_bytes: estimate.unpacked_bytes,
        suggest_folder: estimate.suggest_folder,
        unpack_in_progress: false,
        unpack_progress: 0,
    };
    state
        .package_storage
        .insert(session.clone());

    Ok((package, session))
}

pub fn ensure_writable_bundle(
    state: &AppState,
    tab_path: &str,
) -> Result<PathBuf, String> {
    let session = state
        .package_storage
        .get(tab_path)
        .ok_or("no package storage session")?;
    if session.mode == PackageStorageMode::ViewOnly {
        return Err("project is open in View only mode".into());
    }
    if is_memory_storage_session(&session.bundle_root) {
        let archive = session
            .archive_path
            .as_ref()
            .ok_or("archive path missing")?;
        let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
        let stamp = archive
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("pkg");
        let dest = cache.join(format!("edit-{}", stamp));
        let root = materialize_archive_bundle(archive, &dest).map_err(|e| e.to_string())?;
        state.package_storage.update(tab_path, |s| {
            s.bundle_root = root.clone();
            s.edited = true;
        });
        return Ok(root);
    }
    state.package_storage.update(tab_path, |s| {
        s.edited = true;
    });
    Ok(session.bundle_root)
}

pub fn run_unpack_active(
    state: &AppState,
    tab_path: &str,
    dest: &Path,
) -> Result<PathBuf, String> {
    let session = state
        .package_storage
        .get(tab_path)
        .ok_or("no package storage session")?;
    let archive = session
        .archive_path
        .as_ref()
        .ok_or("not an archive tab")?;
    state.package_storage.update(tab_path, |s| {
        s.unpack_in_progress = true;
        s.unpack_progress = 10;
    });
    let root = unpack_archive_to_folder(archive, dest).map_err(|e| e.to_string())?;
    state.package_storage.update(tab_path, |s| {
        s.mode = PackageStorageMode::Folder;
        s.bundle_root = root.clone();
        s.unpack_in_progress = false;
        s.unpack_progress = 100;
        s.archive_path = None;
    });
    Ok(root)
}

pub fn run_pack_active(
    state: &AppState,
    tab_path: &str,
    dest_zip: &Path,
) -> Result<(), String> {
    let session = state
        .package_storage
        .get(tab_path)
        .ok_or("no package storage session")?;
    let root = if is_memory_storage_session(&session.bundle_root) {
        ensure_writable_bundle(state, tab_path)?
    } else {
        session.bundle_root.clone()
    };
    pack_folder_to_archive(&root, dest_zip).map_err(|e| e.to_string())?;
    Ok(())
}
