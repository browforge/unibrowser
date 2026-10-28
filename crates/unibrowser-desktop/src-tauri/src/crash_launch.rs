//! Recovery banner items after session restore (no modal, no auto-apply).

use serde::{Deserialize, Serialize};

use crate::autosave::apply_saved_content;
use crate::commands::{update_content_webview, ws_label};
use crate::edit_recovery;
use crate::state::AppState;
use tauri::AppHandle;
use unibrowser_mod_api::BrowserLocation;

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

pub fn recovery_bar_items_for_window(
    state: &AppState,
    window_label: &str,
) -> Result<Vec<RecoveryBarItemDto>, String> {
    let _sess = ws_label(state, window_label)?;
    let tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
    recovery_bar_items_with_tabs(state, &tabs)
}

pub fn recovery_bar_items_with_tabs(
    state: &AppState,
    tabs: &unibrowser_engine::TabManager,
) -> Result<Vec<RecoveryBarItemDto>, String> {
    let mut items = Vec::new();
    for row in edit_recovery::list_pending() {
        if !tab_open_for_path_locked(tabs, state, &row.path)? {
            continue;
        }
        items.push(RecoveryBarItemDto {
            backup_id: row.journal_id,
            path: row.path,
            backend_id: row.backend_id,
        });
    }
    for row in crate::crash_recovery::list_backups() {
        if !tab_open_for_path_locked(tabs, state, &row.path)? {
            continue;
        }
        items.push(RecoveryBarItemDto {
            backup_id: row.backup_id,
            path: row.path,
            backend_id: row.backend_id,
        });
    }
    Ok(items)
}

pub fn discover_recovery_bar_items(
    state: &AppState,
    window_label: &str,
) -> Result<Vec<RecoveryBarItemDto>, String> {
    let items = recovery_bar_items_for_window(state, window_label)?;
    if let Ok(mut bar) = ws_label(state, window_label)?.recovery_bar.lock() {
        bar.items = items.clone();
    }
    Ok(items)
}

pub fn apply_crash_backups_after_session(
    state: &AppState,
    app: &AppHandle,
    window_label: &str,
) -> Result<Vec<RecoveryBarItemDto>, String> {
    let _ = app;
    discover_recovery_bar_items(state, window_label)
}

pub fn restore_recovery_item(
    state: &AppState,
    window_label: &str,
    app: &AppHandle,
    backup_id: &str,
) -> Result<(), String> {
    if backup_id.len() == 16 && backup_id.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Ok(rec) = edit_recovery::load_recovered(backup_id) {
            apply_recovered_edit(state, window_label, app, &rec)?;
            dismiss_recovery_item(backup_id)?;
            if let Ok(sess) = ws_label(state, window_label) {
                if let Ok(mut bar) = sess.recovery_bar.lock() {
                    bar.items.retain(|i| i.backup_id != backup_id);
                }
            }
            return Ok(());
        }
    }
    let record = crate::crash_recovery::load_backup(backup_id)?;
    apply_backup_record(state, window_label, app, &record)?;
    dismiss_recovery_item(backup_id)?;
    if let Ok(sess) = ws_label(state, window_label) {
        if let Ok(mut bar) = sess.recovery_bar.lock() {
            bar.items.retain(|i| i.backup_id != backup_id);
        }
    }
    Ok(())
}

fn apply_recovered_edit(
    state: &AppState,
    window_label: &str,
    app: &AppHandle,
    rec: &unibrowser_document_model::RecoveredEdit,
) -> Result<(), String> {
    let disk = edit_recovery::read_disk_bytes(&rec.header.path, &state.cwd)?;
    let content = String::from_utf8_lossy(&rec.content).into_owned();
    let disk_text = String::from_utf8_lossy(&disk).into_owned();
    let file_path = crate::path_security::resolve_open_path(&rec.header.path, &state.cwd)?;
    let _sess = ws_label(state, window_label)?;
    let loc = {
        let mut tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
        let index = tabs.tabs().iter().position(|t| {
            match &t.location {
                BrowserLocation::File { path } => {
                    crate::path_security::resolve_open_path(&path.to_string_lossy(), &state.cwd)
                        .ok()
                        == Some(file_path.clone())
                }
                _ => false,
            }
        });
        let Some(index) = index else {
            return Err("document tab not open".into());
        };
        tabs.set_active(index);
        let tab = tabs.active_tab_mut().ok_or("tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        if doc.path != rec.header.path {
            return Err("path mismatch".into());
        }
        let before = current_content_string(doc)?;
        apply_saved_content(doc, &rec.header.backend_id, &content)?;
        let mut stacks = _sess.undo_stacks.lock().map_err(|e| e.to_string())?;
        if before != content {
            stacks.record_change(&rec.header.path, before, &content);
        } else if disk_text != content {
            stacks.record_change(&rec.header.path, disk_text, &content);
        }
        tab.location.clone()
    };
    update_content_webview(app, window_label, &loc)?;
    Ok(())
}

