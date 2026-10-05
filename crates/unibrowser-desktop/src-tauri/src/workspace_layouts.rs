//! Named presets combining window geometry and command-manager toolbars.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use unibrowser_engine::CommandManagerFile;

use crate::config::config_dir;
use crate::preferences::{PinnedLocation, WindowState};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceChromeState {
    #[serde(default)]
    pub pinned_locations: Vec<PinnedLocation>,
    #[serde(default)]
    pub split_view_enabled: bool,
    #[serde(default)]
    pub split_secondary_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayoutPreset {
    pub id: String,
    pub name: String,
    pub saved_at_ms: u64,
    pub window: WindowState,
    pub command_manager: CommandManagerFile,
    #[serde(default)]
    pub chrome: WorkspaceChromeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayoutsFile {
    #[serde(default)]
    pub presets: Vec<WorkspaceLayoutPreset>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceLayoutPresetSummaryDto {
    pub id: String,
    pub name: String,
    pub saved_at_ms: u64,
}

fn layouts_file_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("workspace-layouts.json"))
}

fn load_file() -> WorkspaceLayoutsFile {
    let path = layouts_file_path();
    if let Some(path) = path {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(file) = serde_json::from_str(&data) {
                return file;
            }
        }
    }
    WorkspaceLayoutsFile {
        presets: Vec::new(),
    }
}

fn save_file(file: &WorkspaceLayoutsFile) -> Result<(), String> {
    let path = layouts_file_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn list_presets() -> Vec<WorkspaceLayoutPresetSummaryDto> {
    load_file()
        .presets
        .iter()
        .map(|p| WorkspaceLayoutPresetSummaryDto {
            id: p.id.clone(),
            name: p.name.clone(),
            saved_at_ms: p.saved_at_ms,
        })
        .collect()
}

pub fn save_preset(
    name: String,
    window: WindowState,
    command_manager: CommandManagerFile,
    chrome: WorkspaceChromeState,
) -> Result<WorkspaceLayoutPresetSummaryDto, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("name required".into());
    }
    let mut file = load_file();
    let id = format!("wl-{:x}", now_ms());
    let preset = WorkspaceLayoutPreset {
        id,
        name: trimmed.to_string(),
        saved_at_ms: now_ms(),
        window,
        command_manager,
        chrome,
    };
    file.presets.push(preset);
    save_file(&file)?;
    let last = file.presets.last().expect("preset just pushed");
    Ok(WorkspaceLayoutPresetSummaryDto {
        id: last.id.clone(),
        name: last.name.clone(),
        saved_at_ms: last.saved_at_ms,
    })
}

pub fn load_preset(id: &str) -> Result<WorkspaceLayoutPreset, String> {
    let file = load_file();
    file.presets
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| "preset not found".into())
}

pub fn delete_preset(id: &str) -> Result<(), String> {
    let mut file = load_file();
    let before = file.presets.len();
    file.presets.retain(|p| p.id != id);
    if file.presets.len() == before {
        return Err("preset not found".into());
    }
    save_file(&file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_name() {
        let err = save_preset(
            "   ".into(),
            WindowState::default(),
            CommandManagerFile::factory_default(),
            WorkspaceChromeState::default(),
        );
        assert!(err.is_err());
    }
}
