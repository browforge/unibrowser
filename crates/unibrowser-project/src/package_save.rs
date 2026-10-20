//! Journalled folder saves and atomic archive replace ([PKG-07], ADR 0032 §5).

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::cell::RefCell;

use serde::{Deserialize, Serialize};
use unibrowser_document_model::history_file::{self, HistoryLogEntry};

use crate::opc_bundle::write_opc_zip_from_root;
use crate::{
    gc_content_addressed_assets, list_referenced_content_hashes, ProjectError, ProjectManifest,
    ProjectPackage, DIR_DERIVED, HISTORY_FILE, MANIFEST_FILE, PROJECT_FILE,
};

pub const OP_SAVE_BEGIN: &str = "unibrowser.package.save_begin";
pub const OP_SAVE_PART: &str = "unibrowser.package.save_part";
pub const OP_SAVE_COMMIT: &str = "unibrowser.package.save_commit";
pub const OP_SAVE_ABORT: &str = "unibrowser.package.save_abort";

const JOURNAL_BACKUP_DIR: &str = "save-journal";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFaultPoint {
    AfterSaveBeginJournal,
    AfterFirstPartJournal,
    AfterFirstPartWrite,
    BeforeSaveCommitJournal,
    AfterArchiveTempWrite,
    BeforeArchiveRename,
}

/// Return `true` to abort the save at this point (simulates crash).
type FaultHook = Box<dyn Fn(SaveFaultPoint) -> bool>;

thread_local! {
    static FAULT_HOOK: RefCell<Option<FaultHook>> = RefCell::new(None);
}

/// Test-only: abort save at a fixed point (simulates kill mid-write) on this thread.
pub fn set_save_fault_hook(hook: Option<FaultHook>) {
    FAULT_HOOK.with(|cell| {
        *cell.borrow_mut() = hook;
    });
}

fn maybe_fault(point: SaveFaultPoint) -> Result<(), ProjectError> {
    FAULT_HOOK.with(|cell| {
        if let Some(h) = cell.borrow().as_ref() {
            if h(point) {
                return Err(ProjectError::Msg(format!(
                    "save fault injection at {:?}",
                    point
                )));
            }
        }
        Ok(())
    })
}

fn history_path(root: &Path) -> PathBuf {
    root.join(HISTORY_FILE)
}

fn journal_backup_root(root: &Path, save_id: &str) -> PathBuf {
    root.join(DIR_DERIVED).join(JOURNAL_BACKUP_DIR).join(save_id)
}

fn now_id() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("save-{}", secs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveBeginPayload {
    save_id: String,
    parts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavePartPayload {
    save_id: String,
    part: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveCommitPayload {
    save_id: String,
}

fn append_package_journal_op(
    root: &Path,
    op_type: &str,
    payload: serde_json::Value,
) -> Result<(), ProjectError> {
    let path = history_path(root);
    let (header, mut ops) = history_file::read_header_and_ops(&path).map_err(ProjectError::Msg)?;
    let parent_seq = ops.last().map(|o| o.seq).unwrap_or(0);
    let seq = parent_seq + 1;
    let entry = HistoryLogEntry {
        op_id: format!("pkg-{}-{}", seq, history_file::now_rfc3339()),
        seq,
        parent_seq,
        timestamp: history_file::now_rfc3339(),
        plugin_id: "unibrowser-core".into(),
        plugin_version: "0.1.0".into(),
        op_type: op_type.into(),
        op_schema_version: 1,
        payload,
    };
    let header = unibrowser_document_model::history_pins::upsert_pin_from_entry(&header, &entry);
    ops.push(entry);
    history_file::write_file_atomic(&path, &header, &ops).map_err(ProjectError::Msg)?;
    Ok(())
}

fn backup_part(root: &Path, save_id: &str, rel: &str) -> Result<(), ProjectError> {
    let src = root.join(rel);
    if !src.is_file() {
        return Ok(());
    }
    let dest = journal_backup_root(root, save_id).join(rel);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&src, &dest)?;
    Ok(())
}

fn restore_part(root: &Path, save_id: &str, rel: &str) -> Result<(), ProjectError> {
    let backup = journal_backup_root(root, save_id).join(rel);
    let dest = root.join(rel);
    if backup.is_file() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&backup, &dest)?;
    } else if dest.is_file() {
        fs::remove_file(&dest)?;
    }
    Ok(())
}

fn clear_journal_backup(root: &Path, save_id: &str) {
    let dir = journal_backup_root(root, save_id);
    let _ = fs::remove_dir_all(&dir);
}

