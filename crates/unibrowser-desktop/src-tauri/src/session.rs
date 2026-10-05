//! Last tab session snapshot (restored only when preference opt-in).

use serde::{Deserialize, Serialize};
use std::fs;
use crate::config::session_file_path;
use crate::preferences::{load_preferences, save_preferences, WindowState};
use crate::state::AppState;
use tauri::{AppHandle, Manager};
use unibrowser_mod_api::BrowserLocation;

pub const SESSION_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowSessionEntry {
    pub label: String,
    pub tab_inputs: Vec<String>,
    pub active_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<WindowState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSnapshot {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub windows: Vec<WindowSessionEntry>,
    /// Legacy v1 fields (migrated into `windows` on load).
    #[serde(default)]
    pub tab_inputs: Vec<String>,
    #[serde(default)]
    pub active_index: usize,
}

impl SessionSnapshot {
    pub fn normalized_windows(&self) -> Vec<WindowSessionEntry> {
        if !self.windows.is_empty() {
            return self.windows.clone();
        }
        if self.tab_inputs.is_empty() {
            return Vec::new();
        }
        vec![WindowSessionEntry {
            label: "main".into(),
            tab_inputs: self.tab_inputs.clone(),
            active_index: self.active_index,
            geometry: None,
        }]
    }
}

pub fn save_session_snapshot(snapshot: &SessionSnapshot) -> Result<(), String> {
    let path = session_file_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(snapshot).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_session_snapshot() -> Option<SessionSnapshot> {
    let path = session_file_path()?;
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn location_to_omnibox(loc: &BrowserLocation) -> String {
    match loc {
        BrowserLocation::File { path } => format!("file://{}", path.display()),
        BrowserLocation::Http { url } => url.clone(),
        BrowserLocation::Internal { page } => format!("unibrowser://{}", page),
    }
}

pub fn capture_window(state: &AppState, window_label: &str) -> WindowSessionEntry {
    let session = state.session(window_label).ok();
    let (tab_inputs, active_index) = match session {
        Some(sess) => match sess.tabs.lock() {
            Ok(tabs) => (
                tabs
                    .tabs()
                    .iter()
                    .map(|t| location_to_omnibox(&t.location))
                    .collect(),
                tabs.active_index(),
            ),
            Err(_) => (Vec::new(), 0),
        },
        None => (Vec::new(), 0),
    };
    WindowSessionEntry {
        label: window_label.to_string(),
        tab_inputs,
        active_index,
        geometry: None,
    }
}

pub fn capture_all_windows(state: &AppState, app: &AppHandle) -> SessionSnapshot {
    let mut windows = Vec::new();
    for (label, _window) in app.webview_windows() {
        let mut entry = capture_window(state, &label);
        if let Some(window) = app.get_window(&label) {
            if let Ok(geom) = crate::preferences::capture_window_state(&window) {
                entry.geometry = Some(geom);
            }
        }
        windows.push(entry);
    }
    if windows.is_empty() {
        windows.push(capture_window(state, "main"));
    }
    SessionSnapshot {
        version: SESSION_FORMAT_VERSION,
        windows,
        tab_inputs: Vec::new(),
        active_index: 0,
    }
}

pub fn persist_session_if_enabled(state: &AppState, app: &AppHandle) {
    let prefs = load_preferences();
    if !prefs.restore_session_on_launch {
        return;
    }
    let snap = capture_all_windows(state, app);
    let _ = save_session_snapshot(&snap);
}

pub fn set_restore_session_enabled(enabled: bool) -> Result<(), String> {
    let mut prefs = load_preferences();
    prefs.restore_session_on_launch = enabled;
    save_preferences(&prefs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_v2_roundtrip_serde() {
        let snap = SessionSnapshot {
            version: SESSION_FORMAT_VERSION,
            windows: vec![WindowSessionEntry {
                label: "main".into(),
                tab_inputs: vec!["file:///tmp/a".into(), "https://example.com".into()],
                active_index: 1,
                geometry: None,
            }],
            tab_inputs: vec![],
            active_index: 0,
        };
        let json = serde_json::to_string(&snap).expect("json");
        let back: SessionSnapshot = serde_json::from_str(&json).expect("parse");
        assert_eq!(back.windows.len(), 1);
        assert_eq!(back.windows[0].active_index, 1);
        assert_eq!(back.windows[0].tab_inputs.len(), 2);
    }

    #[test]
    fn multi_window_entry_labels() {
        let snap = SessionSnapshot {
            version: SESSION_FORMAT_VERSION,
            windows: vec![
                WindowSessionEntry {
                    label: "main".into(),
                    tab_inputs: vec!["file:///a".into()],
                    active_index: 0,
                    geometry: None,
                },
                WindowSessionEntry {
                    label: "browser-2".into(),
                    tab_inputs: vec!["https://example.com".into()],
                    active_index: 0,
                    geometry: None,
                },
            ],
            tab_inputs: vec![],
            active_index: 0,
        };
        let wins = snap.normalized_windows();
        assert_eq!(wins.len(), 2);
        assert_eq!(wins[1].label, "browser-2");
    }

    #[test]
    fn migrates_legacy_snapshot() {
        let legacy = SessionSnapshot {
            version: 0,
            windows: vec![],
            tab_inputs: vec!["file:///tmp/a".into()],
            active_index: 1,
        };
        let wins = legacy.normalized_windows();
        assert_eq!(wins.len(), 1);
        assert_eq!(wins[0].label, "main");
        assert_eq!(wins[0].active_index, 1);
    }
}
