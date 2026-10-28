//! Host-side edit recovery journal under the application config dir (REC-01).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use unibrowser_document_model::{
    append_checkpoint, begin_journal, clear_journal, journal_id_for_path, list_pending_journals,
    recover_from_journal, PendingJournal, RecoveredEdit,
};

use crate::config::config_dir;

const CHECKPOINT_DEBOUNCE: Duration = Duration::from_millis(800);

struct PendingCheckpoint {
    content: String,
    due: Instant,
}

struct DebounceState {
    pending: std::collections::HashMap<String, PendingCheckpoint>,
}

static DEBOUNCE_STATE: OnceLock<Mutex<DebounceState>> = OnceLock::new();

fn debounce() -> &'static Mutex<DebounceState> {
    DEBOUNCE_STATE.get_or_init(|| Mutex::new(DebounceState {
        pending: std::collections::HashMap::new(),
    }))
}

pub fn journal_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("edit-recovery"))
}

fn flush_due() {
    let dir = match journal_dir() {
        Some(d) => d,
        None => return,
    };
    let now = Instant::now();
    let mut guard = debounce().lock().expect("edit recovery debounce");
    let due: Vec<String> = guard
        .pending
        .iter()
        .filter(|(_, p)| p.due <= now)
        .map(|(k, _)| k.clone())
        .collect();
    for path in due {
        let pending = guard.pending.remove(&path).expect("pending");
        let bytes = pending.content.as_bytes();
        if journal_path_exists(&dir, &path) {
            let _ = append_checkpoint(&dir, &path, bytes);
        }
    }
}

fn journal_path_exists(dir: &Path, doc_path: &str) -> bool {
    dir.join(format!("{}.journal", journal_id_for_path(doc_path))).is_file()
}

/// Call when the buffer becomes dirty; ensures a journal exists and schedules a checkpoint.
pub fn note_dirty_edit(path: &str, backend_id: &str, baseline_on_disk: &[u8], content: &str) {
    let dir = match journal_dir() {
        Some(d) => d,
        None => return,
    };
    flush_due();
    if !journal_path_exists(&dir, path) {
        let _ = begin_journal(&dir, path, backend_id, baseline_on_disk);
    }
    let mut guard = debounce().lock().expect("edit recovery debounce");
    guard.pending.insert(
        path.to_string(),
        PendingCheckpoint {
            content: content.to_string(),
            due: Instant::now() + CHECKPOINT_DEBOUNCE,
        },
    );
}

pub fn flush_pending_for_test() {
    let dir = match journal_dir() {
        Some(d) => d,
        None => return,
    };
    let mut guard = debounce().lock().expect("edit recovery debounce");
    let keys: Vec<String> = guard.pending.keys().cloned().collect();
    for path in keys {
        let pending = guard.pending.remove(&path).expect("pending");
        if journal_path_exists(&dir, &path) {
            let _ = append_checkpoint(&dir, &path, pending.content.as_bytes());
        }
    }
}

pub fn clear_for_path(path: &str) {
    if let Some(dir) = journal_dir() {
        let _ = clear_journal(&dir, path);
    }
    let mut guard = debounce().lock().expect("edit recovery debounce");
    guard.pending.remove(path);
}

pub fn list_pending() -> Vec<PendingJournal> {
    flush_due();
    let dir = match journal_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };
    list_pending_journals(&dir).unwrap_or_default()
}

pub fn load_recovered(journal_id: &str) -> Result<RecoveredEdit, String> {
    let dir = journal_dir().ok_or("config path unavailable")?;
    recover_from_journal(&dir, journal_id)
}

pub fn read_disk_bytes(path: &str, cwd: &Path) -> Result<Vec<u8>, String> {
    let resolved = crate::path_security::resolve_open_path(path, cwd)?;
    fs::read(&resolved).map_err(|e| e.to_string())
}

pub fn backup_id_for_path(path: &str) -> String {
    journal_id_for_path(path)
}

pub fn seed_capture_demo_for_path(path: &str, backend_id: &str, disk: &[u8]) -> Result<(), String> {
    let recovered = format!(
        "{}\n# recovered unsaved paragraph for REC capture\n",
        String::from_utf8_lossy(disk)
    );
    let dir = journal_dir().ok_or("config path unavailable")?;
    let _ = begin_journal(&dir, path, backend_id, disk);
    let _ = append_checkpoint(&dir, path, recovered.as_bytes());
    Ok(())
}

pub fn write_recovery_preview(journal_id: &str) -> Result<std::path::PathBuf, String> {
    let rec = load_recovered(journal_id)?;
    let dir = journal_dir().ok_or("config path unavailable")?;
    let path = dir.join(format!("{}.preview", journal_id));
    std::fs::write(&path, &rec.content).map_err(|e| e.to_string())?;
    Ok(path)
}
