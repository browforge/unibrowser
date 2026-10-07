//! Floating Command Manager bars in separate Tauri windows (multi-monitor).

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use unibrowser_engine::{ToolbarAnchor, ToolbarPresentation, save_command_manager_file};

use crate::config::toolbars_file_path;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorWorkArea {
    pub name: String,
    pub edid: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn toolbar_window_label(bar_id: &str) -> String {
    format!("toolbar-{}", bar_id.replace('.', "__dot__"))
}

pub fn is_toolbar_window_label(label: &str) -> bool {
    label.starts_with("toolbar-")
}

pub fn bar_id_from_toolbar_window(label: &str) -> Option<String> {
    label
        .strip_prefix("toolbar-")
        .map(|id| id.replace("__dot__", "."))
}

pub fn recover_toolbar_position(
    monitors: &[MonitorWorkArea],
    monitor_name: Option<&str>,
    monitor_edid: Option<&str>,
    x: f64,
    y: f64,
) -> (String, Option<String>, f64, f64) {
    let win_w = 480.0;
    let win_h = 88.0;
    let fallback = MonitorWorkArea {
        name: "primary".into(),
        edid: None,
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    let monitor = monitors
        .iter()
        .find(|m| {
            monitor_edid.is_some() && m.edid.as_deref() == monitor_edid
                || monitor_name.is_some() && Some(m.name.as_str()) == monitor_name
        })
        .or_else(|| {
            monitors.iter().min_by(|a, b| {
                let distance = |m: &MonitorWorkArea| {
                    let cx = m.x + m.width / 2.0;
                    let cy = m.y + m.height / 2.0;
                    (cx - x).powi(2) + (cy - y).powi(2)
                };
                distance(a).total_cmp(&distance(b))
            })
        })
        .unwrap_or(&fallback);
    let max_x = (monitor.x + monitor.width - win_w).max(monitor.x);
    let max_y = (monitor.y + monitor.height - win_h).max(monitor.y);
    (
        monitor.name.clone(),
        monitor.edid.clone(),
        x.clamp(monitor.x, max_x),
        y.clamp(monitor.y, max_y),
    )
}

fn monitor_work_areas(app: &AppHandle) -> Vec<MonitorWorkArea> {
    app.available_monitors()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| {
            let pos = monitor.position();
            let size = monitor.size();
            MonitorWorkArea {
                name: monitor
                    .name()
                    .map(String::from)
                    .unwrap_or_else(|| format!("monitor-{}", index)),
                edid: None,
                x: pos.x as f64,
                y: pos.y as f64,
                width: size.width as f64,
                height: size.height as f64,
            }
        })
        .collect()
}

pub fn recover_bar_geometry(
    app: &AppHandle,
    monitor_name: Option<&str>,
    monitor_edid: Option<&str>,
    x: f64,
    y: f64,
) -> (String, Option<String>, f64, f64) {
    recover_toolbar_position(
        &monitor_work_areas(app),
        monitor_name,
        monitor_edid,
        x,
        y,
    )
}

pub fn clamp_toolbar_screen_position(app: &AppHandle, x: f64, y: f64) -> (f64, f64) {
    let (_, _, x, y) = recover_toolbar_position(&monitor_work_areas(app), None, None, x, y);
    (x, y)
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
    let url = format!("index.html?toolbar={}", label);
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
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

/// Dock every detached bar and close its window so settings content stays reachable.
pub fn dock_all_detached_toolbars(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let ids: Vec<String> = {
        let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
        cm.floating_bars
            .iter()
            .filter(|bar| bar.presentation == ToolbarPresentation::DetachedWindow)
            .map(|bar| bar.id.clone())
            .collect()
    };
    for bar_id in ids {
        set_bar_presentation(state, &bar_id, ToolbarPresentation::InWindow)?;
        close_toolbar_window(app, &bar_id);
    }
    Ok(())
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
    let monitors = monitor_work_areas(app);
    let mut recovered = Vec::new();
    {
        let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
        for bar in &mut cm.floating_bars {
            if bar.presentation != ToolbarPresentation::DetachedWindow {
                continue;
            }
            if bar.anchor != ToolbarAnchor::Floating {
                continue;
            }
            let (name, edid, x, y) = recover_toolbar_position(
                &monitors,
                bar.monitor_name.as_deref(),
                bar.monitor_edid.as_deref(),
                bar.x,
                bar.y,
            );
            bar.monitor_name = Some(name);
            bar.monitor_edid = edid;
            bar.x = x;
            bar.y = y;
            recovered.push((bar.id.clone(), x, y));
        }
    }
    persist_layout(state)?;
    for (bar_id, x, y) in recovered {
        ensure_toolbar_window(app, &bar_id, x, y)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_monitor_recovers_to_nearest_visible_monitor_and_clamps() {
        let monitors = vec![
            MonitorWorkArea {
                name: "left".into(),
                edid: Some("A".into()),
                x: 0.0,
                y: 0.0,
                width: 1280.0,
                height: 720.0,
            },
            MonitorWorkArea {
                name: "right".into(),
                edid: Some("B".into()),
                x: 1280.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
        ];
        let (name, edid, x, y) =
            recover_toolbar_position(&monitors, Some("missing"), Some("gone"), 5000.0, -50.0);
        assert_eq!(name, "right");
        assert_eq!(edid.as_deref(), Some("B"));
        assert_eq!(x, 2720.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn toolbar_window_label_roundtrips_dotted_contribution_id() {
        let id = "cmd-proving.detached-markdown";
        let label = toolbar_window_label(id);
        assert_eq!(label, "toolbar-cmd-proving__dot__detached-markdown");
        assert_eq!(bar_id_from_toolbar_window(&label).as_deref(), Some(id));
    }
}

pub fn dock_bar_on_window_close(app: &AppHandle, state: &AppState, label: &str) -> Result<(), String> {
    let bar_id = bar_id_from_toolbar_window(label).ok_or("not a toolbar window")?;
    set_bar_presentation(state, &bar_id, ToolbarPresentation::InWindow)?;
    close_toolbar_window(app, &bar_id);
    Ok(())
}
