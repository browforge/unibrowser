//! Per-origin page zoom for live web tabs (CSS zoom injection, separate from font-size).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::config_dir;

const DEFAULT_PERCENT: u8 = 100;
const MIN_PERCENT: u8 = 25;
const MAX_PERCENT: u8 = 255;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Store {
    #[serde(default)]
    origins: HashMap<String, u8>,
}

fn store_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("web-zoom-prefs.json"))
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

pub fn origin_key_from_url(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url.trim()).ok()?;
    Some(parsed.origin().ascii_serialization())
}

pub fn get_origin_zoom_percent(url: &str) -> u8 {
    let key = origin_key_from_url(url).unwrap_or_default();
    if key.is_empty() {
        return DEFAULT_PERCENT;
    }
    let store = load_store();
    store.origins.get(&key).copied().unwrap_or(DEFAULT_PERCENT)
}

pub fn set_origin_zoom_percent(url: &str, percent: u8) -> Result<u8, String> {
    let key = origin_key_from_url(url).ok_or_else(|| "invalid url".to_string())?;
    let percent = percent.clamp(MIN_PERCENT, MAX_PERCENT);
    let mut store = load_store();
    store.origins.insert(key, percent);
    save_store(&store)?;
    Ok(percent)
}

pub fn adjust_origin_zoom_percent(url: &str, delta: i32) -> Result<u8, String> {
    let current = get_origin_zoom_percent(url) as i32;
    let next = (current + delta).clamp(MIN_PERCENT as i32, MAX_PERCENT as i32) as u8;
    set_origin_zoom_percent(url, next)
}

pub fn reset_origin_zoom_percent(url: &str) -> Result<u8, String> {
    let key = origin_key_from_url(url).ok_or_else(|| "invalid url".to_string())?;
    let mut store = load_store();
    store.origins.remove(&key);
    save_store(&store)?;
    Ok(DEFAULT_PERCENT)
}

/// Clear every saved per-origin zoom; returns how many origins were removed.
pub fn reset_all_origin_zoom_prefs() -> Result<usize, String> {
    let mut store = load_store();
    let count = store.origins.len();
    store.origins.clear();
    save_store(&store)?;
    Ok(count)
}
