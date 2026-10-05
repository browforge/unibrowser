//! Debounced backups of `project.json` for open `.unibrow` packages (crash recovery).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::config::config_dir;
use crate::preferences::load_preferences;

const DEBOUNCE: Duration = Duration::from_secs(2);

pub const BACKEND_ID: &str = "unibrow-project";

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectBackupRecord {
    /// Tab path (`.unibrow` file or package folder).
    pub project_path: String,
    /// Directory containing `project.json`.
    pub package_root: String,
    pub saved_at_ms: u64,
    pub project_json: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectRecoveryBackupDto {
    pub backup_id: String,
    pub project_path: String,
    pub package_root: String,
    pub saved_at_ms: u64,
    pub bytes: usize,
}

struct PendingBackup {
    package_root: String,
    project_json: String,
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
    config_dir().map(|d| d.join("project-recovery"))
}

pub fn backup_path_for(project_path: &str) -> Option<PathBuf> {
    let dir = recovery_dir()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    project_path.hash(&mut hasher);
    let id = hasher.finish();
    Some(dir.join(format!("{:016x}.json", id)))
}

pub fn backup_id_for_path(project_path: &str) -> Option<String> {
    backup_path_for(project_path).map(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    })
}

fn recovery_enabled() -> bool {
    let prefs = load_preferences();
    prefs.crash_recovery_enabled || prefs.project_autosave_enabled
}

fn write_backup(project_path: &str, package_root: &str, project_json: &str) {
    if project_json.is_empty() {
        return;
    }
    let dir = match recovery_dir() {
        Some(d) => d,
        None => return,
    };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let dest = match backup_path_for(project_path) {
        Some(p) => p,
        None => return,
    };
    let record = ProjectBackupRecord {
        project_path: project_path.to_string(),
        package_root: package_root.to_string(),
        saved_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        project_json: project_json.to_string(),
    };
    if let Ok(data) = serde_json::to_string(&record) {
        let _ = fs::write(dest, data);
    }
}

fn flush_due_backups() {
    let now = Instant::now();
    let mut guard = debounce_state().lock().expect("project recovery debounce lock");
    let due_paths: Vec<String> = guard
        .pending
        .iter()
        .filter(|(_, p)| p.due <= now)
        .map(|(k, _)| k.clone())
        .collect();
    for path in due_paths {
        let pending = guard.pending.remove(&path).expect("pending project backup");
        write_backup(&path, &pending.package_root, &pending.project_json);
    }
}

/// Queue a debounced snapshot of `project.json` after edits.
pub fn maybe_backup_project(project_path: &str, package_root: &Path) {
    if !recovery_enabled() {
        return;
    }
    let manifest = package_root.join(unibrowser_project::PROJECT_FILE);
    let project_json = match fs::read_to_string(&manifest) {
        Ok(s) => s,
        Err(_) => return,
    };
    flush_due_backups();
    let mut guard = debounce_state().lock().expect("project recovery debounce lock");
    guard.pending.insert(
        project_path.to_string(),
        PendingBackup {
            package_root: package_root.to_string_lossy().into_owned(),
            project_json,
            due: Instant::now() + DEBOUNCE,
        },
    );
}

#[cfg(test)]
pub fn flush_all_pending_for_test() {
    let mut guard = debounce_state().lock().expect("project recovery debounce lock");
    let keys: Vec<String> = guard.pending.keys().cloned().collect();
    for path in keys {
        let pending = guard.pending.remove(&path).expect("pending");
        write_backup(&path, &pending.package_root, &pending.project_json);
    }
}

fn read_backup_file(file: &PathBuf) -> Option<ProjectBackupRecord> {
    let data = fs::read_to_string(file).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn list_backups() -> Vec<ProjectRecoveryBackupDto> {
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
            out.push(ProjectRecoveryBackupDto {
                backup_id,
                project_path: record.project_path,
                package_root: record.package_root,
                saved_at_ms: record.saved_at_ms,
                bytes: record.project_json.len(),
            });
        }
    }
    out.sort_by(|a, b| b.saved_at_ms.cmp(&a.saved_at_ms));
    out
}

pub fn load_backup(backup_id: &str) -> Result<ProjectBackupRecord, String> {
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

pub fn restore_backup_to_disk(record: &ProjectBackupRecord) -> Result<(), String> {
    let root = PathBuf::from(&record.package_root);
    let dest = root.join(unibrowser_project::PROJECT_FILE);
    fs::write(&dest, &record.project_json).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_path_stable() {
        let a = backup_path_for("/tmp/foo.unibrow");
        let b = backup_path_for("/tmp/foo.unibrow");
        assert_eq!(a, b);
    }
}
