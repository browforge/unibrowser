use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
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
use crate::mod_portal::{ModSuggestionDto, suggest_mods_for_unknown_path};
use crate::preferences::{load_preferences, save_preferences, WindowState};
use crate::webview::{content_webview_navigate, inject_page_font_scale, sync_content_bounds};
use unibrowser_mod_api::handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID};
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
    pub context_pinned: bool,
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
pub struct ProjectNodeDto {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub asset: Option<String>,
}

#[derive(Serialize)]
pub struct OpenFileTabDto {
    pub tab_index: usize,
    pub path: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct DocumentViewerDto {
    pub path: String,
    pub viewer: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_nodes: Option<Vec<ProjectNodeDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_file_tabs: Option<Vec<OpenFileTabDto>>,
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
    pub mod_portal: Option<ModPortalDto>,
    pub document_viewer: Option<DocumentViewerDto>,
    pub show_onboarding: bool,
}

#[derive(Serialize)]
pub struct ModPortalDto {
    pub title: String,
    pub message: String,
    pub suggestions: Vec<ModSuggestionDto>,
    pub sideload_hint: String,
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

fn open_file_tabs_dto(state: &AppState) -> Vec<OpenFileTabDto> {
    let tabs = state.tabs.lock().ok();
    let tabs = match tabs {
        Some(t) => t,
        None => return Vec::new(),
    };
    tabs
        .tabs()
        .iter()
        .enumerate()
        .filter_map(|(index, tab)| {
            if let BrowserLocation::File { path } = &tab.location {
                if path.is_file() && !unibrowser_project::is_unibrow_path(path) {
                    return Some(OpenFileTabDto {
                        tab_index: index,
                        path: path.to_string_lossy().into_owned(),
                        label: tab.location.display_label(),
                    });
                }
            }
            None
        })
        .collect()
}

fn document_viewer_dto(
    doc: &mut unibrowser_engine::DocumentSession,
    state: &AppState,
) -> Option<DocumentViewerDto> {
    if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
        let root = Path::new(&doc.path);
        let opened = unibrowser_project::open_package(root).ok();
        let nodes = opened.as_ref().map(|pkg| {
            pkg.manifest
                .nodes
                .iter()
                .map(|n| ProjectNodeDto {
                    id: n.id.clone(),
                    node_type: n.node_type.clone(),
                    label: n.label.clone(),
                    asset: n.asset.clone(),
                })
                .collect()
        });
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::project_backend::VIEWER_KIND.into(),
            title: opened
                .map(|p| p.manifest.title.clone())
                .unwrap_or_else(|| doc.backend.display_name().into()),
            pdf_scale: None,
            pdf_page: None,
            media_note: None,
            project_nodes: nodes,
            open_file_tabs: Some(open_file_tabs_dto(state)),
        });
    }
    let viewer = match doc.backend.backend_id() {
        base_docx::BACKEND_ID => base_docx::VIEWER_KIND,
        base_pdf::BACKEND_ID => base_pdf::VIEWER_KIND,
        base_dxf::BACKEND_ID => base_dxf::VIEWER_KIND,
        base_js::BACKEND_ID => base_js::VIEWER_KIND,
        base_video::BACKEND_ID => base_video::VIEWER_KIND,
        base_audio::BACKEND_ID => base_audio::VIEWER_KIND,
        _ => return None,
    };
    let (pdf_scale, pdf_page) = if doc.backend.backend_id() == base_pdf::BACKEND_ID {
        base_pdf::try_pdf_backend(&mut *doc.backend)
            .map(|b| (Some(b.render_scale), Some(b.page_index)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };
    let media_note = if doc.backend.backend_id() == base_video::BACKEND_ID
        || doc.backend.backend_id() == base_audio::BACKEND_ID
    {
        Some(doc.backend.status_line())
    } else {
        None
    };
    Some(DocumentViewerDto {
        path: doc.path.clone(),
        viewer: viewer.into(),
        title: doc.backend.display_name().into(),
        pdf_scale,
        pdf_page,
        media_note,
        project_nodes: None,
        open_file_tabs: None,
    })
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
    let (document_viewer, text_document) = match tabs.active_tab_mut() {
        Some(tab) => {
            let viewer = tab
                .document
                .as_mut()
                .and_then(|d| document_viewer_dto(d, state));
            let text = if viewer.is_some() {
                None
            } else {
                tab.document.as_mut().and_then(text_document_dto)
            };
            (viewer, text)
        }
        None => (None, None),
    };
    let mod_portal = active_mod_portal(tabs.active_tab());
    let prefs = load_preferences();
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
        mod_portal,
        document_viewer,
        show_onboarding: !prefs.onboarding_dismissed,
    })
}

