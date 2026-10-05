//! Per-origin reader mode on/off for https tabs.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::config_dir;
use crate::web_zoom_prefs::origin_key_from_url;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Store {
    #[serde(default)]
    origins: HashMap<String, bool>,
}

fn store_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("web-reader-prefs.json"))
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

pub fn get_origin_reader_mode(url: &str) -> bool {
    let key = origin_key_from_url(url).unwrap_or_default();
    if key.is_empty() {
        return false;
    }
    let store = load_store();
    store.origins.get(&key).copied().unwrap_or(false)
}

pub fn set_origin_reader_mode(url: &str, enabled: bool) -> Result<(), String> {
    let key = origin_key_from_url(url).ok_or_else(|| "invalid url".to_string())?;
    let mut store = load_store();
    if enabled {
        store.origins.insert(key, true);
    } else {
        store.origins.remove(&key);
    }
    save_store(&store)?;
    Ok(())
}