/// Incomplete `save_begin` without matching `save_commit` after it.
pub fn pending_folder_save_id(root: &Path) -> Result<Option<String>, ProjectError> {
    let path = history_path(root);
    if !path.is_file() {
        return Ok(None);
    }
    let (_, ops) = history_file::read_header_and_ops(&path).map_err(ProjectError::Msg)?;
    let mut open_begin: Option<String> = None;
    for op in &ops {
        if op.op_type == OP_SAVE_BEGIN {
            open_begin = op
                .payload
                .get("save_id")
                .and_then(|v| v.as_str())
                .map(String::from);
        } else if op.op_type == OP_SAVE_COMMIT || op.op_type == OP_SAVE_ABORT {
            let id = op.payload.get("save_id").and_then(|v| v.as_str());
            if open_begin.as_deref() == id {
                open_begin = None;
            }
        }
    }
    Ok(open_begin)
}

/// Roll back a crashed folder save using journal backups and history markers.
pub fn recover_incomplete_folder_save(root: &Path) -> Result<Option<String>, ProjectError> {
    let save_id = pending_folder_save_id(root)?;
    let Some(save_id) = save_id else {
        return Ok(None);
    };
    let path = history_path(root);
    let (_, ops) = history_file::read_header_and_ops(&path).map_err(ProjectError::Msg)?;
    let mut parts: Vec<String> = Vec::new();
    let mut in_pending = false;
    for op in &ops {
        if op.op_type == OP_SAVE_BEGIN
            && op.payload.get("save_id").and_then(|v| v.as_str()) == Some(save_id.as_str())
        {
            in_pending = true;
            parts = op
                .payload
                .get("parts")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            continue;
        }
        if !in_pending {
            continue;
        }
        if op.op_type == OP_SAVE_PART {
            if let Some(p) = op.payload.get("part").and_then(|v| v.as_str()) {
                parts.push(p.to_string());
            }
        }
        if op.op_type == OP_SAVE_COMMIT {
            in_pending = false;
        }
    }
    for rel in parts {
        restore_part(root, &save_id, &rel)?;
    }
    append_package_journal_op(
        root,
        OP_SAVE_ABORT,
        serde_json::to_value(SaveCommitPayload {
            save_id: save_id.clone(),
        })?,
    )?;
    clear_journal_backup(root, &save_id);
    Ok(Some(save_id))
}

/// Parts written during a folder save (manifest, project graph, container manifest).
fn folder_save_parts(root: &Path, manifest: &ProjectManifest) -> Vec<String> {
    let mut parts = vec![MANIFEST_FILE.to_string(), PROJECT_FILE.to_string()];
    for hash in list_referenced_content_hashes(root, manifest) {
        parts.push(format!("assets/{}", hash));
    }
    parts.sort();
    parts.dedup();
    parts
}

/// Journalled folder save: history entries precede each part write; GC runs on success.
pub fn save_folder_package(package: &ProjectPackage) -> Result<(), ProjectError> {
    let root = &package.root;
    if pending_folder_save_id(root)?.is_some() {
        recover_incomplete_folder_save(root)?;
    }
    gc_content_addressed_assets(root, &package.manifest, None)?;
    let save_id = now_id();
    let parts = folder_save_parts(root, &package.manifest);
    for rel in &parts {
        backup_part(root, &save_id, rel)?;
    }
    append_package_journal_op(
        root,
        OP_SAVE_BEGIN,
        serde_json::to_value(SaveBeginPayload {
            save_id: save_id.clone(),
            parts: parts.clone(),
        })?,
    )?;
    maybe_fault(SaveFaultPoint::AfterSaveBeginJournal)?;

    let mut to_write = package.manifest.clone();
    crate::validate_manifest_for_save(&to_write)?;
    if to_write.title.trim().is_empty() {
        to_write.title = "Untitled project".into();
    }
    to_write.format_version = crate::FORMAT_VERSION;

    let save_id_journal = save_id.clone();
    let part_idx = std::cell::RefCell::new(0usize);
    let journal_part =
        |rel: &str| -> Result<(), ProjectError> {
            append_package_journal_op(
                root,
                OP_SAVE_PART,
                serde_json::to_value(SavePartPayload {
                    save_id: save_id_journal.clone(),
                    part: rel.to_string(),
                })?,
            )?;
            let idx = *part_idx.borrow();
            if idx == 0 {
                maybe_fault(SaveFaultPoint::AfterFirstPartJournal)?;
            }
            *part_idx.borrow_mut() = idx + 1;
            Ok(())
        };

    journal_part(MANIFEST_FILE)?;
    crate::opc_bundle::migrate_assets_to_content_hashes_with_hook(
        root,
        &mut to_write,
        Some(&|rel| {
            journal_part(rel).expect("journal part");
        }),
    )?;
    crate::opc_bundle::write_container_manifest(root, &to_write)?;
    crate::opc_bundle::ensure_package_history_file(root)?;

    journal_part(PROJECT_FILE)?;
    let data = serde_json::to_string_pretty(&to_write)?;
    unibrowser_fs::atomic_write(&root.join(PROJECT_FILE), data.as_bytes())
        .map_err(|e| ProjectError::Msg(e.to_string()))?;
    maybe_fault(SaveFaultPoint::AfterFirstPartWrite)?;

    maybe_fault(SaveFaultPoint::BeforeSaveCommitJournal)?;
    append_package_journal_op(
        root,
        OP_SAVE_COMMIT,
        serde_json::to_value(SaveCommitPayload {
            save_id: save_id.clone(),
        })?,
    )?;
    clear_journal_backup(root, &save_id);
    Ok(())
}

