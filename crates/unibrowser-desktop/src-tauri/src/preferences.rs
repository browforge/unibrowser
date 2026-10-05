use serde::{Deserialize, Serialize};
use std::fs;
use crate::config::preferences_file_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UiTheme {
    Light,
    Dark,
    System,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::System
    }
}

pub const RECENT_FILES_MAX: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentFileEntry {
    pub path: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    #[serde(default)]
    pub onboarding_dismissed: bool,
    #[serde(default)]
    pub theme: UiTheme,
    #[serde(default)]
    pub recent_files: Vec<RecentFileEntry>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            onboarding_dismissed: false,
            theme: UiTheme::default(),
            recent_files: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_files_dedupe_and_cap() {
        let mut p = Preferences::default();
        for i in 0..20 {
            push_recent_file(&mut p, format!("/tmp/f{i}.txt"), format!("f{i}"));
        }
        assert_eq!(p.recent_files.len(), RECENT_FILES_MAX);
        assert_eq!(p.recent_files[0].label, "f19");
        push_recent_file(&mut p, "/tmp/f5.txt".into(), "f5".into());
        assert_eq!(p.recent_files[0].path, "/tmp/f5.txt");
        assert!(p.recent_files.iter().filter(|e| e.path == "/tmp/f5.txt").count() == 1);
    }
}

pub fn push_recent_file(prefs: &mut Preferences, path: String, label: String) {
    prefs.recent_files.retain(|e| e.path != path);
    prefs.recent_files.insert(
        0,
        RecentFileEntry {
            path,
            label,
        },
    );
    if prefs.recent_files.len() > RECENT_FILES_MAX {
        prefs.recent_files.truncate(RECENT_FILES_MAX);
    }
}

pub fn load_preferences() -> Preferences {
    let path = preferences_file_path();
    if let Some(path) = path {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(p) = serde_json::from_str(&data) {
                return p;
            }
        }
    }
    Preferences::default()
}

pub fn save_preferences(prefs: &Preferences) -> Result<(), String> {
    let path = preferences_file_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            x: None,
            y: None,
        }
    }
}

pub fn load_window_state() -> WindowState {
    let path = crate::config::window_state_file_path();
    if let Some(path) = path {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str(&data) {
                return s;
            }
        }
    }
    WindowState::default()
}

pub fn save_window_state(state: &WindowState) -> Result<(), String> {
    let path = crate::config::window_state_file_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_window_state(window: &tauri::Window, state: &WindowState) -> Result<(), String> {
    use tauri::LogicalSize;
    window
        .set_size(LogicalSize::new(state.width, state.height))
        .map_err(|e| e.to_string())?;
    if let (Some(x), Some(y)) = (state.x, state.y) {
        use tauri::LogicalPosition;
        window
            .set_position(LogicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn capture_window_state(window: &tauri::Window) -> Result<WindowState, String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok(WindowState {
        width: size.width as f64,
        height: size.height as f64,
        x: Some(pos.x as f64),
        y: Some(pos.y as f64),
    })
}