fn active_mod_portal(tab: Option<&unibrowser_engine::tabs::Tab>) -> Option<ModPortalDto> {
    let tab = tab?;
    if let BrowserLocation::Internal { page } = &tab.location {
        if page.starts_with("mods") {
            let ext = page
                .strip_prefix("mods?ext=")
                .unwrap_or("");
            let suggestions = if ext.is_empty() {
                suggest_mods_for_unknown_path("example.unknown")
            } else {
                suggest_mods_for_unknown_path(&format!("file.{}", ext))
            };
            return Some(ModPortalDto {
                title: "Mod portal".into(),
                message: "Install support for formats you need. Catalog: Open VSX + native .ubmod (stub).".into(),
                suggestions,
                sideload_hint: "Install from file… (.vsix, .ubmod) — always allowed.".into(),
            });
        }
    }
    if let BrowserLocation::File { path } = &tab.location {
        if tab.document.is_none() && path.is_file() {
            let suggestions = suggest_mods_for_unknown_path(&path.to_string_lossy());
            if !suggestions.is_empty() {
                return Some(ModPortalDto {
                    title: "Install support for this file".into(),
                    message: format!("Unibrowser cannot open {} yet.", path.file_name().unwrap_or_default().to_string_lossy()),
                    suggestions,
                    sideload_hint: "Install from file… or browse Open VSX when online.".into(),
                });
            }
        }
    }
    None
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

fn allow_media_asset_path(app: &AppHandle, path: &Path) {
    if !path.is_file() {
        return;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let is_media = matches!(
        ext.as_str(),
        "mp4" | "webm" | "wav" | "mid" | "midi" | "mp3" | "ogg"
    );
    if !is_media {
        return;
    }
    let _ = app.asset_protocol_scope().allow_file(path);
}

#[tauri::command]
pub fn allow_asset_path(path: String, app: AppHandle) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.is_file() {
        return Err("not a file".into());
    }
    app.asset_protocol_scope()
        .allow_file(&p)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_path(path: String, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let file_path = PathBuf::from(&path);
    allow_media_asset_path(&app, &file_path);
    let loc = BrowserLocation::File { path: file_path };
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
        context_pinned: cm.is_context_pinned(),
        tabs: tab_dtos,
        floating_bars,
    })
}

#[tauri::command]
pub fn pin_command_manager_context(state: State<AppState>) -> Result<(), String> {
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    cm.pin_selected_tab();
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, &cm.to_file())?;
    Ok(())
}

#[tauri::command]
pub fn clear_command_manager_context_pin(state: State<AppState>) -> Result<(), String> {
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    cm.clear_context_pin();
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, &cm.to_file())?;
    Ok(())
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
pub fn execute_command(
    command_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Ok("palette".into());
    }
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    if tab.document.is_none() {
        return Err("command needs an open document".into());
    }
    let doc = tab.document.as_mut().unwrap();
    if doc.backend.backend_id() == base_web::BACKEND_ID
        && (command_id == FONT_INCREASE_ID || command_id == FONT_DECREASE_ID)
    {
        let delta = if command_id == FONT_INCREASE_ID { 1 } else { -1 };
        inject_page_font_scale(&app, delta)?;
        return Ok(format!("Page font scale {:+}px (user style stub)", delta));
    }
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

#[derive(Serialize)]
pub struct PreferencesDto {
    pub onboarding_dismissed: bool,
}

