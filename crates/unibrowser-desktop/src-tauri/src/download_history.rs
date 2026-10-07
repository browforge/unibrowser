//! Recent download attempts (opened-as-tab, saved, or failed).

use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::config_dir;

pub const HISTORY_MAX: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadHistoryStatus {
    OpenedTab,
    InProgress,
    Saved,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadHistoryEntry {
    pub id: String,
    pub url: String,
    pub label: String,
    pub status: DownloadHistoryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u8>,
    pub at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Store {
    #[serde(default)]
    entries: Vec<DownloadHistoryEntry>,
}

fn store_path() -> Option<std::path::PathBuf> {
    config_dir().map(|d| d.join("download-history.json"))
}

fn load_store() -> Store {
    let path = store_path();
    if let Some(path) = path {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str(&data) {
                return s;
            }
        }
    }
    Store::default()
}

fn save_store(store: &Store) -> Result<(), String> {
    let path = store_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn entry_id(url: &str) -> String {
    format!("{:x}", md5_hash(url.as_bytes()))
}

fn md5_hash(data: &[u8]) -> u64 {
    // Simple FNV-1a 64-bit (no extra deps); stable id for retry lookup.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data {
        h ^= *b as u64;
        h = (h * 0x100000001b3) & 0xffff_ffff_ffff_ffff;
    }
    h
}

pub fn list_entries() -> Vec<DownloadHistoryEntry> {
    load_store().entries
}

pub fn clear_entries() -> Result<(), String> {
    save_store(&Store::default())
}

/// Fills demo download rows when history is empty (New tab screenshots).
pub fn seed_demo_entries_if_empty(entries: Vec<DownloadHistoryEntry>) -> Result<(), String> {
    let mut store = load_store();
    if !store.entries.is_empty() {
        return Ok(());
    }
    store.entries = entries;
    save_store(&store)
}

pub fn record_success(url: &str, status: DownloadHistoryStatus, destination: Option<&str>) {
    let label = crate::downloads::suggested_download_filename(url);
    let mut store = load_store();
    let id = entry_id(url);
    store.entries.retain(|e| e.id != id);
    store.entries.insert(
        0,
        DownloadHistoryEntry {
            id,
            url: url.trim().to_string(),
            label,
            status,
            error: None,
            destination: destination.map(String::from),
            progress_percent: None,
            at_ms: now_ms(),
        },
    );
    if store.entries.len() > HISTORY_MAX {
        store.entries.truncate(HISTORY_MAX);
    }
    let _ = save_store(&store);
}

pub fn record_failure(url: &str, error: &str) {
    let label = crate::downloads::suggested_download_filename(url);
    let mut store = load_store();
    let id = entry_id(url);
    store.entries.retain(|e| e.id != id);
    store.entries.insert(
        0,
        DownloadHistoryEntry {
            id,
            url: url.trim().to_string(),
            label,
            status: DownloadHistoryStatus::Failed,
            error: Some(error.chars().take(240).collect()),
            destination: None,
            progress_percent: None,
            at_ms: now_ms(),
        },
    );
    if store.entries.len() > HISTORY_MAX {
        store.entries.truncate(HISTORY_MAX);
    }
    let _ = save_store(&store);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        let e = DownloadHistoryEntry {
            id: "a".into(),
            url: "https://example.com/x".into(),
            label: "x".into(),
            status: DownloadHistoryStatus::OpenedTab,
            error: None,
            destination: None,
            progress_percent: None,
            at_ms: 1,
        };
        let json = serde_json::to_string(&e).expect("json");
        assert!(json.contains("opened_tab"));
    }
}
