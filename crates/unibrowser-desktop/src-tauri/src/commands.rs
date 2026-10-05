use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};
use unibrowser_engine::palette::search_commands;
use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

use crate::downloads::apply_download_disposition;
use crate::state::{AppState, list_dir};
use crate::webview::{content_webview_navigate, sync_content_bounds};

#[derive(Serialize)]
pub struct TabDto {
    pub id: u64,
    pub label: String,
    pub location_kind: String,
    pub is_directory: bool,
    pub is_http: bool,
}

#[derive(Serialize)]
pub struct ToolbarCommandDto {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct PaletteEntryDto {
    pub command_id: String,
    pub title: String,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct TabContentDto {
    pub tabs: Vec<TabDto>,
    pub active_index: usize,
    pub omnibox: String,
    pub mods_loaded: String,
    pub status: String,
    pub directory_entries: Option<Vec<base_files::DirEntry>>,
    pub document_status: Option<String>,
}

fn location_to_omnibox(loc: &BrowserLocation) -> String {
    match loc {
        BrowserLocation::File { path } => format!("file://{}", path.display()),
        BrowserLocation::Http { url } => url.clone(),
        BrowserLocation::Internal { page } => format!("unibrowser://{}", page),
    }
}

fn build_tab_dto(tab: &unibrowser_engine::tabs::Tab) -> TabDto {
    let (kind, is_http) = match &tab.location {
        BrowserLocation::File { .. } => ("file", false),
        BrowserLocation::Http { .. } => ("http", true),
        BrowserLocation::Internal { .. } => ("internal", false),
    };
    TabDto {
        id: tab.id.0,
        label: tab.location.display_label(),
        location_kind: kind.into(),
        is_directory: tab.is_directory,
        is_http,
    }
}

fn snapshot(state: &AppState) -> Result<TabContentDto, String> {
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let active = tabs.active_tab().ok_or("no active tab")?;
    let omnibox = location_to_omnibox(&active.location);
    let directory_entries = if active.is_directory {
        if let BrowserLocation::File { path } = &active.location {
            Some(list_dir(path)?)
        } else {
            None
        }
    } else {
        None
    };
    let document_status = active.document.as_ref().map(|d| d.backend.status_line());
    Ok(TabContentDto {
        tabs: tabs.tabs().iter().map(build_tab_dto).collect(),
        active_index: tabs.active_index(),
        omnibox,
        mods_loaded: state.mods_loaded.clone(),
        status: String::new(),
        directory_entries,
        document_status,
    })
}

#[tauri::command]
pub fn get_browser_state(state: State<AppState>) -> Result<TabContentDto, String> {
    snapshot(&state)
}

#[tauri::command]
pub fn new_tab(state: State<AppState>) -> Result<TabContentDto, String> {
    let loc = BrowserLocation::Internal {
        page: format!("files?path={}", state.cwd.display()),
    };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = is_dir;
        tab.document = doc;
    }
    snapshot(&state)
}

#[tauri::command]
pub fn select_tab(
    index: usize,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    tabs.set_active(index);
    let loc = tabs
        .active_tab()
        .map(|t| t.location.clone())
        .ok_or("no tab")?;
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

#[tauri::command]
pub fn navigate_omnibox(input: String, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let loc = state.navigate_input(&input)?;
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

#[tauri::command]
pub fn open_path(path: String, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let loc = BrowserLocation::File {
        path: PathBuf::from(path),
    };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    tabs.navigate_active(loc.clone(), is_dir, doc);
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

#[tauri::command]
pub fn go_back(app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    if let Some(loc) = tabs.go_back() {
        let (loc, is_dir, doc) = state.resolve_location(loc);
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
        update_content_webview(&app, &loc)?;
    }
    snapshot(&state)
}

#[tauri::command]
pub fn go_forward(app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    if let Some(loc) = tabs.go_forward() {
        let (loc, is_dir, doc) = state.resolve_location(loc);
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
        update_content_webview(&app, &loc)?;
    }
    snapshot(&state)
}

#[tauri::command]
pub fn reload_tab(app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    if let Some(loc) = tabs.reload_location() {
        update_content_webview(&app, &loc)?;
    }
    snapshot(&state)
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<base_files::DirEntry>, String> {
    list_dir(PathBuf::from(path).as_path())
}

#[tauri::command]
pub fn get_toolbar_commands(state: State<AppState>) -> Result<Vec<ToolbarCommandDto>, String> {
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no tab")?;
    let doc = tab.document.as_ref().ok_or("no document for commands")?;
    let ids = [
        unibrowser_mod_api::commands::FONT_SIZE_INCREASE,
        unibrowser_mod_api::commands::FONT_SIZE_DECREASE,
    ];
    let out = ids
        .iter()
        .map(|id| {
            let meta = state.registry.command_meta(id);
            let en = state.registry.enablement(id, doc);
            ToolbarCommandDto {
                id: id.to_string(),
                title: meta.map(|m| m.title.to_string()).unwrap_or_else(|| id.to_string()),
                enabled: en.is_enabled(),
                reason: en.disabled_reason().map(String::from),
            }
        })
        .collect();
    Ok(out)
}

#[tauri::command]
pub fn execute_command(command_id: String, state: State<AppState>) -> Result<String, String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("command needs an open document")?;
    state.registry.execute(&command_id, doc)
}

#[tauri::command]
pub fn palette_search(query: String, state: State<AppState>) -> Result<Vec<PaletteEntryDto>, String> {
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no tab")?;
    let ctx = tab
        .document
        .as_ref()
        .map(|d| d.host_context())
        .unwrap_or_else(|| unibrowser_mod_api::HostContext {
            mode: unibrowser_mod_api::Mode::Browse,
            doc_type: String::new(),
            backend_id: String::new(),
            selection: unibrowser_mod_api::SelectionSnapshot::default(),
            capabilities: unibrowser_mod_api::Capabilities::default(),
        });
    Ok(search_commands(&state.registry, &ctx, &query)
        .into_iter()
        .map(|e| PaletteEntryDto {
            command_id: e.command_id,
            title: e.title,
            enabled: e.enablement.is_enabled(),
            reason: e.enablement.disabled_reason().map(String::from),
        })
        .collect())
}

#[tauri::command]
pub fn handle_download_url(
    url: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let disposition = apply_download_disposition(&url, None);
    match disposition {
        DownloadDisposition::OpenInNewTab { suggested_location } => {
            let (loc, is_dir, doc) = state.resolve_location(suggested_location);
            let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
            tabs.open_tab(loc.clone());
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
            }
            update_content_webview(&app, &loc)?;
            snapshot(&state)
        }
        DownloadDisposition::SaveToDisk { .. } => Err("Save to disk not implemented in MVP".into()),
    }
}

#[tauri::command]
pub fn set_content_bounds(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    app: AppHandle,
) -> Result<(), String> {
    sync_content_bounds(&app, x, y, width, height)
}

fn update_content_webview(app: &AppHandle, loc: &BrowserLocation) -> Result<(), String> {
    match loc {
        BrowserLocation::Http { url } => {
            content_webview_navigate(app, url)?;
        }
        _ => {
            content_webview_navigate(app, "about:blank")?;
        }
    }
    Ok(())
}
