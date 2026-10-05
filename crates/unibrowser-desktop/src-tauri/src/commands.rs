use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};
use unibrowser_engine::{
    keybinding::COMMAND_PALETTE_TOGGLE,
    palette::search_commands,
    KeyChord,
};
use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

use base_text::{TextBackend, try_text_backend};

use crate::config::toolbars_file_path;
use crate::downloads::apply_download_disposition;
use crate::state::{AppState, list_dir};
use crate::webview::{content_webview_navigate, sync_content_bounds};
use unibrowser_engine::{
    host_context_for_browse, save_command_manager_file, CommandManagerFile, ToolbarPhase,
};

#[derive(Serialize)]
pub struct TabDto {
    pub id: u64,
    pub label: String,
    pub location_kind: String,
    pub is_directory: bool,
    pub is_http: bool,
}

#[derive(Serialize)]
pub struct UiCommandDto {
    pub command_id: String,
    pub title: String,
    pub icon: String,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct ManagerTabDto {
    pub id: String,
    pub label: String,
    pub phase: String,
    pub selected: bool,
    pub commands: Vec<UiCommandDto>,
}

#[derive(Serialize)]
pub struct FloatingBarDto {
    pub id: String,
    pub label: String,
    pub anchor: String,
    pub x: f64,
    pub y: f64,
    pub commands: Vec<UiCommandDto>,
}

#[derive(Serialize)]
pub struct CommandManagerDto {
    pub auto_context: String,
    pub selected_tab: String,
    pub tabs: Vec<ManagerTabDto>,
    pub floating_bars: Vec<FloatingBarDto>,
}

#[derive(Serialize)]
pub struct PaletteEntryDto {
    pub command_id: String,
    pub title: String,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct TextDocumentDto {
    pub path: String,
    pub content: String,
    pub font_size_pt: f32,
    pub language: String,
    pub dirty: bool,
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
    pub text_document: Option<TextDocumentDto>,
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

fn text_document_dto(doc: &mut unibrowser_engine::DocumentSession) -> Option<TextDocumentDto> {
    let backend = try_text_backend(&mut *doc.backend)?;
    Some(TextDocumentDto {
        path: backend.path.clone(),
        content: backend.content.clone(),
        font_size_pt: backend.font_size_pt,
        language: TextBackend::language_id(&backend.path).into(),
        dirty: backend.dirty,
    })
}

fn snapshot(state: &AppState) -> Result<TabContentDto, String> {
    state.refresh_command_manager_context();
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let omnibox = tabs
        .active_tab()
        .map(|t| location_to_omnibox(&t.location))
        .ok_or("no active tab")?;
    let directory_entries = if let Some(active) = tabs.active_tab() {
        if active.is_directory {
            if let BrowserLocation::File { path } = &active.location {
                Some(list_dir(path)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let document_status = tabs
        .active_tab()
        .and_then(|t| t.document.as_ref())
        .map(|d| d.backend.status_line());
    let text_document = tabs
        .active_tab_mut()
        .and_then(|t| t.document.as_mut())
        .and_then(text_document_dto);
    Ok(TabContentDto {
        tabs: tabs.tabs().iter().map(build_tab_dto).collect(),
        active_index: tabs.active_index(),
        omnibox,
        mods_loaded: state.mods_loaded.clone(),
        status: document_status
            .clone()
            .unwrap_or_else(|| format!("{} for command palette", state.keybindings.palette_chord_display())),
        directory_entries,
        document_status,
        text_document,
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
    let loc = {
        let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
        tabs.reload_location()
    };
    if let Some(loc) = loc {
        let (loc, is_dir, doc) = state.resolve_location(loc);
        let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
        update_content_webview(&app, &loc)?;
    }
    snapshot(&state)
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<base_files::DirEntry>, String> {
    list_dir(PathBuf::from(path).as_path())
}

fn phase_name(phase: ToolbarPhase) -> &'static str {
    match phase {
        ToolbarPhase::Active => "active",
        ToolbarPhase::Planned => "planned",
    }
}

fn map_ui_commands(cmds: Vec<unibrowser_engine::ResolvedUiCommand>) -> Vec<UiCommandDto> {
    cmds.into_iter()
        .map(|c| UiCommandDto {
            command_id: c.command_id,
            title: c.title,
            icon: c.icon,
            enabled: c.enabled,
            reason: c.reason,
        })
        .collect()
}

#[tauri::command]
pub fn get_command_manager(state: State<AppState>) -> Result<CommandManagerDto, String> {
    state.refresh_command_manager_context();
    let tabs_guard = state.tabs.lock().map_err(|e| e.to_string())?;
    let active = tabs_guard.active_tab();
    let host = active
        .and_then(|t| t.document.as_ref())
        .map(|d| d.host_context())
        .unwrap_or_else(host_context_for_browse);
    let doc_ref = active.and_then(|t| t.document.as_ref());
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let selected = cm.selected_tab.clone();
    let auto_context = cm.auto_context.tab_id().to_string();
    let tab_dtos = cm
        .tabs
        .iter()
        .map(|tab| {
            let commands = map_ui_commands(
                cm.resolve_tab_commands(&state.registry, &host, doc_ref, &tab.id),
            );
            ManagerTabDto {
                id: tab.id.clone(),
                label: tab.label.clone(),
                phase: phase_name(tab.phase).into(),
                selected: tab.id == selected,
                commands,
            }
        })
        .collect();
    let floating_bars = cm
        .floating_bars
        .iter()
        .map(|bar| {
            let commands = map_ui_commands(
                cm.resolve_floating_bar(&state.registry, &host, doc_ref, &bar.id)
                    .unwrap_or_default(),
            );
            FloatingBarDto {
                id: bar.id.clone(),
                label: bar.label.clone(),
                anchor: match bar.anchor {
                    unibrowser_engine::ToolbarAnchor::Docked => "docked",
                    unibrowser_engine::ToolbarAnchor::Floating => "floating",
                }
                .into(),
                x: bar.x,
                y: bar.y,
                commands,
            }
        })
        .collect();
    Ok(CommandManagerDto {
        auto_context,
        selected_tab: selected,
        tabs: tab_dtos,
        floating_bars,
    })
}

#[tauri::command]
pub fn save_command_manager_layout(
    layout: CommandManagerFile,
    state: State<AppState>,
) -> Result<(), String> {
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, &layout)?;
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    *cm = unibrowser_engine::CommandManagerState::from_file(
        &layout,
        cm.auto_context,
    );
    Ok(())
}

#[tauri::command]
pub fn select_command_manager_tab(tab_id: String, state: State<AppState>) -> Result<(), String> {
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    if cm.tabs.iter().any(|t| t.id == tab_id) {
        cm.selected_tab = tab_id;
    }
    Ok(())
}

#[tauri::command]
pub fn execute_command(command_id: String, state: State<AppState>) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Ok("palette".into());
    }
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("command needs an open document")?;
    state.registry.execute(&command_id, doc)
}

#[tauri::command]
pub fn sync_text_document(
    content: String,
    selection_start: usize,
    selection_end: usize,
    state: State<AppState>,
) -> Result<(), String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no text document")?;
    let backend = try_text_backend(&mut *doc.backend).ok_or("not a text document")?;
    backend.set_content(content);
    backend.set_selection_range(selection_start, selection_end);
    Ok(())
}

#[derive(Serialize)]
pub struct KeybindingDto {
    pub command_id: String,
    pub chord: String,
}

#[derive(serde::Deserialize)]
pub struct KeyChordInput {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub key: String,
}

#[tauri::command]
pub fn get_keybindings(state: State<AppState>) -> Result<Vec<KeybindingDto>, String> {
    Ok(state
        .keybindings
        .bindings_for_display()
        .into_iter()
        .map(|(command_id, chord)| KeybindingDto {
            command_id,
            chord,
        })
        .collect())
}

/// Resolve a keyboard chord to a command id (e.g. `unibrowser.palette.toggle`).
#[tauri::command]
pub fn dispatch_key_chord(
    chord: KeyChordInput,
    state: State<AppState>,
) -> Result<Option<String>, String> {
    let key = KeyChord {
        ctrl: chord.ctrl,
        alt: chord.alt,
        shift: chord.shift,
        meta: chord.meta,
        key: chord.key.to_ascii_lowercase(),
    };
    Ok(state
        .keybindings
        .command_for_chord(&key)
        .map(|s| s.to_string()))
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
