//! Opt-in backup of dirty editor buffers under the config dir.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::config_dir;
use crate::preferences::load_preferences;

#[derive(Debug, Serialize, Deserialize)]
struct BackupRecord {
    pub path: String,
    pub backend_id: String,
    pub saved_at_ms: u64,
    pub content: String,
}

fn recovery_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("crash-recovery"))
}

fn backup_path_for(path: &str) -> Option<PathBuf> {
    let dir = recovery_dir()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    path.hash(&mut hasher);
    let id = hasher.finish();
    Some(dir.join(format!("{:016x}.json", id)))
}

pub fn maybe_backup_dirty(path: &str, backend_id: &str, content: &str) {
    let prefs = load_preferences();
    if !prefs.crash_recovery_enabled {
        return;
    }
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
}
