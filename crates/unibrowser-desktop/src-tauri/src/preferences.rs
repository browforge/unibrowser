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

/// How completed downloads are handled (ADR 0009). Default: open in a new tab.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DownloadPreference {
    #[default]
    OpenInNewTab,
    SaveToDisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentFileEntry {
    pub path: String,
    pub label: String,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    #[serde(default)]
    pub onboarding_dismissed: bool,
    #[serde(default)]
    pub theme: UiTheme,
    #[serde(default)]
    pub recent_files: Vec<RecentFileEntry>,
    /// When true, last tab set is saved on exit and restored on launch.
    #[serde(default)]
    pub restore_session_on_launch: bool,
    /// Opt-in automatic save for text/SVG (off by default).
    #[serde(default)]
    pub auto_save_enabled: bool,
    /// Seconds between auto-save attempts (clamped 30–600 when enabled).
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_secs: u32,
    #[serde(default)]
    pub pinned_locations: Vec<PinnedLocation>,
    #[serde(default)]
    pub download_preference: DownloadPreference,
    /// Snap board card positions to a 24px grid in `.unibrow` projects.
    #[serde(default)]
    pub board_snap_to_grid: bool,
    /// Play short Web Audio beeps when MIDI clips are scrubbed on the timeline.
    #[serde(default)]
    pub midi_scrub_audio_enabled: bool,
    /// Backup `project.json` under `derived/` for open `.unibrow` tabs.
    #[serde(default)]
    pub project_autosave_enabled: bool,
    #[serde(default = "default_auto_save_interval")]
    pub project_autosave_interval_secs: u32,
    /// Backup dirty text/SVG buffers under config dir (off by default).
    #[serde(default)]
    pub crash_recovery_enabled: bool,
    #[serde(default)]
    pub ntp_sections: NtpSectionPrefs,
    /// When true, embedded https webviews may open WebKit/Chromium devtools (advanced).
    #[serde(default)]
    pub advanced_webview_devtools: bool,
    #[serde(default)]
    pub files_view: FilesViewPrefs,
    /// Favourite folder paths (bookmarked from files view).
    #[serde(default)]
    pub folder_favorites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PinnedLocation {
    pub id: String,
    pub label: String,
    pub location: String,
}

fn default_auto_save_interval() -> u32 {
    120
}

fn default_true() -> bool {
    true
}

/// Which blocks appear on the new tab page (`unibrowser://home`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NtpSectionPrefs {
    #[serde(default = "default_true")]
    pub show_hero: bool,
    #[serde(default = "default_true")]
    pub show_actions: bool,
    #[serde(default = "default_true")]
    pub show_browse: bool,
    #[serde(default = "default_true")]
    pub show_recent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FilesSortMode {
    #[default]
    Name,
    Date,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesViewPrefs {
    #[serde(default)]
    pub sort: FilesSortMode,
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
    #[serde(default)]
    pub show_hidden: bool,
}

impl Default for FilesViewPrefs {
    fn default() -> Self {
        Self {
            sort: FilesSortMode::Name,
            sort_ascending: true,
            show_hidden: false,
        }
    }
}

impl Default for NtpSectionPrefs {
    fn default() -> Self {
        Self {
            show_hero: true,
            show_actions: true,
            show_browse: true,
            show_recent: true,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            onboarding_dismissed: false,
            theme: UiTheme::default(),
            recent_files: Vec::new(),
            restore_session_on_launch: false,
            auto_save_enabled: false,
            auto_save_interval_secs: default_auto_save_interval(),
            pinned_locations: Vec::new(),
            download_preference: DownloadPreference::default(),
            board_snap_to_grid: false,
            midi_scrub_audio_enabled: false,
            project_autosave_enabled: false,
            project_autosave_interval_secs: default_auto_save_interval(),
            crash_recovery_enabled: false,
            ntp_sections: NtpSectionPrefs::default(),
            advanced_webview_devtools: false,
            files_view: FilesViewPrefs::default(),
            folder_favorites: Vec::new(),
        }
    }
}

pub fn toggle_folder_favorite(prefs: &mut Preferences, path: &str) {
    if let Some(idx) = prefs.folder_favorites.iter().position(|p| p == path) {
        prefs.folder_favorites.remove(idx);
        return;
    }
    prefs.folder_favorites.insert(0, path.to_string());
    if prefs.folder_favorites.len() > 24 {
        prefs.folder_favorites.truncate(24);
    }
}

pub fn default_pinned_locations(home: &str) -> Vec<PinnedLocation> {
    vec![
        PinnedLocation {
            id: "home".into(),
            label: "Home".into(),
            location: format!("file://{}", home),
        },
        PinnedLocation {
            id: "downloads".into(),
            label: "Downloads".into(),
            location: format!("file://{}/Downloads", home.trim_end_matches('/')),
        },
        PinnedLocation {
            id: "new-project".into(),
            label: "New project".into(),
            location: "unibrowser://action/new-project".into(),
        },
        PinnedLocation {
            id: "settings".into(),
            label: "Settings".into(),
            location: "unibrowser://settings".into(),
        },
    ]
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
    let pinned = prefs
        .recent_files
        .iter()
        .find(|e| e.path == path)
        .map(|e| e.pinned)
        .unwrap_or(false);
    prefs.recent_files.retain(|e| e.path != path);
    prefs.recent_files.insert(
        0,
        RecentFileEntry {
            path,
            label,
            pinned,
        },
    );
    if prefs.recent_files.len() > RECENT_FILES_MAX {
        prefs.recent_files.truncate(RECENT_FILES_MAX);
    }
}

pub fn toggle_recent_pin(prefs: &mut Preferences, path: &str) -> bool {
    if let Some(entry) = prefs.recent_files.iter_mut().find(|e| e.path == path) {
        entry.pinned = !entry.pinned;
        return entry.pinned;
    }
    false
}

pub fn clear_unpinned_recent(prefs: &mut Preferences) {
    prefs.recent_files.retain(|e| e.pinned);
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