#[tauri::command]
pub fn read_document_bytes(path: String) -> Result<Vec<u8>, String> {
    let p = PathBuf::from(&path);
    if !p.is_file() {
        return Err("not a file".into());
    }
    std::fs::read(&p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_pdf_viewer_state(
    has_text_selection: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let pdf = base_pdf::try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    pdf.set_text_selected(has_text_selection);
    Ok(())
}

#[tauri::command]
pub fn sync_pdf_page(page: u32, state: State<AppState>) -> Result<(), String> {
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let pdf = base_pdf::try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    pdf.set_page_index(page);
    Ok(())
}

#[tauri::command]
pub async fn pick_unibrow_asset_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Add asset to project")
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn add_unibrow_asset(
    project_path: String,
    source_path: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = PathBuf::from(&project_path);
    if !unibrowser_project::is_unibrow_path(&project) {
        return Err("not a .unibrow project".into());
    }
    let source = PathBuf::from(&source_path);
    if !source.is_file() {
        return Err("source is not a file".into());
    }
    let mut package = unibrowser_project::open_package(&project).map_err(|e| e.to_string())?;
    let node_type = unibrowser_project::guess_asset_node_type(&source);
    let label = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "asset".into());
    unibrowser_project::add_asset_copy(&mut package, &source, node_type, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(&state, &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

fn reload_unibrow_project_tab(
    state: &AppState,
    project: &Path,
    project_path: String,
) -> Result<(), String> {
    let package = unibrowser_project::open_package(project).map_err(|e| e.to_string())?;
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let project_index = tabs.tabs().iter().position(|t| {
        matches!(&t.location, BrowserLocation::File { path } if path == project)
    });
    if let Some(index) = project_index {
        if let Some(tab) = tabs.tab_mut(index) {
            use crate::project_backend::UnibrowBackend;
            tab.document = Some(unibrowser_engine::DocumentSession::new(
                project_path,
                Box::new(UnibrowBackend::open(package)),
            ));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn remove_unibrow_node(
    project_path: String,
    node_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = PathBuf::from(&project_path);
    let mut package = unibrowser_project::open_package(&project).map_err(|e| e.to_string())?;
    unibrowser_project::remove_node(&mut package, &node_id).map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(&state, &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

#[tauri::command]
pub fn rename_unibrow_node(
    project_path: String,
    node_id: String,
    label: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = PathBuf::from(&project_path);
    let mut package = unibrowser_project::open_package(&project).map_err(|e| e.to_string())?;
    unibrowser_project::rename_node_label(&mut package, &node_id, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(&state, &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

#[tauri::command]
pub fn get_ffmpeg_probe() -> unibrowser_media::FfmpegProbe {
    unibrowser_media::probe_ffmpeg_install()
}

#[tauri::command]
pub fn get_home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())
}

#[tauri::command]
pub fn get_preferences() -> PreferencesDto {
    let p = load_preferences();
    PreferencesDto {
        onboarding_dismissed: p.onboarding_dismissed,
    }
}

#[tauri::command]
pub fn dismiss_onboarding() -> Result<(), String> {
    let mut p = load_preferences();
    p.onboarding_dismissed = true;
    save_preferences(&p)
}

#[tauri::command]
pub fn save_window_geometry(
    width: f64,
    height: f64,
    x: Option<f64>,
    y: Option<f64>,
) -> Result<(), String> {
    let state = WindowState {
        width,
        height,
        x,
        y,
    };
    crate::preferences::save_window_state(&state)
}

#[tauri::command]
pub fn create_unibrow_project(
    parent_dir: String,
    name: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let parent = PathBuf::from(parent_dir);
    let root = parent.join(format!("{}.unibrow", name));
    let _package = unibrowser_project::create_package(&root, &name).map_err(|e| e.to_string())?;
    let loc = BrowserLocation::File { path: root.clone() };
    let doc = crate::state::open_file_document(&root);
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = false;
        tab.document = doc;
    }
    update_content_webview(&app, &loc)?;
    snapshot(&state)
}

#[tauri::command]
pub fn search_open_vsx_catalog(
    query: String,
    network_opt_in: bool,
) -> Result<Vec<crate::open_vsx::OpenVsxHitDto>, String> {
    crate::open_vsx::search_open_vsx(&query, network_opt_in)
}

#[tauri::command]
pub fn open_mod_portal(
    ext: Option<String>,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let page = match ext {
        Some(e) if !e.is_empty() => format!("mods?ext={}", e),
        _ => "mods".into(),
    };
    let loc = BrowserLocation::Internal { page };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = is_dir;
        tab.document = doc;
    }
    update_content_webview(&app, &loc)?;
    snapshot(&state)
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
