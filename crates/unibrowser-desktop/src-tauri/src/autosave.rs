//! Local rotating autosave slots (ADR 0023). Never overwrites the user's file.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::config_dir;
use crate::preferences::load_preferences;

static LAST_AUTOSAVE_MS: OnceLock<Mutex<u64>> = OnceLock::new();

fn last_ms() -> &'static Mutex<u64> {
    LAST_AUTOSAVE_MS.get_or_init(|| Mutex::new(0))
}

pub fn last_autosave_at_ms() -> u64 {
    last_ms().lock().map(|v| *v).unwrap_or(0)
}

fn touch_last_autosave() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if let Ok(mut g) = last_ms().lock() {
        *g = now;
    }
    now
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveSlotRecord {
    pub document_path: String,
    pub backend_id: String,
    pub content: String,
    pub saved_at_ms: u64,
}

/// True when Save must show a path picker (untitled buffers only).
pub fn document_needs_save_picker(document_path: &str) -> bool {
    let trimmed = document_path.trim();
    if trimmed.is_empty() {
        return true;
    }
    if let Some(cfg) = config_dir() {
        let untitled_root = cfg.join("autosave").join("untitled");
        let path = Path::new(trimmed);
        if path.starts_with(&untitled_root) {
            return true;
        }
    }
    false
}

pub fn autosave_slot_dir(document_path: &str) -> Option<PathBuf> {
    let trimmed = document_path.trim();
    if trimmed.is_empty() {
        let cfg = config_dir()?;
        return Some(cfg.join("autosave").join("untitled"));
    }
    let path = Path::new(trimmed);
    if path.is_absolute() || trimmed.contains('/') || trimmed.contains('\\') {
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".into());
        return Some(parent.join(format!(".{}.ubautosave", name)));
    }
    let cfg = config_dir()?;
    Some(cfg.join("autosave").join("by-name").join(trimmed))
}

pub fn write_document_autosave_slot(
    document_path: &str,
    backend_id: &str,
    content: &str,
) -> Result<PathBuf, String> {
    if content.is_empty() {
        return Err("empty content".into());
    }
    let prefs = load_preferences();
    let keep = prefs.auto_save_slot_count.clamp(1, 32) as usize;
    let dir = autosave_slot_dir(document_path).ok_or("config dir unavailable")?;
    let saved_at_ms = touch_last_autosave();
    let record = AutosaveSlotRecord {
        document_path: document_path.to_string(),
        backend_id: backend_id.to_string(),
        content: content.to_string(),
        saved_at_ms,
    };
    let data = serde_json::to_string(&record).map_err(|e| e.to_string())?;
    unibrowser_fs::write_rotating_slot(&dir, keep, data.as_bytes())
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct AutosaveSlotSummary {
    pub path: String,
    pub saved_at_ms: u64,
}

pub fn list_autosave_slots(document_path: &str) -> Result<Vec<AutosaveSlotSummary>, String> {
    let dir = autosave_slot_dir(document_path).ok_or("config dir unavailable")?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(record) = serde_json::from_str::<AutosaveSlotRecord>(&text) {
            out.push(AutosaveSlotSummary {
                path: path.to_string_lossy().into_owned(),
                saved_at_ms: record.saved_at_ms,
            });
        }
    }
    out.sort_by(|a, b| b.saved_at_ms.cmp(&a.saved_at_ms));
    Ok(out)
}

pub fn read_latest_autosave_content(document_path: &str) -> Result<Option<AutosaveSlotRecord>, String> {
    let slots = list_autosave_slots(document_path)?;
    let Some(latest) = slots.first() else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&latest.path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string()).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn autosave_timer_writes_slot() {
        let cfg = tempfile::tempdir().expect("tempdir");
        // Override via direct dir write (unit test path)
        let dir = cfg.path().join("slots");
        let record = AutosaveSlotRecord {
            document_path: "/tmp/notes.txt".into(),
            backend_id: "base-text".into(),
            content: "hello autosave".into(),
            saved_at_ms: 1,
        };
        let data = serde_json::to_string(&record).expect("json");
        let path =
            unibrowser_fs::write_rotating_slot(&dir, 3, data.as_bytes()).expect("write");
        assert!(path.is_file());
        let read = fs::read_to_string(&path).expect("read");
        assert!(read.contains("hello autosave"));
    }
}
