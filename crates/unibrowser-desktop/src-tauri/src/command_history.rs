//! Last palette commands (local config only).

use serde::{Deserialize, Serialize};
use std::fs;

use crate::config::config_dir;

pub const MAX_ENTRIES: usize = 10;

const SKIP: &[&str] = &[
    "unibrowser.palette.toggle",
    "unibrowser.help.context",
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HistoryFile {
    #[serde(default)]
    entries: Vec<String>,
}

fn history_path() -> Option<std::path::PathBuf> {
    config_dir().map(|d| d.join("command-history.json"))
}

fn load_file() -> HistoryFile {
    let path = history_path();
    let Some(path) = path else {
        return HistoryFile::default();
    };
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_file(file: &HistoryFile) -> Result<(), String> {
    let path = history_path().ok_or("config dir unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn record(command_id: &str) {
    if command_id.is_empty() || SKIP.contains(&command_id) {
        return;
    }
    let mut file = load_file();
    file.entries.retain(|id| id != command_id);
    file.entries.insert(0, command_id.to_string());
    file.entries.truncate(MAX_ENTRIES);
    let _ = save_file(&file);
}

pub fn list() -> Vec<String> {
    load_file().entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_palette_toggle() {
        let before = list();
        record("unibrowser.palette.toggle");
        assert_eq!(list(), before);
    }
}