fn apply_backup_record(
    state: &AppState,
    window_label: &str,
    app: &AppHandle,
    record: &crate::crash_recovery::BackupRecord,
) -> Result<(), String> {
    let file_path = crate::path_security::resolve_open_path(&record.path, &state.cwd)?;
    let _sess = ws_label(state, window_label)?;
    let loc = {
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
            return Err("document tab not open".into());
        };
        tabs.set_active(index);
        let tab = tabs.active_tab_mut().ok_or("tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        if doc.path != record.path {
            return Err("path mismatch".into());
        }
        let disk = edit_recovery::read_disk_bytes(&record.path, &state.cwd)?;
        let disk_text = String::from_utf8_lossy(&disk).into_owned();
        let before = current_content_string(doc)?;
        apply_saved_content(doc, &record.backend_id, &record.content)?;
        let mut stacks = _sess.undo_stacks.lock().map_err(|e| e.to_string())?;
        if before != record.content {
            stacks.record_change(&record.path, before, &record.content);
        } else if disk_text != record.content {
            stacks.record_change(&record.path, disk_text, &record.content);
        }
        tab.location.clone()
    };
    update_content_webview(app, window_label, &loc)?;
    Ok(())
}

fn current_content_string(
    doc: &mut unibrowser_engine::document::DocumentSession,
) -> Result<String, String> {
    use base_js::BACKEND_ID as JS;
    use base_svg::BACKEND_ID as SVG;
    use base_text::BACKEND_ID as TEXT;
    let backend_id = doc.backend.backend_id();
    if backend_id == TEXT {
        let b = base_text::try_text_backend(&mut *doc.backend).ok_or("not text")?;
        Ok(b.content.clone())
    } else if backend_id == JS {
        let b = base_js::try_js_backend(&mut *doc.backend).ok_or("not js")?;
        Ok(b.content.clone())
    } else if backend_id == SVG {
        let b = base_svg::try_svg_backend(&mut *doc.backend).ok_or("not svg")?;
        Ok(b.content.clone())
    } else {
        Err("unsupported backend for recovery".into())
    }
}

fn tab_open_for_path_locked(
    tabs: &unibrowser_engine::TabManager,
    state: &AppState,
    doc_path: &str,
) -> Result<bool, String> {
    let file_path = crate::path_security::resolve_open_path(doc_path, &state.cwd)?;
    Ok(tabs.tabs().iter().any(|t| {
        match &t.location {
            BrowserLocation::File { path } => {
                crate::path_security::resolve_open_path(&path.to_string_lossy(), &state.cwd).ok()
                    == Some(file_path.clone())
            }
            _ => false,
        }
    }))
}

fn tab_open_for_path(
    state: &AppState,
    window_label: &str,
    doc_path: &str,
) -> Result<bool, String> {
    let _sess = ws_label(state, window_label)?;
    let tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
    tab_open_for_path_locked(&tabs, state, doc_path)
}

pub fn dismiss_recovery_item(backup_id: &str) -> Result<(), String> {
    if backup_id.len() == 16 && backup_id.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Ok(rec) = edit_recovery::load_recovered(backup_id) {
            edit_recovery::clear_for_path(&rec.header.path);
            if let Some(dir) = edit_recovery::journal_dir() {
                let preview = dir.join(format!("{}.preview", backup_id));
                if preview.is_file() {
                    let _ = std::fs::remove_file(preview);
                }
            }
            return Ok(());
        }
    }
    crate::crash_recovery::dismiss_backup(backup_id)
}
