//! Opt-in backup of dirty editor buffers under the config dir.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::config::config_dir;
use crate::preferences::load_preferences;

const DEBOUNCE: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupRecord {
    pub path: String,
    pub backend_id: String,
    pub saved_at_ms: u64,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct CrashRecoveryBackupDto {
    pub backup_id: String,
    pub path: String,
    pub backend_id: String,
    pub saved_at_ms: u64,
    pub content_bytes: usize,
}

struct PendingBackup {
    backend_id: String,
    content: String,
    due: Instant,
}

struct DebounceState {
    pending: HashMap<String, PendingBackup>,
}

static DEBOUNCE_STATE: OnceLock<Mutex<DebounceState>> = OnceLock::new();

fn debounce_state() -> &'static Mutex<DebounceState> {
    DEBOUNCE_STATE.get_or_init(|| Mutex::new(DebounceState {
        pending: HashMap::new(),
    }))
}

fn recovery_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("crash-recovery"))
}

pub fn backup_path_for(path: &str) -> Option<PathBuf> {
    let dir = recovery_dir()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    path.hash(&mut hasher);
    let id = hasher.finish();
    Some(dir.join(format!("{:016x}.json", id)))
}

pub fn backup_id_for_path(path: &str) -> Option<String> {
    backup_path_for(path).map(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    })
}

fn write_backup(path: &str, backend_id: &str, content: &str) {
    if content.is_empty() {
        return;
    }
    let dir = match recovery_dir() {
        Some(d) => d,
        None => return,
    };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let dest = match backup_path_for(path) {
        Some(p) => p,
        None => return,
    };
    let record = BackupRecord {
        path: path.to_string(),
        backend_id: backend_id.to_string(),
        saved_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        content: content.to_string(),
    };
    if let Ok(data) = serde_json::to_string(&record) {
        let _ = fs::write(dest, data);
    }
}

fn flush_due_backups() {
    let now = Instant::now();
    let mut guard = debounce_state().lock().expect("crash recovery debounce lock");
    let due_paths: Vec<String> = guard
        .pending
        .iter()
        .filter(|(_, p)| p.due <= now)
        .map(|(k, _)| k.clone())
        .collect();
    for path in due_paths {
        let pending = guard.pending.remove(&path).expect("pending backup");
        write_backup(&path, &pending.backend_id, &pending.content);
    }
}

/// Debounced backup: writes at most every ~2s per path while editing continues.
pub fn maybe_backup_dirty(path: &str, backend_id: &str, content: &str) {
    let prefs = load_preferences();
    if !prefs.crash_recovery_enabled {
        return;
    }
    flush_due_backups();
    let mut guard = debounce_state().lock().expect("crash recovery debounce lock");
    guard.pending.insert(
        path.to_string(),
        PendingBackup {
            backend_id: backend_id.to_string(),
            content: content.to_string(),
            due: Instant::now() + DEBOUNCE,
        },
    );
}

#[cfg(test)]
pub fn flush_all_pending_backups_for_test() {
    let mut guard = debounce_state().lock().expect("crash recovery debounce lock");
    let keys: Vec<String> = guard.pending.keys().cloned().collect();
    for path in keys {
        let pending = guard.pending.remove(&path).expect("pending backup");
        write_backup(&path, &pending.backend_id, &pending.content);
    }
}

fn read_backup_file(file: &PathBuf) -> Option<BackupRecord> {
    let data = fs::read_to_string(file).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn list_backups() -> Vec<CrashRecoveryBackupDto> {
    let dir = match recovery_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let backup_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if let Some(record) = read_backup_file(&path) {
            out.push(CrashRecoveryBackupDto {
                backup_id,
                path: record.path,
                backend_id: record.backend_id,
                saved_at_ms: record.saved_at_ms,
                content_bytes: record.content.len(),
            });
        }
    }
    out.sort_by(|a, b| b.saved_at_ms.cmp(&a.saved_at_ms));
    out
}

pub fn load_backup(backup_id: &str) -> Result<BackupRecord, String> {
    if backup_id.len() != 16 || !backup_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid backup id".into());
    }
    let dir = recovery_dir().ok_or("config path unavailable")?;
    let file = dir.join(format!("{}.json", backup_id));
    read_backup_file(&file).ok_or_else(|| "backup not found".into())
}

pub fn dismiss_backup(backup_id: &str) -> Result<(), String> {
    let dir = recovery_dir().ok_or("config path unavailable")?;
    let file = dir.join(format!("{}.json", backup_id));
    if file.exists() {
        fs::remove_file(&file).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let mut p = load_preferences();
    p.crash_recovery_enabled = enabled;
    crate::preferences::save_preferences(&p)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_path_stable() {
        let a = backup_path_for("/tmp/foo.txt");
        let b = backup_path_for("/tmp/foo.txt");
        assert_eq!(a, b);
    }

    #[test]
    fn backup_id_is_hex_stem() {
        let id = backup_id_for_path("/tmp/foo.txt").expect("id");
        assert_eq!(id.len(), 16);
    }
}
