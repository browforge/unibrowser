//! Floating Command Manager bars in separate Tauri windows (multi-monitor).

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use unibrowser_engine::{ToolbarAnchor, ToolbarPresentation, save_command_manager_file};

use crate::config::toolbars_file_path;
use crate::state::AppState;

pub fn toolbar_window_label(bar_id: &str) -> String {
    format!("toolbar-{}", bar_id)
}

pub fn is_toolbar_window_label(label: &str) -> bool {
    label.starts_with("toolbar-")
}

pub fn bar_id_from_toolbar_window(label: &str) -> Option<&str> {
    label.strip_prefix("toolbar-")
}

pub fn clamp_toolbar_screen_position(app: &AppHandle, x: f64, y: f64) -> (f64, f64) {
    let win_w = 480.0;
    let win_h = 88.0;
    let monitor = app.primary_monitor().ok().flatten();
    let (mx, my, mw, mh) = if let Some(m) = monitor {
        let pos = m.position();
        let size = m.size();
        (
            pos.x as f64,
            pos.y as f64,
            size.width as f64,
            size.height as f64,
        )
    } else {
        (0.0, 0.0, 1920.0, 1080.0)
    };
    let max_x = (mx + mw - win_w).max(mx);
    let max_y = (my + mh - win_h).max(my);
    (x.clamp(mx, max_x), y.clamp(my, max_y))
}

pub fn ensure_toolbar_window(app: &AppHandle, bar_id: &str, x: f64, y: f64) -> Result<(), String> {
    let (x, y) = clamp_toolbar_screen_position(app, x, y);
    let label = toolbar_window_label(bar_id);
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title("Unibrowser commands")
        .inner_size(480.0, 88.0)
        .min_inner_size(200.0, 56.0)
        .decorations(true)
        .resizable(true)
        .position(x, y)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn close_toolbar_window(app: &AppHandle, bar_id: &str) {
    let label = toolbar_window_label(bar_id);
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.close();
    }
}

pub fn close_all_toolbar_windows(app: &AppHandle, state: &AppState) {
    let cm = match state.command_manager.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    for bar in &cm.floating_bars {
        if bar.presentation == ToolbarPresentation::DetachedWindow {
            close_toolbar_window(app, &bar.id);
        }
    }
}

fn persist_layout(state: &AppState) -> Result<(), String> {
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, &cm.to_file()).map_err(|e| e.to_string())
}

pub fn set_bar_presentation(
    state: &AppState,
    bar_id: &str,
    presentation: ToolbarPresentation,
) -> Result<(), String> {
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let bar = cm
        .floating_bars
        .iter_mut()
        .find(|b| b.id == bar_id)
        .ok_or_else(|| format!("toolbar not found: {}", bar_id))?;
    bar.presentation = presentation;
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, &cm.to_file()).map_err(|e| e.to_string())
}

pub fn restore_detached_toolbars(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    for bar in &cm.floating_bars {
        if bar.presentation != ToolbarPresentation::DetachedWindow {
            continue;
        }
        if bar.anchor != ToolbarAnchor::Floating {
            continue;
        }
        ensure_toolbar_window(app, &bar.id, bar.x, bar.y)?;
    }
    Ok(())
}

pub fn dock_bar_on_window_close(app: &AppHandle, state: &AppState, label: &str) -> Result<(), String> {
    let bar_id = bar_id_from_toolbar_window(label).ok_or("not a toolbar window")?;
    set_bar_presentation(state, bar_id, ToolbarPresentation::InWindow)?;
    close_toolbar_window(app, bar_id);
    Ok(())
}
