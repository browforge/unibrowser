//! Last tab session snapshot (restored only when preference opt-in).

use serde::{Deserialize, Serialize};
use std::fs;
use crate::config::session_file_path;
use crate::preferences::{load_preferences, save_preferences};
use crate::state::AppState;
use unibrowser_mod_api::BrowserLocation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSnapshot {
    pub tab_inputs: Vec<String>,
    pub active_index: usize,
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

pub fn capture_from_state(state: &AppState, window_label: &str) -> SessionSnapshot {
    let session = state.session(window_label).ok();
    let Some(sess) = session else {
        return SessionSnapshot {
            tab_inputs: Vec::new(),
            active_index: 0,
        };
    };
    let tabs = match sess.tabs.lock() {
        Ok(t) => t,
        Err(_) => {
            return SessionSnapshot {
                tab_inputs: Vec::new(),
                active_index: 0,
            };
        }
    };
    SessionSnapshot {
        tab_inputs: tabs
            .tabs()
            .iter()
            .map(|t| location_to_omnibox(&t.location))
            .collect(),
        active_index: tabs.active_index(),
    }
}

pub fn location_to_omnibox(loc: &BrowserLocation) -> String {
    match loc {
        BrowserLocation::File { path } => format!("file://{}", path.display()),
        BrowserLocation::Http { url } => url.clone(),
        BrowserLocation::Internal { page } => format!("unibrowser://{}", page),
    }
}

pub fn persist_session_if_enabled(state: &AppState, window_label: &str) {
    let prefs = load_preferences();
    if !prefs.restore_session_on_launch {
        return;
    }
    let snap = capture_from_state(state, window_label);
    let _ = save_session_snapshot(&snap);
}

pub fn set_restore_session_enabled(enabled: bool) -> Result<(), String> {
    let mut prefs = load_preferences();
    prefs.restore_session_on_launch = enabled;
    save_preferences(&prefs)
}