pub fn archive_part_path(archive: &Path) -> PathBuf {
    let name = archive
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "package.unibrow".into());
    archive
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}.part", name))
}

/// Replace `.unibrow` atomically from an on-disk bundle root (temp zip + fsync + rename).
pub fn save_archive_package(archive_path: &Path, bundle_root: &Path) -> Result<(), ProjectError> {
    if archive_path.extension().and_then(|e| e.to_str()) != Some("unibrow") {
        return Err(ProjectError::Msg("not a .unibrow archive path".into()));
    }
    let part = archive_part_path(archive_path);
    if part.exists() {
        let _ = fs::remove_file(&part);
    }
    write_opc_zip_from_root(bundle_root, &part, false)?;
    maybe_fault(SaveFaultPoint::AfterArchiveTempWrite)?;

    {
        let mut f = File::open(&part)?;
        f.sync_all()?;
    }
    maybe_fault(SaveFaultPoint::BeforeArchiveRename)?;

    if archive_path.is_file() {
        fs::rename(&part, archive_path)?;
    } else {
        fs::rename(&part, archive_path)?;
    }
    Ok(())
}

/// If a stale `.part` exists beside the archive, remove it when the main file is valid.
pub fn recover_incomplete_archive_save(archive_path: &Path) -> Result<bool, ProjectError> {
    let part = archive_part_path(archive_path);
    if !part.is_file() {
        return Ok(false);
    }
    if archive_path.is_file() && crate::opc_bundle::is_opc_unibrow_zip(archive_path) {
        fs::remove_file(&part)?;
        return Ok(true);
    }
    if archive_path.is_file() {
        fs::remove_file(archive_path)?;
    }
    fs::rename(&part, archive_path)?;
    Ok(true)
}

/// Validate a zip file opens (not a torn write).
pub fn archive_zip_is_readable(path: &Path) -> bool {
    if let Ok(f) = File::open(path) {
        return zip::ZipArchive::new(f).is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_package, open_package, write_manifest, ProjectManifest};

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ub-save-{}-{}", label, std::process::id()))
    }

    #[test]
    fn pkg07_archive_atomic_replace_keeps_old_on_fault_before_rename() {
        let dir = temp_root("arc-fault");
        let _ = fs::remove_dir_all(&dir);
        let pkg_dir = dir.join("bundle");
        let _pkg = create_package(&pkg_dir, "Arc").expect("create");
        let zip_path = dir.join("pkg.unibrow");
        save_archive_package(&zip_path, &pkg_dir).expect("first save");
        let before = fs::read(&zip_path).expect("read");
        set_save_fault_hook(Some(Box::new(|p| p == SaveFaultPoint::BeforeArchiveRename)));
        let mut pkg = open_package(&pkg_dir).expect("open");
        pkg.manifest.title = "Changed".into();
        write_manifest(&pkg_dir, &pkg.manifest).expect("write");
        assert!(save_archive_package(&zip_path, &pkg_dir).is_err());
        set_save_fault_hook(None);
        assert_eq!(fs::read(&zip_path).expect("read"), before);
        assert!(archive_zip_is_readable(&zip_path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pkg07_folder_save_recovery_after_fault() {
        let dir = temp_root("fold-fault");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Fold").expect("create");
        set_save_fault_hook(Some(Box::new(|p| p == SaveFaultPoint::BeforeSaveCommitJournal)));
        pkg.manifest.title = "Interrupted".into();
        assert!(save_folder_package(&pkg).is_err());
        set_save_fault_hook(None);
        assert!(pending_folder_save_id(&dir).expect("pending").is_some());
        recover_incomplete_folder_save(&dir).expect("recover");
        let again = open_package(&dir).expect("open");
        assert_eq!(again.manifest.title, "Fold");
        let _ = fs::remove_dir_all(&dir);
    }
}
