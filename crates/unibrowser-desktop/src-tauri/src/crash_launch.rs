//! Apply crash-recovery backups when session tabs reopen (no modal).

use serde::{Deserialize, Serialize};
use unibrowser_mod_api::BrowserLocation;

use crate::autosave::apply_saved_content;
use crate::commands::{update_content_webview, ws_label};
use crate::crash_recovery::{list_backups, load_backup, BackupRecord};
use crate::state::AppState;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryBarItemDto {
    pub backup_id: String,
    pub path: String,
    pub backend_id: String,
}

#[derive(Default)]
pub struct RecoveryBarState {
    pub items: Vec<RecoveryBarItemDto>,
}

pub fn apply_crash_backups_after_session(
    state: &AppState,
    app: &AppHandle,
    window_label: &str,
) -> Result<Vec<RecoveryBarItemDto>, String> {
    let backups = list_backups();
    if backups.is_empty() {
        return Ok(Vec::new());
    }
    let _sess = ws_label(state, window_label)?;
    let mut applied = Vec::new();
    for row in backups {
        let record = load_backup(&row.backup_id)?;
        if !apply_backup_to_open_tab(state, window_label, app, &record)? {
            continue;
        }
        applied.push(RecoveryBarItemDto {
            backup_id: row.backup_id,
            path: record.path,
            backend_id: record.backend_id,
        });
    }
    if let Ok(mut bar) = _sess.recovery_bar.lock() {
        bar.items = applied.clone();
    }
    Ok(applied)
}

fn apply_backup_to_open_tab(
    state: &AppState,
    window_label: &str,
    app: &AppHandle,
    record: &BackupRecord,
) -> Result<bool, String> {
    let file_path = crate::path_security::resolve_open_path(&record.path, &state.cwd)?;
    let _sess = ws_label(state, window_label)?;
    let mut tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
    let index = tabs.tabs().iter().position(|t| {
        match &t.location {
            BrowserLocation::File { path } => {
                crate::path_security::resolve_open_path(&path.to_string_lossy(), &state.cwd).ok()
                    == Some(file_path.clone())
            }
            _ => false,
        }
    });
    let Some(index) = index else {
        return Ok(false);
    };
    tabs.set_active(index);
    let tab = tabs.active_tab_mut().ok_or("tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    if doc.path != record.path {
        return Ok(false);
    }
    apply_saved_content(doc, &record.backend_id, &record.content)?;
    let loc = tab.location.clone();
    update_content_webview(app, window_label, &loc)?;
    Ok(true)
}
