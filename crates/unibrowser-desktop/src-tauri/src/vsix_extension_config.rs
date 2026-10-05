//! Per-extension JSON settings for the VSIX `workspace.getConfiguration` shim.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn config_path() -> Option<std::path::PathBuf> {
    crate::config::config_dir().map(|d| d.join("vsix-extension-config.json"))
}

pub fn load_extension_configuration(extension_id: &str) -> Value {
    let path = config_path();
    let Some(path) = path else {
        return Value::Object(serde_json::Map::new());
    };
    if !path.is_file() {
        return Value::Object(serde_json::Map::new());
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    let root: HashMap<String, Value> =
        serde_json::from_str(&data).unwrap_or_default();
    root.get(extension_id)
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()))
}

pub fn save_all(configs: &HashMap<String, Value>) -> Result<(), String> {
    let path = config_path().ok_or("config dir unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(configs).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn ensure_config_dir() -> Option<PathBuf> {
    config_path().and_then(|p| p.parent().map(|d| d.to_path_buf()))
}
