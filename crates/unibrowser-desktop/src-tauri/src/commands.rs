use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Serialize)]
pub struct PathPropertiesDto {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_unix: i64,
    pub extension: Option<String>,
    pub sniffed_type: Option<String>,
}
use tauri::{AppHandle, Manager, State};
use unibrowser_engine::{
    keybinding::COMMAND_PALETTE_TOGGLE,
    palette::{fuzzy_score, search_commands},
    KeyChord,
};
use unibrowser_mod_api::{BrowserLocation, DownloadDisposition};

use base_text::{TextBackend, try_text_backend};

use crate::config::toolbars_file_path;
use crate::downloads::{
    apply_download_disposition, copy_file_to_path, download_http_to_file,
    suggested_download_filename,
};
use crate::state::{AppState, list_dir};
use crate::mod_portal::{ModSuggestionDto, suggest_mods_for_unknown_path};
use crate::preferences::{
    default_pinned_locations, load_preferences, push_recent_file, save_preferences,
    DownloadPreference, PinnedLocation, UiTheme, WindowState,
};
use crate::state::FileCompareState;
use unibrowser_mod_api::types::commands::TEXT_SAVE;
use crate::epub_preview::{EpubChapterDto, EpubSearchHitDto};
use crate::path_security::{resolve_open_path, resolve_unibrow_asset_file};
use crate::webview::{adjust_page_font_for_url, sync_content_bounds, sync_content_webview_for_window};
use unibrowser_mod_api::handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID};
use unibrowser_engine::{
    host_context_for_browse, navigation::parse_user_input, save_command_manager_file,
    CommandManagerFile, ToolbarPhase,
};
use tauri::Emitter;

#[derive(Serialize)]
pub struct TabDto {
    pub id: u64,
    pub label: String,
    pub location_kind: String,
    pub is_directory: bool,
    pub is_http: bool,
    pub dirty: bool,
    /// Folder path when this tab can receive file drops (browse / directory tabs).
    pub browse_path: Option<String>,
}

#[derive(Serialize)]
pub struct TabSearchHitDto {
    pub tab_index: usize,
    pub label: String,
    pub omnibox: String,
    pub score: i32,
}

const ERR_UNSAVED_CLOSE: &str = "unsaved: confirm close";

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
    pub coord_space: String,
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
    pub chord: Option<String>,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectNodeDto {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub asset: Option<String>,
    pub x: f32,
    pub y: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectClipDto {
    pub id: String,
    pub node_id: String,
    pub node_type: String,
    pub start_sec: f32,
    pub duration_sec: f32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectMarkerDto {
    pub id: String,
    pub time_sec: f32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectTransitionDto {
    pub id: String,
    pub after_clip_id: String,
    pub kind: String,
    pub duration_sec: f32,
}

#[derive(Serialize)]
pub struct TimelineScrubPreviewDto {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_notes: Option<Vec<MidiNoteOnDto>>,
}

#[derive(Serialize)]
pub struct MidiSummaryDto {
    pub format: u16,
    pub track_count: u16,
    pub ticks_per_quarter: u16,
}

#[derive(Serialize)]
pub struct MidiNoteOnDto {
    pub track_index: u16,
    pub tick: u32,
    pub note: u8,
    pub velocity: u8,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_clips: Option<Vec<ProjectClipDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_transitions: Option<Vec<ProjectTransitionDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_markers: Option<Vec<ProjectMarkerDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_summary: Option<MidiSummaryDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_snap_to_grid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_scrub_audio_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_note_preview: Option<Vec<MidiNoteOnDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_engines: Option<Vec<crate::pdf_proving::PdfEngineLineDto>>,
}

#[derive(Serialize)]
pub struct TextDocumentDto {
    pub path: String,
    pub content: String,
    pub font_size_pt: f32,
    pub language: String,
    pub dirty: bool,
    pub encoding: String,
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
    pub show_onboarding_tour: bool,
    pub recent_files: Vec<RecentFileDto>,
    pub split_view_enabled: bool,
    pub split_index: Option<usize>,
    pub split_text_document: Option<TextDocumentDto>,
    pub split_document_viewer: Option<DocumentViewerDto>,
    pub split_status: Option<String>,
    pub show_new_tab_page: bool,
    pub browse_folder_path: String,
    pub diff_view: Option<DiffViewDto>,
    pub pinned_locations: Vec<PinnedLocationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome_notice: Option<String>,
    pub ntp_sections: NtpSectionsDto,
    pub files_view: FilesViewDto,
    pub folder_favorites: Vec<String>,
}

#[derive(Serialize)]
pub struct FilesViewDto {
    pub sort: String,
    pub sort_ascending: bool,
    pub show_hidden: bool,
}

fn files_view_dto(prefs: &crate::preferences::Preferences) -> FilesViewDto {
    let v = &prefs.files_view;
    FilesViewDto {
        sort: match v.sort {
            crate::preferences::FilesSortMode::Name => "name",
            crate::preferences::FilesSortMode::Date => "date",
            crate::preferences::FilesSortMode::Size => "size",
        }
        .into(),
        sort_ascending: v.sort_ascending,
        show_hidden: v.show_hidden,
    }
}

#[derive(Serialize)]
pub struct NtpSectionsDto {
    pub show_hero: bool,
    pub show_actions: bool,
    pub show_browse: bool,
    pub show_recent: bool,
}

fn ntp_sections_dto(prefs: &crate::preferences::Preferences) -> NtpSectionsDto {
    let s = &prefs.ntp_sections;
    NtpSectionsDto {
        show_hero: s.show_hero,
        show_actions: s.show_actions,
        show_browse: s.show_browse,
        show_recent: s.show_recent,
    }
}

#[derive(Serialize)]
pub struct DiffViewDto {
    pub left_path: String,
    pub right_path: String,
    pub left_content: String,
    pub right_content: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct PinnedLocationDto {
    pub id: String,
    pub label: String,
    pub location: String,
}

#[derive(Serialize)]
pub struct OmniboxSuggestionDto {
    pub label: String,
    pub value: String,
    pub kind: String,
    pub score: i32,
}

#[derive(Serialize)]
pub struct RecentFileDto {
    pub path: String,
    pub label: String,
    pub pinned: bool,
}

#[derive(Serialize)]
pub struct ModPortalDto {
    pub title: String,
    pub message: String,
    pub suggestions: Vec<ModSuggestionDto>,
    pub sideload_hint: String,
}


fn ws_state(state: &AppState, window: &tauri::Window) -> Result<crate::window_session::SharedWindowSession, String> {
    state.session(window.label())
}

fn ws(state: &State<AppState>, window: &tauri::Window) -> Result<crate::window_session::SharedWindowSession, String> {
    ws_state(state.inner(), window)
}

fn ws_label(state: &AppState, label: &str) -> Result<crate::window_session::SharedWindowSession, String> {
    state.session(label)
}

fn location_to_omnibox(loc: &BrowserLocation) -> String {
    match loc {
        BrowserLocation::File { path } => format!("file://{}", path.display()),
        BrowserLocation::Http { url } => url.clone(),
        BrowserLocation::Internal { page } => format!("unibrowser://{}", page),
    }
}

fn tab_is_dirty(tab: &unibrowser_engine::tabs::Tab) -> bool {
    tab.document
        .as_ref()
        .map(|d| d.backend.is_dirty())
        .unwrap_or(false)
}

fn tab_browse_path(
    state: &AppState,
    tabs: &unibrowser_engine::TabManager,
    tab: &unibrowser_engine::tabs::Tab,
) -> Option<String> {
    if tab_is_new_tab_home(tab) {
        return Some(listing_cwd(state, tabs).to_string_lossy().into_owned());
    }
    if tab.is_directory {
        if let BrowserLocation::File { path } = &tab.location {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

fn build_tab_dto(
    state: &AppState,
    tabs: &unibrowser_engine::TabManager,
    tab: &unibrowser_engine::tabs::Tab,
) -> TabDto {
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
        dirty: tab_is_dirty(tab),
        browse_path: tab_browse_path(state, tabs, tab),
    }
}

fn open_file_tabs_dto(state: &AppState, window_label: &str) -> Vec<OpenFileTabDto> {
    let session = ws_label(state, window_label).ok();
    let Some(sess) = session else {
        return Vec::new();
    };
    let tabs = match sess.tabs.lock() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
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

fn open_unibrow_package(path: &Path) -> Result<unibrowser_project::ProjectPackage, String> {
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(path, &cache).map_err(|e| e.to_string())?;
    unibrowser_project::open_package(&root).map_err(|e| e.to_string())
}

fn document_viewer_dto(
    doc: &mut unibrowser_engine::DocumentSession,
    state: &AppState,
    window_label: &str,
) -> Option<DocumentViewerDto> {
    if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
        let root = Path::new(&doc.path);
        let opened = open_unibrow_package(root).ok();
        let nodes = opened.as_ref().map(|pkg| {
            pkg.manifest
                .nodes
                .iter()
                .map(|n| ProjectNodeDto {
                    id: n.id.clone(),
                    node_type: n.node_type.clone(),
                    label: n.label.clone(),
                    asset: n.asset.clone(),
                    x: n.x,
                    y: n.y,
                    color: n.color.clone(),
                })
                .collect()
        });
        let clips = opened.as_ref().map(|pkg| {
            pkg.manifest
                .timeline
                .clips
                .iter()
                .map(|c| {
                    let node_type = pkg
                        .manifest
                        .nodes
                        .iter()
                        .find(|n| n.id == c.node_id)
                        .map(|n| n.node_type.clone())
                        .unwrap_or_else(|| "asset".into());
                    ProjectClipDto {
                        id: c.id.clone(),
                        node_id: c.node_id.clone(),
                        node_type,
                        start_sec: c.start_sec,
                        duration_sec: c.duration_sec,
                        label: c.label.clone(),
                        color: c.color.clone(),
                    }
                })
                .collect()
        });
        let transitions = opened.as_ref().map(|pkg| {
            pkg.manifest
                .timeline
                .transitions
                .iter()
                .map(|t| ProjectTransitionDto {
                    id: t.id.clone(),
                    after_clip_id: t.after_clip_id.clone(),
                    kind: t.kind.clone(),
                    duration_sec: t.duration_sec,
                })
                .collect()
        });
        let markers = opened.as_ref().map(|pkg| {
            pkg.manifest
                .timeline
                .markers
                .iter()
                .map(|m| ProjectMarkerDto {
                    id: m.id.clone(),
                    time_sec: m.time_sec,
                    title: m.title.clone(),
                    color: m.color.clone(),
                })
                .collect()
        });
        let prefs = load_preferences();
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
            open_file_tabs: Some(open_file_tabs_dto(state, window_label)),
            project_clips: clips,
            project_transitions: transitions,
            project_markers: markers,
            midi_summary: None,
            board_snap_to_grid: Some(prefs.board_snap_to_grid),
            midi_scrub_audio_enabled: Some(prefs.midi_scrub_audio_enabled),
            midi_note_preview: None,
            pdf_engines: None,
        });
    }
    if doc.backend.backend_id() == crate::settings_stub::BACKEND_ID {
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::settings_stub::VIEWER_KIND.into(),
            title: "Settings".into(),
            pdf_scale: None,
            pdf_page: None,
            media_note: None,
            project_nodes: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
        });
    }
    if doc.backend.backend_id() == crate::about_stub::BACKEND_ID {
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::about_stub::VIEWER_KIND.into(),
            title: "About Unibrowser".into(),
            pdf_scale: None,
            pdf_page: None,
            media_note: None,
            project_nodes: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
        });
    }
    if doc.backend.backend_id() == crate::keys_stub::BACKEND_ID {
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::keys_stub::VIEWER_KIND.into(),
            title: "Keyboard shortcuts".into(),
            pdf_scale: None,
            pdf_page: None,
            media_note: None,
            project_nodes: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
        });
    }
    if doc.backend.backend_id() == crate::midi_stub::BACKEND_ID {
        let midi = crate::midi_stub::try_midi_backend(&mut *doc.backend).expect("midi backend");
        let summary = midi.summary.clone();
        let notes = midi
            .note_preview
            .iter()
            .map(|n| MidiNoteOnDto {
                track_index: n.track_index,
                tick: n.tick,
                note: n.note,
                velocity: n.velocity,
            })
            .collect();
        let status = doc.backend.status_line();
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::midi_stub::VIEWER_KIND.into(),
            title: doc.backend.display_name().into(),
            pdf_scale: None,
            pdf_page: None,
            media_note: Some(status),
            project_nodes: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: Some(MidiSummaryDto {
                format: summary.format,
                track_count: summary.track_count,
                ticks_per_quarter: summary.ticks_per_quarter,
            }),
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: Some(notes),
            pdf_engines: None,
        });
    }
    let viewer = match doc.backend.backend_id() {
        base_docx::BACKEND_ID => base_docx::VIEWER_KIND,
        base_pdf::BACKEND_ID => base_pdf::VIEWER_KIND,
        base_dxf::BACKEND_ID => base_dxf::VIEWER_KIND,
        base_js::BACKEND_ID => base_js::VIEWER_KIND,
        base_video::BACKEND_ID => base_video::VIEWER_KIND,
        base_audio::BACKEND_ID => base_audio::VIEWER_KIND,
        base_svg::BACKEND_ID => base_svg::VIEWER_KIND,
        base_png::BACKEND_ID => base_png::VIEWER_KIND,
        crate::epub_stub::BACKEND_ID => crate::epub_stub::VIEWER_KIND,
        crate::csv_stub::BACKEND_ID => crate::csv_stub::VIEWER_KIND,
        _ => return None,
    };
    let (pdf_scale, pdf_page, pdf_engines) = if doc.backend.backend_id() == base_pdf::BACKEND_ID {
        let engines = crate::pdf_proving::pdf_engine_lines(Path::new(&doc.path));
        let eng = Some(engines);
        base_pdf::try_pdf_backend(&mut *doc.backend)
            .map(|b| (Some(b.render_scale), Some(b.page_index), eng.clone()))
            .unwrap_or((None, None, eng))
    } else {
        (None, None, None)
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
        project_clips: None,
        project_transitions: None,
        project_markers: None,
        midi_summary: None,
        board_snap_to_grid: None,
        midi_scrub_audio_enabled: None,
        midi_note_preview: None,
        pdf_engines,
    })
}

fn tab_pane_dtos(
    tabs: &mut unibrowser_engine::TabManager,
    index: usize,
    state: &AppState,
    window_label: &str,
) -> (
    Option<TextDocumentDto>,
    Option<DocumentViewerDto>,
    Option<String>,
) {
    let tab = match tabs.tab_mut(index) {
        Some(t) => t,
        None => return (None, None, None),
    };
    let status = tab
        .document
        .as_ref()
        .map(|d| d.backend.status_line());
    let viewer = tab
        .document
        .as_mut()
        .and_then(|d| document_viewer_dto(d, state, window_label));
    let text = if viewer.is_some() {
        None
    } else {
        tab.document.as_mut().and_then(text_document_dto)
    };
    (text, viewer, status)
}

fn recent_files_dto(prefs: &crate::preferences::Preferences) -> Vec<RecentFileDto> {
    let mut entries: Vec<_> = prefs
        .recent_files
        .iter()
        .map(|e| RecentFileDto {
            path: e.path.clone(),
            label: e.label.clone(),
            pinned: e.pinned,
        })
        .collect();
    entries.sort_by(|a, b| b.pinned.cmp(&a.pinned));
    entries
}

fn list_dir_timed(path: &Path) -> Result<Vec<base_files::DirEntry>, String> {
    use std::sync::Arc;
    use std::time::Duration;
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let timeout = Duration::from_millis(12_000);
    let prefs = load_preferences();
    let raw = crate::listing_control::list_dir_with_timeout(path, cancel, timeout)?;
    Ok(crate::files_listing::apply_files_view(raw, &prefs.files_view))
}

fn list_dir_snapshot(
    sess: &crate::window_session::SharedWindowSession,
    path: &Path,
) -> Result<Vec<base_files::DirEntry>, String> {
    match list_dir_timed(path) {
        Ok(entries) => Ok(entries),
        Err(e) => {
            if let Ok(mut notice) = sess.chrome_notice.lock() {
                *notice = Some(e.clone());
            }
            Ok(Vec::new())
        }
    }
}

fn listing_cwd(state: &AppState, tabs: &unibrowser_engine::TabManager) -> PathBuf {
    if let Some(tab) = tabs.active_tab() {
        if let BrowserLocation::File { path } = &tab.location {
            if path.is_dir() {
                return path.clone();
            }
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
    }
    state.cwd.clone()
}

fn detect_text_encoding(path: &str) -> String {
    use std::io::Read;
    let path = Path::new(path);
    let mut head = [0u8; 4];
    let n = std::fs::File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .unwrap_or(0);
    let b = &head[..n];
    if b.starts_with(&[0xEF, 0xBB, 0xBF]) {
        "UTF-8 with BOM".into()
    } else if b.len() >= 2 && b[0] == 0xFF && b[1] == 0xFE {
        "UTF-16 LE".into()
    } else if b.len() >= 2 && b[0] == 0xFE && b[1] == 0xFF {
        "UTF-16 BE".into()
    } else {
        "UTF-8".into()
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
        encoding: detect_text_encoding(&backend.path),
    })
}

fn tab_is_new_tab_home(tab: &unibrowser_engine::tabs::Tab) -> bool {
    matches!(
        &tab.location,
        BrowserLocation::Internal { page } if page == "home"
    )
}

fn snapshot(state: &AppState, window_label: &str) -> Result<TabContentDto, String> {
    state.refresh_command_manager_context(window_label);
    let sess = ws_label(state, window_label)?;
    let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    let omnibox = tabs
        .active_tab()
        .map(|t| location_to_omnibox(&t.location))
        .ok_or("no active tab")?;
    let browse_folder_path = listing_cwd(state, &tabs).to_string_lossy().into_owned();
    let directory_entries = if let Some(active) = tabs.active_tab() {
        if tab_is_new_tab_home(active) {
            Some(list_dir_snapshot(&sess, Path::new(&browse_folder_path))?)
        } else if active.is_directory {
            if let BrowserLocation::File { path } = &active.location {
                Some(list_dir_snapshot(&sess, path)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let show_new_tab_page = tabs
        .active_tab()
        .map(tab_is_new_tab_home)
        .unwrap_or(false);
    let document_status = tabs
        .active_tab()
        .and_then(|t| t.document.as_ref())
        .map(|d| d.backend.status_line());
    let (document_viewer, text_document) = match tabs.active_tab_mut() {
        Some(tab) => {
            let viewer = tab
                .document
                .as_mut()
                .and_then(|d| document_viewer_dto(d, state, window_label));
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
    let split_cfg = sess.split_view.lock().map_err(|e| e.to_string())?;
    let split_enabled = split_cfg.enabled;
    let split_index = split_cfg.secondary_index;
    let (split_text_document, split_document_viewer, split_status) =
        if split_enabled {
            let idx = split_index.unwrap_or_else(|| {
                let active = tabs.active_index();
                if active > 0 {
                    active - 1
                } else {
                    active
                }
            });
            tab_pane_dtos(&mut tabs, idx, state, window_label)
        } else {
            (None, None, None)
        };
    let prefs = load_preferences();
    let diff_view = build_diff_view(state, window_label)?;
    let (text_document, document_viewer) = if diff_view.is_some() {
        (None, None)
    } else {
        (text_document, document_viewer)
    };
    let pinned_locations = pinned_locations_dto(&prefs);
    let chrome_notice = sess
        .chrome_notice
        .lock()
        .map_err(|e| e.to_string())?
        .take();
    Ok(TabContentDto {
        tabs: tabs
            .tabs()
            .iter()
            .map(|t| build_tab_dto(state, &tabs, t))
            .collect(),
        active_index: tabs.active_index(),
        omnibox,
        mods_loaded: state.mods_loaded_line(),
        status: document_status
            .clone()
            .unwrap_or_else(|| {
                format!(
                    "{} for command palette",
                    state
                        .keybindings
                        .lock()
                        .map(|k| k.palette_chord_display())
                        .unwrap_or_else(|_| "Ctrl+Shift+P".into())
                )
            }),
        directory_entries,
        document_status,
        text_document,
        mod_portal,
        document_viewer,
        show_onboarding: !prefs.onboarding_dismissed,
        show_onboarding_tour: !prefs.onboarding_tour_skipped_forever,
        recent_files: recent_files_dto(&prefs),
        split_view_enabled: split_enabled,
        split_index: if split_enabled { split_index } else { None },
        split_text_document,
        split_document_viewer,
        split_status,
        show_new_tab_page,
        browse_folder_path,
        diff_view,
        pinned_locations,
        chrome_notice,
        ntp_sections: ntp_sections_dto(&prefs),
        files_view: files_view_dto(&prefs),
        folder_favorites: prefs.folder_favorites.clone(),
    })
}

fn build_diff_view(state: &AppState, window_label: &str) -> Result<Option<DiffViewDto>, String> {
    let sess = ws_label(state, window_label)?;
    let guard = sess
        .file_compare
        .lock()
        .map_err(|e| e.to_string())?;
    let (left_path, right_path) = match guard.as_ref() {
        Some(c) => (c.left_path.clone(), c.right_path.clone()),
        None => return Ok(None),
    };
    let left = resolve_open_path(&left_path, &state.cwd)?;
    let right = resolve_open_path(&right_path, &state.cwd)?;
    let left_content = std::fs::read_to_string(&left).map_err(|e| e.to_string())?;
    let right_content = std::fs::read_to_string(&right).map_err(|e| e.to_string())?;
    let language = TextBackend::language_id(&left_path).into();
    Ok(Some(DiffViewDto {
        left_path,
        right_path,
        left_content,
        right_content,
        language,
    }))
}

fn pinned_locations_dto(prefs: &crate::preferences::Preferences) -> Vec<PinnedLocationDto> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let pins = if prefs.pinned_locations.is_empty() {
        default_pinned_locations(&home)
    } else {
        prefs.pinned_locations.clone()
    };
    pins.into_iter()
        .map(|p| PinnedLocationDto {
            id: p.id,
            label: p.label,
            location: p.location,
        })
        .collect()
}

fn active_document_path(state: &AppState, window: &tauri::Window) -> Result<String, String> {
    let _sess_tabs = ws_state(state, window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    let doc = tab.document.as_ref().ok_or("no document open")?;
    Ok(doc.path.clone())
}

fn touch_recent_file(path: &Path) {
    if !path.is_file() {
        return;
    }
    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    let mut prefs = load_preferences();
    push_recent_file(&mut prefs, path.to_string_lossy().into_owned(), label);
    let _ = save_preferences(&prefs);
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
                message: "Install support for formats you need. Open VSX search + VSIX install (V1 command metadata; no extension host).".into(),
                suggestions,
                sideload_hint: "Install from file… (.vsix, .ubmod) — always allowed.".into(),
            });
        }
    }
    if let BrowserLocation::File { path } = &tab.location {
        if tab.document.is_none() && path.is_file() {
            let suggestions = suggest_mods_for_unknown_path(&path.to_string_lossy());
            return Some(ModPortalDto {
                title: "Install support for this file".into(),
                message: format!(
                    "Unibrowser has no handler for {} yet. Install a mod below (Open VSX or .ubmod); the panel stays beside your tab and does not cover file content.",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                suggestions,
                sideload_hint: "Install from file… (.vsix, .ubmod) or search Open VSX when online.".into(),
            });
        }
    }
    None
}

#[tauri::command]
pub fn get_browser_state(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn new_tab(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    let loc = BrowserLocation::Internal {
        page: "home".into(),
    };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = is_dir;
        tab.document = doc;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn select_tab(window: tauri::Window, 
    index: usize,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.set_active(index);
    let loc = tabs
        .active_tab()
        .map(|t| t.location.clone())
        .ok_or("no tab")?;
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn close_tab(window: tauri::Window, 
    index: usize,
    force: bool,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    {
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        if index >= tabs.tab_count() {
            return Err("invalid tab index".into());
        }
        if !force {
            let tab = tabs.tabs().get(index).ok_or("invalid tab")?;
            if tab_is_dirty(tab) {
                return Err(ERR_UNSAVED_CLOSE.into());
            }
        }
        if let Some(tab) = tabs.tabs().get(index) {
            _sess_tabs.purge_tab_resources(tab);
        }
        if !tabs.close_tab(index) {
            return Err("cannot close last tab".into());
        }
    }
    let loc = {
        let _sess_tabs = ws(&state, &window)?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs
            .active_tab()
            .map(|t| t.location.clone())
            .ok_or("no tab")?
    };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn search_open_tabs(window: tauri::Window, query: String, state: State<AppState>) -> Result<Vec<TabSearchHitDto>, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let mut hits: Vec<TabSearchHitDto> = tabs
        .tabs()
        .iter()
        .enumerate()
        .filter_map(|(index, tab)| {
            let label = tab.location.display_label();
            let omnibox = location_to_omnibox(&tab.location);
            let score = fuzzy_score(&query, &label).max(fuzzy_score(&query, &omnibox));
            if score < 0 && !query.is_empty() {
                return None;
            }
            Some(TabSearchHitDto {
                tab_index: index,
                label,
                omnibox,
                score: score.max(0),
            })
        })
        .collect();
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.label.cmp(&b.label)));
    Ok(hits)
}

pub fn restore_session_on_launch(state: &AppState, app: &AppHandle) -> Result<(), String> {
    let prefs = load_preferences();
    if !prefs.restore_session_on_launch {
        return Ok(());
    }
    let snap = crate::session::load_session_snapshot();
    let snap = match snap {
        Some(s) if !s.tab_inputs.is_empty() => s,
        _ => return Ok(()),
    };
    let mut locations = Vec::new();
    for input in &snap.tab_inputs {
        let loc = parse_user_input(input, &state.cwd);
        let (loc, _, _) = state.resolve_location(loc);
        locations.push(loc);
    }
    {
        let _sess_tabs = ws_label(state, "main")?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.load_locations(locations, snap.active_index);
        let count = tabs.tab_count();
        for i in 0..count {
            let loc = tabs
                .tab_mut(i)
                .map(|t| t.location.clone())
                .ok_or("tab missing")?;
            let (loc, is_dir, doc) = state.resolve_location(loc);
            if let Some(tab) = tabs.tab_mut(i) {
                tab.location = loc;
                tab.is_directory = is_dir;
                tab.document = doc;
            }
        }
    }
    let loc = {
        let _sess_tabs = ws_label(state, "main")?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs
            .active_tab()
            .map(|t| t.location.clone())
            .ok_or("no tab")?
    };
    update_content_webview(app, "main", &loc)?;
    Ok(())
}

pub fn open_dropped_paths(
    app: &AppHandle,
    state: &AppState,
    paths: Vec<PathBuf>,
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    for (i, path) in paths.iter().enumerate() {
        let safe = resolve_open_path(&path.to_string_lossy(), &state.cwd)?;
        allow_asset_protocol_file(app, &safe);
        if safe.is_file() {
            touch_recent_file(&safe);
        }
        let loc = BrowserLocation::File { path: safe };
        let (loc, is_dir, doc) = state.resolve_location(loc);
        {
            let _sess_tabs = ws_label(state, "main")?;
            let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
            if i == 0 {
                tabs.navigate_active(loc.clone(), is_dir, doc);
            } else {
                tabs.open_tab(loc.clone());
                if let Some(tab) = tabs.active_tab_mut() {
                    tab.is_directory = is_dir;
                    tab.document = doc;
                }
            }
        }
        update_content_webview(app, "main", &loc)?;
    }
    app.emit("browser-state-updated", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn navigate_omnibox(window: tauri::Window, input: String, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let loc = state.navigate_input(&input, window.label())?;
    if let BrowserLocation::File { path } = &loc {
        allow_asset_protocol_file(&app, path);
        if path.is_file() {
            touch_recent_file(path);
        }
    }
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

fn allow_asset_protocol_file(app: &AppHandle, path: &Path) {
    if !path.is_file() {
        return;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let needs_asset = matches!(
        ext.as_str(),
        "mp4"
            | "webm"
            | "wav"
            | "mid"
            | "midi"
            | "mp3"
            | "ogg"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "svg"
    );
    if !needs_asset {
        return;
    }
    let _ = app.asset_protocol_scope().allow_file(path);
}

#[tauri::command]
pub fn allow_asset_path(window: tauri::Window, 
    path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    let p = resolve_open_path(&path, &state.cwd)?;
    if !p.is_file() {
        return Err("not a file".into());
    }
    app.asset_protocol_scope()
        .allow_file(&p)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn allow_unibrow_asset(window: tauri::Window, 
    project_path: String,
    asset_name: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let file = resolve_unibrow_asset_file(&project, &asset_name)?;
    if !file.is_file() {
        return Err("asset file missing".into());
    }
    allow_asset_protocol_file(&app, &file);
    app.asset_protocol_scope()
        .allow_file(&file)
        .map_err(|e| e.to_string())?;
    Ok(file.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn open_path(window: tauri::Window, path: String, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let file_path = resolve_open_path(&path, &state.cwd)?;
    let profile_open = std::env::var("UNIBROWSER_PROFILE_OPEN")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let profile_pdf = profile_open
        && file_path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    let open_started = if profile_pdf {
        Some(std::time::Instant::now())
    } else {
        None
    };
    allow_asset_protocol_file(&app, &file_path);
    let loc = BrowserLocation::File { path: file_path.clone() };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    if let Some(started) = open_started {
        eprintln!(
            "[unibrowser] open_path pdf {} resolve+open took {:?}",
            file_path.display(),
            started.elapsed()
        );
    }
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.navigate_active(loc.clone(), is_dir, doc);
    if let BrowserLocation::File { path } = &loc {
        if path.is_file() {
            touch_recent_file(path);
        }
    }
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn open_path_in_new_tab(
    window: tauri::Window,
    path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let file_path = resolve_open_path(&path, &state.cwd)?;
    allow_asset_protocol_file(&app, &file_path);
    let loc = BrowserLocation::File { path: file_path.clone() };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = is_dir;
        tab.document = doc;
    }
    if let BrowserLocation::File { path } = &loc {
        if path.is_file() {
            touch_recent_file(path);
        }
    }
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn reveal_path_in_system(
    path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let p = resolve_open_path(&path, &state.cwd)?;
    app.opener()
        .reveal_item_in_dir(p)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_folder_favorite(path: String, state: State<AppState>) -> Result<Vec<String>, String> {
    let resolved = resolve_open_path(&path, &state.cwd)?;
    if !resolved.is_dir() {
        return Err("only folders can be favourites".into());
    }
    let path_str = resolved.to_string_lossy().into_owned();
    let mut prefs = load_preferences();
    crate::preferences::toggle_folder_favorite(&mut prefs, &path_str);
    save_preferences(&prefs)?;
    Ok(prefs.folder_favorites.clone())
}

#[derive(Serialize)]
pub struct ArchiveEntryDto {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[tauri::command]
pub fn peek_archive_entries(
    path: String,
    max_entries: Option<usize>,
    state: State<AppState>,
) -> Result<Vec<ArchiveEntryDto>, String> {
    let p = resolve_open_path(&path, &state.cwd)?;
    let cap = max_entries.unwrap_or(200).clamp(1, 500);
    let entries = unibrowser_project::list_zip_archive_entries(&p, cap)
        .map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| ArchiveEntryDto {
            path: e.path,
            is_dir: e.is_dir,
            size: e.size,
        })
        .collect())
}

#[tauri::command]
pub fn open_archive_entry(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
    archive_path: String,
    entry_path: String,
) -> Result<TabContentDto, String> {
    let archive = resolve_open_path(&archive_path, &state.cwd)?;
    let dest_root = crate::config::archive_peek_extract_dir()
        .ok_or("could not resolve archive extract directory")?;
    std::fs::create_dir_all(&dest_root).map_err(|e| e.to_string())?;
    let extracted = unibrowser_project::extract_zip_archive_entry(&archive, &entry_path, &dest_root)
        .map_err(|e| e.to_string())?;
    allow_asset_protocol_file(&app, &extracted);
    let loc = BrowserLocation::File {
        path: extracted.clone(),
    };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    if extracted.is_file() {
        touch_recent_file(&extracted);
    }
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.navigate_active(loc.clone(), is_dir, doc);
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

fn guard_deletable(path: &Path) -> Result<(), String> {
    crate::file_ops::guard_mutating(path)
}

#[tauri::command]
pub fn delete_paths(
    window: tauri::Window,
    state: State<AppState>,
    paths: Vec<String>,
) -> Result<TabContentDto, String> {
    if paths.is_empty() {
        return Err("no paths to delete".into());
    }
    for raw in paths {
        let p = resolve_open_path(&raw, &state.cwd)?;
        guard_deletable(&p)?;
        crate::trash::trash_or_remove(&p)?;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn transfer_paths(
    window: tauri::Window,
    state: State<AppState>,
    paths: Vec<String>,
    destination_dir: String,
    mode: String,
) -> Result<TabContentDto, String> {
    if paths.is_empty() {
        return Err("no paths selected".into());
    }
    let copy = match mode.as_str() {
        "copy" => true,
        "move" => false,
        _ => return Err("mode must be copy or move".into()),
    };
    let dest = resolve_open_path(&destination_dir, &state.cwd)?;
    if !dest.is_dir() {
        return Err("destination is not a directory".into());
    }
    let sources: Vec<PathBuf> = paths
        .iter()
        .map(|p| resolve_open_path(p, &state.cwd))
        .collect::<Result<Vec<_>, _>>()?;
    crate::file_ops::transfer_paths(&sources, &dest, copy)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn create_folder(
    window: tauri::Window,
    state: State<AppState>,
    parent_dir: String,
    name: String,
) -> Result<TabContentDto, String> {
    let parent = resolve_open_path(&parent_dir, &state.cwd)?;
    if !parent.is_dir() {
        return Err("parent is not a directory".into());
    }
    let name = crate::file_ops::validate_entry_name(&name)?;
    let target = parent.join(&name);
    if target.exists() {
        return Err("an item with that name already exists".into());
    }
    fs::create_dir(&target).map_err(|e| e.to_string())?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn create_text_file(
    window: tauri::Window,
    state: State<AppState>,
    parent_dir: String,
    name: String,
) -> Result<TabContentDto, String> {
    let parent = resolve_open_path(&parent_dir, &state.cwd)?;
    if !parent.is_dir() {
        return Err("parent is not a directory".into());
    }
    let name = crate::file_ops::validate_entry_name(&name)?;
    let target = parent.join(&name);
    if target.exists() {
        return Err("an item with that name already exists".into());
    }
    fs::write(&target, b"").map_err(|e| e.to_string())?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn rename_path(
    window: tauri::Window,
    state: State<AppState>,
    path: String,
    new_name: String,
) -> Result<TabContentDto, String> {
    let src = resolve_open_path(&path, &state.cwd)?;
    crate::file_ops::guard_mutating(&src)?;
    let new_name = crate::file_ops::validate_entry_name(&new_name)?;
    let parent = src
        .parent()
        .ok_or("path has no parent directory")?;
    let dest = parent.join(&new_name);
    if dest == src {
        return snapshot(&state, window.label());
    }
    if dest.exists() {
        return Err("an item with that name already exists".into());
    }
    fs::rename(&src, &dest).map_err(|e| e.to_string())?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn duplicate_path(
    window: tauri::Window,
    state: State<AppState>,
    path: String,
) -> Result<TabContentDto, String> {
    let src = resolve_open_path(&path, &state.cwd)?;
    crate::file_ops::guard_mutating(&src)?;
    let parent = src
        .parent()
        .ok_or("path has no parent directory")?;
    let dest = crate::file_ops::duplicate_destination(parent, &src)?;
    crate::file_ops::copy_path_recursive(&src, &dest)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn path_properties(path: String, state: State<AppState>) -> Result<PathPropertiesDto, String> {
    let p = resolve_open_path(&path, &state.cwd)?;
    let meta = fs::metadata(&p).map_err(|e| e.to_string())?;
    let modified_unix = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = p
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .filter(|e| !e.is_empty());
    let sniffed_type = if meta.is_dir() {
        None
    } else {
        crate::content_sniff::sniff_format_label(&p).map(|s| s.to_string())
    };
    Ok(PathPropertiesDto {
        path: p.to_string_lossy().into_owned(),
        name,
        is_dir: meta.is_dir(),
        size_bytes: if meta.is_dir() { 0 } else { meta.len() },
        modified_unix,
        extension,
        sniffed_type,
    })
}

#[tauri::command]
pub fn compress_paths_to_zip(
    window: tauri::Window,
    state: State<AppState>,
    paths: Vec<String>,
    destination_dir: String,
    zip_name: String,
) -> Result<TabContentDto, String> {
    if paths.is_empty() {
        return Err("no paths selected".into());
    }
    let dest = resolve_open_path(&destination_dir, &state.cwd)?;
    if !dest.is_dir() {
        return Err("destination is not a directory".into());
    }
    let sources: Vec<PathBuf> = paths
        .iter()
        .map(|p| resolve_open_path(p, &state.cwd))
        .collect::<Result<Vec<_>, _>>()?;
    crate::file_ops::compress_paths_to_zip(&sources, &dest, &zip_name)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn go_back(window: tauri::Window, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    if let Some(loc) = tabs.go_back() {
        let (loc, is_dir, doc) = state.resolve_location(loc);
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
        update_content_webview(&app, window.label(), &loc)?;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn go_forward(window: tauri::Window, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    if let Some(loc) = tabs.go_forward() {
        let (loc, is_dir, doc) = state.resolve_location(loc);
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
        update_content_webview(&app, window.label(), &loc)?;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn reload_tab(window: tauri::Window, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let loc = {
        let _sess_tabs = ws(&state, &window)?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.reload_location()
    };
    if let Some(loc) = loc {
        let (loc, is_dir, doc) = state.resolve_location(loc);
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
        update_content_webview(&app, window.label(), &loc)?;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn begin_directory_listing(state: State<AppState>) -> u64 {
    state.listing_control.register().0
}

#[tauri::command]
pub fn cancel_directory_listing(listing_id: u64, state: State<AppState>) {
    state.listing_control.cancel(listing_id);
}

#[tauri::command]
pub fn list_directory(
    path: String,
    listing_id: Option<u64>,
    timeout_ms: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<base_files::DirEntry>, String> {
    use std::sync::Arc;
    use std::time::Duration;
    let p = PathBuf::from(path);
    let cancel = if let Some(id) = listing_id {
        state
            .listing_control
            .token(id)
            .ok_or("unknown listing id")?
    } else {
        Arc::new(std::sync::atomic::AtomicBool::new(false))
    };
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(30_000).clamp(1_000, 120_000));
    let prefs = load_preferences();
    let result = crate::listing_control::list_dir_with_timeout(&p, cancel, timeout);
    if let Some(id) = listing_id {
        state.listing_control.finish(id);
    }
    let raw = result?;
    Ok(crate::files_listing::apply_files_view(raw, &prefs.files_view))
}

#[tauri::command]
pub fn drop_paths_on_tab(
    window: tauri::Window,
    state: State<AppState>,
    tab_index: usize,
    paths: Vec<String>,
    mode: String,
) -> Result<TabContentDto, String> {
    if paths.is_empty() {
        return Err("no paths to drop".into());
    }
    let copy = match mode.as_str() {
        "copy" => true,
        "move" => false,
        _ => return Err("mode must be copy or move".into()),
    };
    let dest = {
        let sess = ws(&state, &window)?;
        let tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs
            .tabs()
            .get(tab_index)
            .ok_or("invalid tab index")?;
        let path = tab_browse_path(state.inner(), &tabs, tab)
            .ok_or("tab is not a folder browse view")?;
        PathBuf::from(path)
    };
    let sources: Vec<PathBuf> = paths
        .iter()
        .map(|p| resolve_open_path(p, &state.cwd))
        .collect::<Result<Vec<_>, _>>()?;
    crate::file_ops::transfer_paths(&sources, &dest, copy)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn toggle_recent_pin(
    window: tauri::Window,
    path: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _ = resolve_open_path(&path, &state.cwd)?;
    let mut prefs = load_preferences();
    crate::preferences::toggle_recent_pin(&mut prefs, &path);
    save_preferences(&prefs)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn clear_recent_files(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let mut prefs = load_preferences();
    crate::preferences::clear_unpinned_recent(&mut prefs);
    save_preferences(&prefs)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn cancel_filename_search(state: State<AppState>) {
    state
        .filename_search_cancel
        .store(true, std::sync::atomic::Ordering::SeqCst);
}

#[tauri::command]
pub fn search_filenames(
    root: String,
    query: String,
    max_depth: Option<u32>,
    max_results: Option<usize>,
    state: State<AppState>,
) -> Result<Vec<crate::dir_search::FilenameHitDto>, String> {
    use std::sync::atomic::Ordering;
    state.filename_search_cancel.store(false, Ordering::SeqCst);
    let root = resolve_open_path(&root, &state.cwd)?;
    if !root.is_dir() {
        return Err("root is not a directory".into());
    }
    let depth = max_depth.unwrap_or(8).min(16);
    let cap = max_results.unwrap_or(200).clamp(1, 500);
    crate::dir_search::search_filenames(
        &root,
        &query,
        depth,
        cap,
        &state.filename_search_cancel,
    )
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
pub fn get_command_manager(window: tauri::Window, state: State<AppState>) -> Result<CommandManagerDto, String> {
    state.refresh_command_manager_context(window.label());
    let _sess_tabs_guard = ws(&state, &window)?;
    let tabs_guard = _sess_tabs_guard.tabs.lock().map_err(|e| e.to_string())?;
    let active = tabs_guard.active_tab();
    let host = active
        .and_then(|t| t.document.as_ref())
        .map(|d| d.host_context())
        .unwrap_or_else(host_context_for_browse);
    let doc_ref = active.and_then(|t| t.document.as_ref());
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let selected = cm.selected_tab.clone();
    let auto_context = cm.auto_context.tab_id().to_string();
    let mut tab_dtos: Vec<ManagerTabDto> = cm
        .tabs
        .iter()
        .map(|tab| {
            let commands = map_ui_commands(
                cm.resolve_tab_commands(&registry, &host, doc_ref, &tab.id),
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
                cm.resolve_floating_bar(&registry, &host, doc_ref, &bar.id)
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
                coord_space: match bar.coord_space {
                    unibrowser_engine::ToolbarCoordSpace::Viewport => "viewport",
                    unibrowser_engine::ToolbarCoordSpace::Screen => "screen",
                }
                .into(),
                commands,
            }
        })
        .collect();
    if let Some(vsix_root) = crate::config::vsix_extensions_dir() {
        let installed = unibrowser_vsix::list_installed(&vsix_root).unwrap_or_default();
        let details = unibrowser_vsix::enabled_command_details(&installed);
        if !details.is_empty() {
            tab_dtos.push(ManagerTabDto {
                id: "vsix".into(),
                label: "Extensions".into(),
                phase: phase_name(ToolbarPhase::Active).into(),
                selected: false,
                commands: details
                    .into_iter()
                    .map(|d| UiCommandDto {
                        command_id: d.command_id,
                        title: d.title,
                        icon: "vsix".into(),
                        enabled: true,
                        reason: Some(unibrowser_extension_host::VSIX_STUB_ENABLED_HINT.into()),
                    })
                    .collect(),
            });
        }
    }
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

fn export_command_manager_layout(state: State<AppState>) -> Result<String, String> {
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&cm.to_file()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_command_manager_layout_file(window: tauri::Window, 
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let json = export_command_manager_layout(state)?;
    let picked = app
        .dialog()
        .file()
        .set_title("Export command manager layout")
        .add_filter("JSON", &["json"])
        .set_file_name("command-manager.json")
        .blocking_save_file();
    if let Some(path) = picked {
        let dest = PathBuf::from(path.to_string());
        std::fs::write(&dest, json).map_err(|e| e.to_string())?;
        Ok(Some(dest.to_string_lossy().into_owned()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn import_command_manager_layout_file(window: tauri::Window, 
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Import command manager layout")
        .add_filter("JSON", &["json"])
        .blocking_pick_file();
    let path = picked.ok_or_else(|| "import cancelled".to_string())?;
    let data = std::fs::read_to_string(path.to_string()).map_err(|e| e.to_string())?;
    let layout: CommandManagerFile = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    apply_command_manager_layout(&layout, &state)
}

fn apply_command_manager_layout(layout: &CommandManagerFile, state: &AppState) -> Result<(), String> {
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, layout)?;
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    *cm = unibrowser_engine::CommandManagerState::from_file(
        layout,
        cm.auto_context,
    );
    Ok(())
}

#[tauri::command]
pub fn save_command_manager_layout(
    _window: tauri::Window,
    layout: CommandManagerFile,
    state: State<AppState>,
) -> Result<(), String> {
    apply_command_manager_layout(&layout, state.inner())
}

#[tauri::command]
pub fn list_workspace_layout_presets(window: tauri::Window) -> Vec<crate::workspace_layouts::WorkspaceLayoutPresetSummaryDto> {
    crate::workspace_layouts::list_presets()
}

#[tauri::command]
pub fn save_workspace_layout_preset(
    name: String,
    window: tauri::Window,
    state: State<AppState>,
) -> Result<crate::workspace_layouts::WorkspaceLayoutPresetSummaryDto, String> {
    let window_state = crate::preferences::capture_window_state(&window)?;
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let file = cm.to_file();
    let prefs = load_preferences();
    let _sess_split = ws(&state, &window)?;
    let split = _sess_split.split_view.lock().map_err(|e| e.to_string())?;
    let chrome = crate::workspace_layouts::WorkspaceChromeState {
        pinned_locations: if prefs.pinned_locations.is_empty() {
            default_pinned_locations(
                &state
                    .cwd
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            prefs.pinned_locations.clone()
        },
        split_view_enabled: split.enabled,
        split_secondary_index: split.secondary_index,
    };
    crate::workspace_layouts::save_preset(name, window_state, file, chrome)
}

#[tauri::command]
pub fn load_workspace_layout_preset(
    id: String,
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let preset = crate::workspace_layouts::load_preset(&id)?;
    if window.label() == "main" || app.get_webview_window(window.label()).is_some() {
        crate::preferences::apply_window_state(&window, &preset.window)?;
        let _ = crate::preferences::save_window_state(&preset.window);
    }
    let mut prefs = load_preferences();
    prefs.pinned_locations = preset.chrome.pinned_locations.clone();
    save_preferences(&prefs)?;
    {
        let _sess_split = ws(&state, &window)?;
        let mut split = _sess_split.split_view.lock().map_err(|e| e.to_string())?;
        split.enabled = preset.chrome.split_view_enabled;
        split.secondary_index = preset.chrome.split_secondary_index;
    }
    apply_command_manager_layout(&preset.command_manager, state.inner())?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn delete_workspace_layout_preset(window: tauri::Window, id: String) -> Result<(), String> {
    crate::workspace_layouts::delete_preset(&id)
}

#[tauri::command]
pub fn select_command_manager_tab(window: tauri::Window, tab_id: String, state: State<AppState>) -> Result<(), String> {
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    if cm.tabs.iter().any(|t| t.id == tab_id) {
        cm.selected_tab = tab_id;
    }
    Ok(())
}

#[tauri::command]
pub fn execute_command(window: tauri::Window, 
    command_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Ok("palette".into());
    }
    if command_id == unibrowser_mod_api::commands::SCRIPT_DEMO_HELLO {
        let allow_doc = {
            let _sess_tabs = ws(&state, &window)?;
            let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
            tabs
                .active_tab()
                .and_then(|t| t.document.as_ref())
                .map(|d| d.backend.backend_id() == base_text::BACKEND_ID)
                .unwrap_or(false)
        };
        let source = if allow_doc {
            crate::rhai_host::DEMO_HELLO_SCRIPT
        } else {
            crate::rhai_host::DEMO_HELLO_SCRIPT_BROWSE
        };
        let out = crate::rhai_host::eval_rhai_script(state.inner(), source, allow_doc)?;
        let _ = app.emit("unibrowser-script-output", &out);
        return Ok(out);
    }
    {
        let host_cmds = state
            .vsix_host_commands
            .lock()
            .map_err(|e| e.to_string())?;
        if host_cmds.contains(&command_id) {
            let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
            let (extension_id, activation_events) =
                unibrowser_vsix::lookup_vsix_command(&root, &command_id)
                    .ok_or_else(|| format!("VSIX command not found: {}", command_id))?;
            return Ok(unibrowser_extension_host::execute_vsix_stub(
                &command_id,
                &extension_id,
                &activation_events,
            ));
        }
    }
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    if tab.document.is_none() {
        return Err("command needs an open document".into());
    }
    let doc = tab.document.as_mut().unwrap();
    if doc.backend.backend_id() == base_web::BACKEND_ID
        && (command_id == FONT_INCREASE_ID || command_id == FONT_DECREASE_ID)
    {
        let delta = if command_id == FONT_INCREASE_ID { 1 } else { -1 };
        let url = match &tab.location {
            BrowserLocation::Http { url } => url.clone(),
            _ => return Err("not a web tab".into()),
        };
        let px = adjust_page_font_for_url(&app, &url, delta)?;
        return Ok(format!("Page font {}px (saved for origin)", px));
    }
    state
        .registry
        .lock()
        .map_err(|e| e.to_string())?
        .execute(&command_id, doc)
}

#[tauri::command]
pub fn sync_text_document(window: tauri::Window, 
    content: String,
    selection_start: usize,
    selection_end: usize,
    state: State<AppState>,
) -> Result<(), String> {
    if content.len() > crate::document_limits::MAX_SYNC_TEXT_CHARS {
        return Err(crate::document_limits::sync_limit_error(content.len()));
    }
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no text document")?;
    let backend = try_text_backend(&mut *doc.backend).ok_or("not a text document")?;
    let before = backend.content.clone();
    let path = backend.path.clone();
    backend.set_content(content);
    backend.set_selection_range(selection_start, selection_end);
    let _sess_stacks = ws(&state, &window)?;
    let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
    stacks.record_change(&path, before, &backend.content);
    if backend.dirty {
        crate::crash_recovery::maybe_backup_dirty(&path, base_text::BACKEND_ID, &backend.content);
    }
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
pub fn get_keybindings(window: tauri::Window, state: State<AppState>) -> Result<Vec<KeybindingDto>, String> {
    let kb = state.keybindings.lock().map_err(|e| e.to_string())?;
    Ok(kb
        .bindings_for_display()
        .into_iter()
        .map(|(command_id, chord)| KeybindingDto {
            command_id,
            chord,
        })
        .collect())
}

/// Re-read `keybindings.json` and VSIX chords (user overrides win over defaults).
#[tauri::command]
pub fn reload_keybindings(state: State<AppState>) -> Result<Vec<KeybindingDto>, String> {
    let merged = crate::state::load_merged_keybindings();
    let mut kb = state.keybindings.lock().map_err(|e| e.to_string())?;
    *kb = merged;
    Ok(kb
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
pub fn dispatch_key_chord(window: tauri::Window, 
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
    let kb = state.keybindings.lock().map_err(|e| e.to_string())?;
    if let Some(cmd) = kb.command_for_chord(&key) {
        state
            .key_sequences
            .lock()
            .map_err(|e| e.to_string())?
            .reset();
        return Ok(Some(cmd.to_string()));
    }
    let mut seq = state.key_sequences.lock().map_err(|e| e.to_string())?;
    Ok(seq.on_chord(&key))
}

#[tauri::command]
pub fn palette_search(window: tauri::Window, query: String, state: State<AppState>) -> Result<Vec<PaletteEntryDto>, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
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
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let keybindings = state.keybindings.lock().map_err(|e| e.to_string())?;
    let host_cmds = state
        .vsix_host_commands
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(search_commands(&registry, &ctx, &query)
        .into_iter()
        .map(|e| {
            let (enabled, reason) = if host_cmds.contains(&e.command_id) {
                (
                    true,
                    Some(unibrowser_extension_host::VSIX_STUB_ENABLED_HINT.to_string()),
                )
            } else {
                (
                    e.enablement.is_enabled(),
                    e.enablement.disabled_reason().map(String::from),
                )
            };
            let chord = keybindings
                .chord_for_command(&e.command_id)
                .map(|c| c.display());
            PaletteEntryDto {
                command_id: e.command_id,
                title: e.title,
                chord,
                enabled,
                reason,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn handle_download_url(window: tauri::Window, 
    url: String,
    destination: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TabContentDto, String> {
    use tauri_plugin_dialog::DialogExt;

    let prefs = load_preferences();
    let has_dest_hint = destination.is_some();
    let hinted = destination.as_ref().map(|s| PathBuf::from(s.as_str()));
    let disposition = apply_download_disposition(
        &url,
        hinted,
        &state.cwd,
        prefs.download_preference,
    )?;
    match disposition {
        DownloadDisposition::OpenInNewTab { suggested_location } => {
            let (loc, is_dir, doc) = state.resolve_location(suggested_location);
            let _sess_tabs = ws(&state, &window)?;
            let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
            tabs.open_tab(loc.clone());
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
            }
            update_content_webview(&app, window.label(), &loc)?;
            snapshot(&state, window.label())
        }
        DownloadDisposition::SaveToDisk { path } => {
            let dest = if has_dest_hint {
                path
            } else {
                let default = path;
                let picked = app
                    .dialog()
                    .file()
                    .set_title("Save download")
                    .set_file_name(
                        default
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| suggested_download_filename(&url)),
                    )
                    .blocking_save_file();
                picked
                    .map(|p| PathBuf::from(p.to_string()))
                    .ok_or_else(|| "save cancelled".to_string())?
            };
            let trimmed = url.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                download_http_to_file(trimmed, &dest)?;
            } else {
                let source = if trimmed.starts_with("file://") {
                    resolve_open_path(trimmed.trim_start_matches("file://"), &state.cwd)?
                } else {
                    resolve_open_path(trimmed, &state.cwd)?
                };
                copy_file_to_path(&source, &dest)?;
            }
            *ws(&state, &window)?
                .chrome_notice
                .lock()
                .map_err(|e| e.to_string())? = Some(format!("Saved to {}", dest.display()));
            snapshot(&state, window.label())
        }
    }
}

#[tauri::command]
pub fn set_content_bounds(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    window_label: String,
    app: AppHandle,
) -> Result<(), String> {
    sync_content_bounds(&app, &window_label, x, y, width, height)
}

#[derive(Serialize)]
pub struct PreferencesDto {
    pub onboarding_dismissed: bool,
    pub onboarding_tour_skipped_forever: bool,
    pub ui_locale: String,
    pub onboarding_checklist_skip_forever: bool,
    pub onboarding_checklist_completed: Vec<String>,
    pub theme: String,
    pub command_manager_dense: bool,
    pub high_contrast: bool,
    pub reduce_motion: String,
    pub restore_session_on_launch: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval_secs: u32,
    pub download_preference: String,
    pub board_snap_to_grid: bool,
    pub midi_scrub_audio_enabled: bool,
    pub project_autosave_enabled: bool,
    pub project_autosave_interval_secs: u32,
    pub crash_recovery_enabled: bool,
    pub ntp_sections: NtpSectionsDto,
    pub advanced_webview_devtools: bool,
    pub advanced_experimental_spellcheck: bool,
    pub open_downloads_in_system_browser: bool,
    pub files_view: FilesViewDto,
}

#[tauri::command]
pub fn read_document_bytes(path: String, state: State<AppState>) -> Result<Vec<u8>, String> {
    let p = resolve_open_path(&path, &state.cwd)?;
    if !p.is_file() {
        return Err("not a file".into());
    }
    let meta = std::fs::metadata(&p).map_err(|e| e.to_string())?;
    let len = meta.len() as usize;
    if len > crate::document_limits::MAX_READ_BYTES {
        return Err(crate::document_limits::read_limit_error(len));
    }
    std::fs::read(&p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_document_bytes(path: String, bytes: Vec<u8>, state: State<AppState>) -> Result<(), String> {
    if bytes.len() > crate::document_limits::MAX_WRITE_BYTES {
        return Err(crate::document_limits::write_limit_error(bytes.len()));
    }
    let p = resolve_open_path(&path, &state.cwd)?;
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            return Err("parent directory does not exist".into());
        }
    }
    std::fs::write(&p, bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_save_image_path(window: tauri::Window, 
    app: AppHandle,
    suggested_name: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Save image")
        .set_file_name(suggested_name)
        .add_filter("PNG", &["png"])
        .blocking_save_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn resolve_preview_link(window: tauri::Window, 
    base_path: String,
    href: String,
    state: State<AppState>,
) -> Result<String, String> {
    let trimmed = href.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("javascript:") || lower.starts_with("data:") {
        return Err("unsupported link scheme".into());
    }
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Ok(trimmed.to_string());
    }
    if lower.starts_with("file://") {
        let p = resolve_open_path(trimmed.trim_start_matches("file://"), &state.cwd)?;
        return Ok(format!("file://{}", p.display()));
    }
    let base = resolve_open_path(&base_path, &state.cwd)?;
    let parent = base.parent().ok_or("base file has no parent directory")?;
    let joined = parent.join(trimmed);
    let resolved = resolve_open_path(&joined.to_string_lossy(), &state.cwd)?;
    Ok(format!("file://{}", resolved.display()))
}

#[tauri::command]
pub fn open_new_browser_window(window: tauri::Window, app: AppHandle) -> Result<String, String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    let mut n = app.webview_windows().len().max(1) + 1;
    let mut label = format!("browser-{}", n);
    while app.get_webview_window(&label).is_some() {
        n += 1;
        label = format!("browser-{}", n);
    }
    let _window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .title("Unibrowser")
        .inner_size(1200.0, 800.0)
        .build()
        .map_err(|e| e.to_string())?;
    crate::webview::ensure_content_webview(&app, &label)?;
    Ok(label)
}

#[tauri::command]
pub fn sync_pdf_viewer_state(window: tauri::Window, 
    has_text_selection: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let pdf = base_pdf::try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    pdf.set_text_selected(has_text_selection);
    Ok(())
}

#[tauri::command]
pub fn sync_pdf_page(window: tauri::Window, page: u32, state: State<AppState>) -> Result<(), String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let pdf = base_pdf::try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    pdf.set_page_index(page);
    Ok(())
}

#[tauri::command]
pub fn ensure_pdf_raster_proxy(path: String) -> Result<Option<String>, String> {
    let p = Path::new(&path);
    if !p.is_file() {
        return Err("not a file".into());
    }
    match crate::pdf_preview::pdf_first_page_preview(p) {
        Ok(out) if out.is_file() => Ok(Some(out.to_string_lossy().into_owned())),
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn sync_pdf_scale(window: tauri::Window, scale: f32, state: State<AppState>) -> Result<(), String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let pdf = base_pdf::try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    pdf.set_render_scale(scale);
    Ok(())
}

#[tauri::command]
pub fn web_page_zoom(window: tauri::Window, delta_px: i32, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no tab")?;
    let url = match &tab.location {
        BrowserLocation::Http { url } => url.clone(),
        _ => return Err("not a web tab".into()),
    };
    adjust_page_font_for_url(&app, &url, delta_px)?;
    Ok(())
}

#[tauri::command]
pub async fn pick_open_file(window: tauri::Window, app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Open a file")
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn pick_browse_folder(window: tauri::Window, app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Choose a folder")
        .blocking_pick_folder();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn pick_unibrow_asset_file(window: tauri::Window, app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Add asset to project")
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn add_unibrow_asset(window: tauri::Window, 
    project_path: String,
    source_path: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    if !unibrowser_project::is_unibrow_path(&project) {
        return Err("not a .unibrow project".into());
    }
    let source = resolve_open_path(&source_path, &state.cwd)?;
    if !source.is_file() {
        return Err("source is not a file".into());
    }
    let mut package = open_unibrow_package(&project)?;
    let node_type = unibrowser_project::guess_asset_node_type(&source);
    let label = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "asset".into());
    unibrowser_project::add_asset_copy(&mut package, &source, node_type, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

fn reload_unibrow_project_tab(
    state: &AppState,
    window_label: &str,
    project: &Path,
    project_path: String,
) -> Result<(), String> {
    let package = open_unibrow_package(project)?;
    crate::project_recovery::maybe_backup_project(&project_path, &package.root);
    let _sess_tabs = ws_label(state, window_label)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
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
pub fn remove_unibrow_node(window: tauri::Window, 
    project_path: String,
    node_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::remove_node(&mut package, &node_id).map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_unibrow_node_position(window: tauri::Window, 
    project_path: String,
    node_id: String,
    x: f32,
    y: f32,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::set_node_board_position(&mut package, &node_id, x, y)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn add_unibrow_timeline_clip(window: tauri::Window, 
    project_path: String,
    node_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::append_timeline_clip_for_node(&mut package, &node_id)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn remove_unibrow_timeline_clip(window: tauri::Window, 
    project_path: String,
    clip_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::remove_timeline_clip(&mut package, &clip_id)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn update_unibrow_timeline_clip(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    start_sec: f32,
    duration_sec: f32,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::update_timeline_clip_timing(
        &mut package,
        &clip_id,
        start_sec,
        duration_sec,
    )
    .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn video_scrub_thumbnail(
    window: tauri::Window,
    path: String,
    time_sec: f64,
) -> Result<unibrowser_media::frame::VideoFrameThumbnail, String> {
    let p = PathBuf::from(&path);
    if !p.is_file() {
        return Err("file not found".into());
    }
    unibrowser_media::video_frame_thumbnail(&p, time_sec).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct Mp4PreviewExportDto {
    pub path: String,
    /// `encoded` when ffmpeg concat succeeded; otherwise `stub`.
    pub mode: String,
    pub message: String,
}

#[tauri::command]
pub fn export_unibrow_mp4_preview(
    window: tauri::Window,
    project_path: String,
) -> Result<Mp4PreviewExportDto, String> {
    let project = PathBuf::from(&project_path);
    if !unibrowser_project::is_unibrow_path(&project) {
        return Err("not a .unibrow project".into());
    }
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(&project, &cache).map_err(|e| e.to_string())?;
    let pkg = unibrowser_project::open_package(&root).map_err(|e| e.to_string())?;
    if pkg.manifest.timeline.clips.is_empty() {
        let path = unibrowser_project::write_mp4_export_stub(&pkg, "timeline has no clips")
            .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: "Added export notes under derived/ (no clips).".into(),
        });
    }
    let resolved = unibrowser_project::clip_asset_paths(&pkg);
    if resolved.len() != pkg.manifest.timeline.clips.len() {
        let path = unibrowser_project::write_mp4_export_stub(
            &pkg,
            "some clips are missing board assets",
        )
        .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: "Added export notes under derived/ (missing assets).".into(),
        });
    }
    if !resolved.iter().all(|(_, _, ty)| ty == "video") {
        let path = unibrowser_project::write_mp4_export_stub(
            &pkg,
            "preview concat needs video-only clips",
        )
        .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: "Added export notes under derived/ (mixed clip types).".into(),
        });
    }
    let probe = unibrowser_media::probe_ffmpeg_install();
    if !probe.available {
        let path = unibrowser_project::write_mp4_export_stub(&pkg, "ffmpeg not on PATH")
            .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: "Added export notes under derived/ (install ffmpeg for concat).".into(),
        });
    }
    let inputs: Vec<PathBuf> = resolved.into_iter().map(|(_, p, _)| p).collect();
    let dest = unibrowser_project::derived_mp4_preview_path(&pkg);
    if let Err(err) = unibrowser_media::concat_mp4_videos(&inputs, &dest) {
        let reason = err.to_string();
        let path = unibrowser_project::write_mp4_export_stub(&pkg, &reason)
            .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: format!("Concat failed; wrote notes: {}", reason),
        });
    }
    Ok(Mp4PreviewExportDto {
        path: dest.to_string_lossy().into_owned(),
        mode: "encoded".into(),
        message: "Concatenated MP4 clips to derived/timeline-preview.mp4.".into(),
    })
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeProgressPayload {
    pub percent: u8,
    pub stage: String,
}

fn resolve_timeline_mp4_inputs(project_path: &Path) -> Result<(unibrowser_project::ProjectPackage, Vec<PathBuf>), String> {
    if !unibrowser_project::is_unibrow_path(project_path) {
        return Err("not a .unibrow project".into());
    }
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(project_path, &cache).map_err(|e| e.to_string())?;
    let pkg = unibrowser_project::open_package(&root).map_err(|e| e.to_string())?;
    if pkg.manifest.timeline.clips.is_empty() {
        return Err("timeline has no clips".into());
    }
    let resolved = unibrowser_project::clip_asset_paths(&pkg);
    if resolved.len() != pkg.manifest.timeline.clips.len() {
        return Err("some clips are missing board assets".into());
    }
    if !resolved.iter().all(|(_, _, ty)| ty == "video") {
        return Err("export needs video-only mp4 clips".into());
    }
    let inputs: Vec<PathBuf> = resolved.into_iter().map(|(_, p, _)| p).collect();
    Ok((pkg, inputs))
}

#[tauri::command]
pub async fn export_unibrow_mp4_h264(
    window: tauri::Window,
    project_path: String,
    preset: Option<String>,
) -> Result<Mp4PreviewExportDto, String> {
    use tauri::Emitter;
    let preset = preset.unwrap_or_else(|| "fast".to_string());
    let (pkg, inputs) = resolve_timeline_mp4_inputs(Path::new(&project_path))?;
    let probe = unibrowser_media::probe_ffmpeg_install();
    if !probe.available {
        let path = unibrowser_project::write_mp4_export_stub(&pkg, "ffmpeg not on PATH")
            .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: "Install ffmpeg for H.264 export.".into(),
        });
    }
    let dest = unibrowser_project::derived_mp4_h264_path(&pkg);
    let dest_display = dest.to_string_lossy().into_owned();
    let dest_encode = dest.clone();
    let app = window.app_handle().clone();
    let label = window.label().to_string();
    let preset_copy = preset.clone();
    let encode_result = tauri::async_runtime::spawn_blocking(move || {
        unibrowser_media::encode_h264_timeline(
            &inputs,
            &dest_encode,
            &preset_copy,
            &mut |percent, stage| {
                let _ = app.emit_to(
                    &label,
                    "unibrowser-encode-progress",
                    EncodeProgressPayload {
                        percent,
                        stage: stage.to_string(),
                    },
                );
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?;
    if let Err(err) = encode_result {
        let reason = err.to_string();
        let path = unibrowser_project::write_mp4_export_stub(&pkg, &reason)
            .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: format!("H.264 export failed: {}", reason),
        });
    }
    Ok(Mp4PreviewExportDto {
        path: dest_display,
        mode: "encoded".into(),
        message: format!(
            "Exported H.264 ({}) to derived/timeline-export-h264.mp4.",
            preset
        ),
    })
}

#[tauri::command]
pub fn batch_convert_images(
    window: tauri::Window,
    paths: Vec<String>,
    format: String,
) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Err("no paths selected".into());
    }
    crate::batch_images::batch_convert_images(&paths, &format)
}

#[tauri::command]
pub fn timeline_clip_scrub_preview(
    window: tauri::Window,
    state: State<AppState>,
    project_path: String,
    clip_id: String,
    offset_sec: f64,
) -> Result<TimelineScrubPreviewDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let package = open_unibrow_package(&project)?;
    let clip = package
        .manifest
        .timeline
        .clips
        .iter()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| format!("unknown clip {}", clip_id))?;
    let node = package
        .manifest
        .nodes
        .iter()
        .find(|n| n.id == clip.node_id)
        .ok_or_else(|| "clip node missing".to_string())?;
    let asset_name = node.asset.as_ref().ok_or("clip has no asset")?;
    let asset_path = resolve_unibrow_asset_file(&project, asset_name)?;
    let node_type = node.node_type.to_ascii_lowercase();
    if node_type == "video" {
        let frame = unibrowser_media::video_frame_thumbnail(&asset_path, offset_sec)
            .map_err(|e| e.to_string())?;
        return Ok(TimelineScrubPreviewDto {
            kind: "video".into(),
            image_path: Some(frame.path),
            image_mime: Some(frame.mime),
            midi_notes: None,
        });
    }
    if node_type == "pdf" {
        let path = crate::pdf_preview::pdf_first_page_preview(&asset_path)?;
        return Ok(TimelineScrubPreviewDto {
            kind: "pdf".into(),
            image_path: Some(path.to_string_lossy().into_owned()),
            image_mime: Some("image/jpeg".into()),
            midi_notes: None,
        });
    }
    if node_type == "midi" {
        let notes = crate::smf::note_on_events_near_time(
            &asset_path,
            offset_sec,
            clip.duration_sec as f64,
            480,
        )?;
        let mapped = notes
            .into_iter()
            .map(|n| MidiNoteOnDto {
                track_index: n.track_index,
                tick: n.tick,
                note: n.note,
                velocity: n.velocity,
            })
            .collect();
        return Ok(TimelineScrubPreviewDto {
            kind: "midi".into(),
            image_path: None,
            image_mime: None,
            midi_notes: Some(mapped),
        });
    }
    if node_type == "audio" {
        return Ok(TimelineScrubPreviewDto {
            kind: "audio".into(),
            image_path: None,
            image_mime: None,
            midi_notes: None,
        });
    }
    Err(format!("scrub preview not available for {}", node_type))
}

#[tauri::command]
pub fn set_unibrow_node_color(
    window: tauri::Window,
    project_path: String,
    node_id: String,
    color: Option<String>,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    let raw = color.as_deref();
    unibrowser_project::set_node_label_color(&mut package, &node_id, raw)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_unibrow_timeline_clip_color(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    color: Option<String>,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::set_timeline_clip_color(&mut package, &clip_id, color.as_deref())
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn add_unibrow_timeline_marker(
    window: tauri::Window,
    project_path: String,
    time_sec: f32,
    title: String,
    color: Option<String>,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::add_timeline_marker(&mut package, time_sec, title, color.as_deref())
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn remove_unibrow_timeline_marker(
    window: tauri::Window,
    project_path: String,
    marker_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::remove_timeline_marker(&mut package, &marker_id)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_unibrow_timeline_transition(
    window: tauri::Window,
    project_path: String,
    after_clip_id: String,
    enabled: bool,
    duration_sec: f32,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    if enabled {
        unibrowser_project::upsert_fade_transition(&mut package, &after_clip_id, duration_sec)
            .map_err(|e| e.to_string())?;
    } else {
        unibrowser_project::remove_timeline_transition(&mut package, &after_clip_id)
            .map_err(|e| e.to_string())?;
    }
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn reorder_unibrow_timeline_clips(window: tauri::Window, 
    project_path: String,
    from_index: usize,
    to_index: usize,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::reorder_timeline_clip(&mut package, from_index, to_index)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn rename_unibrow_node(window: tauri::Window, 
    project_path: String,
    node_id: String,
    label: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::rename_node_label(&mut package, &node_id, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn get_ffmpeg_probe(window: tauri::Window) -> unibrowser_media::FfmpegProbe {
    unibrowser_media::probe_ffmpeg_install()
}

#[tauri::command]
pub fn get_home_dir(window: tauri::Window) -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())
}

#[derive(Serialize)]
pub struct BuildInfoDto {
    pub version: String,
    pub version_line: String,
    pub git_hash: Option<String>,
}

#[tauri::command]
pub fn get_build_info() -> BuildInfoDto {
    BuildInfoDto {
        version: unibrowser_build_info::VERSION.into(),
        version_line: unibrowser_build_info::version_line(),
        git_hash: unibrowser_build_info::GIT_HASH.map(|s| s.to_string()),
    }
}

#[tauri::command]
fn preferences_dto() -> PreferencesDto {
    let p = load_preferences();
    PreferencesDto {
        onboarding_dismissed: p.onboarding_dismissed,
        onboarding_tour_skipped_forever: p.onboarding_tour_skipped_forever,
        ui_locale: p.ui_locale.clone(),
        onboarding_checklist_skip_forever: p.onboarding_checklist_skip_forever,
        onboarding_checklist_completed: p.onboarding_checklist_completed.clone(),
        theme: theme_to_str(&p.theme),
        command_manager_dense: p.command_manager_dense,
        high_contrast: p.high_contrast,
        reduce_motion: reduce_motion_to_str(&p.reduce_motion),
        restore_session_on_launch: p.restore_session_on_launch,
        auto_save_enabled: p.auto_save_enabled,
        auto_save_interval_secs: p.auto_save_interval_secs,
        download_preference: download_pref_to_str(&p.download_preference),
        board_snap_to_grid: p.board_snap_to_grid,
        midi_scrub_audio_enabled: p.midi_scrub_audio_enabled,
        project_autosave_enabled: p.project_autosave_enabled,
        project_autosave_interval_secs: p.project_autosave_interval_secs,
        crash_recovery_enabled: p.crash_recovery_enabled,
        ntp_sections: ntp_sections_dto(&p),
        advanced_webview_devtools: p.advanced_webview_devtools,
        advanced_experimental_spellcheck: p.advanced_experimental_spellcheck,
        open_downloads_in_system_browser: p.open_downloads_in_system_browser,
        files_view: files_view_dto(&p),
    }
}

fn parse_files_sort_mode(s: &str) -> crate::preferences::FilesSortMode {
    match s.to_ascii_lowercase().as_str() {
        "date" => crate::preferences::FilesSortMode::Date,
        "size" => crate::preferences::FilesSortMode::Size,
        _ => crate::preferences::FilesSortMode::Name,
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesViewInput {
    pub sort: String,
    pub sort_ascending: bool,
    pub show_hidden: bool,
}

#[tauri::command]
pub fn set_files_view_prefs(input: FilesViewInput) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.files_view = crate::preferences::FilesViewPrefs {
        sort: parse_files_sort_mode(&input.sort),
        sort_ascending: input.sort_ascending,
        show_hidden: input.show_hidden,
    };
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[derive(serde::Deserialize)]
pub struct NtpSectionsInput {
    pub show_hero: bool,
    pub show_actions: bool,
    pub show_browse: bool,
    pub show_recent: bool,
}

#[tauri::command]
pub fn set_ntp_sections(sections: NtpSectionsInput) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.ntp_sections = crate::preferences::NtpSectionPrefs {
        show_hero: sections.show_hero,
        show_actions: sections.show_actions,
        show_browse: sections.show_browse,
        show_recent: sections.show_recent,
    };
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_advanced_webview_devtools(
    window: tauri::Window,
    app: AppHandle,
    enabled: bool,
) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.advanced_webview_devtools = enabled;
    save_preferences(&p)?;
    crate::webview::set_content_webview_devtools(&app, window.label(), enabled)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_advanced_experimental_spellcheck(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.advanced_experimental_spellcheck = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_open_downloads_in_system_browser(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.open_downloads_in_system_browser = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn open_url_externally(url: String, app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("empty url".into());
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("only http(s) urls supported".into());
    }
    app.opener().open_url(trimmed, None::<&str>).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_preferences(_window: tauri::Window) -> PreferencesDto {
    preferences_dto()
}

#[tauri::command]
pub fn set_crash_recovery_enabled(_window: tauri::Window, enabled: bool) -> Result<PreferencesDto, String> {
    crate::crash_recovery::set_enabled(enabled)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn list_crash_recovery_backups(window: tauri::Window) -> Vec<crate::crash_recovery::CrashRecoveryBackupDto> {
    crate::crash_recovery::list_backups()
}

#[tauri::command]
pub fn dismiss_crash_recovery_backup(window: tauri::Window, backup_id: String) -> Result<(), String> {
    crate::crash_recovery::dismiss_backup(&backup_id)
}

#[tauri::command]
pub fn restore_crash_recovery_backup(window: tauri::Window, 
    backup_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let record = crate::crash_recovery::load_backup(&backup_id)?;
    let file_path = resolve_open_path(&record.path, &state.cwd)?;
    allow_asset_protocol_file(&app, &file_path);
    let loc = BrowserLocation::File { path: file_path };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.navigate_active(loc.clone(), is_dir, doc);
    if let BrowserLocation::File { path } = &loc {
        if path.is_file() {
            touch_recent_file(path);
        }
    }
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    if doc.path != record.path {
        return Err("restore path mismatch".into());
    }
    if record.backend_id == base_text::BACKEND_ID {
        let backend = try_text_backend(&mut *doc.backend).ok_or("not a text document")?;
        backend.set_content(record.content);
    } else if record.backend_id == base_svg::BACKEND_ID {
        let backend = base_svg::try_svg_backend(&mut *doc.backend).ok_or("not an SVG document")?;
        backend.set_content(record.content);
    } else {
        return Err("unsupported backup type".into());
    }
    update_content_webview(&app, window.label(), &loc)?;
    let _ = crate::crash_recovery::dismiss_backup(&backup_id);
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn list_project_recovery_backups(_window: tauri::Window) -> Vec<crate::project_recovery::ProjectRecoveryBackupDto> {
    crate::project_recovery::list_backups()
}

#[tauri::command]
pub fn dismiss_project_recovery_backup(_window: tauri::Window, backup_id: String) -> Result<(), String> {
    crate::project_recovery::dismiss_backup(&backup_id)
}

#[tauri::command]
pub fn restore_project_recovery_backup(
    window: tauri::Window,
    backup_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let record = crate::project_recovery::load_backup(&backup_id)?;
    crate::project_recovery::restore_backup_to_disk(&record)?;
    let file_path = resolve_open_path(&record.project_path, &state.cwd)?;
    allow_asset_protocol_file(&app, &file_path);
    let loc = BrowserLocation::File { path: file_path };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.navigate_active(loc.clone(), is_dir, doc);
    if let BrowserLocation::File { path } = &loc {
        if path.is_file() {
            touch_recent_file(path);
        }
    }
    update_content_webview(&app, window.label(), &loc)?;
    let _ = crate::project_recovery::dismiss_backup(&backup_id);
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_board_snap_to_grid(window: tauri::Window, enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.board_snap_to_grid = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_midi_scrub_audio_enabled(
    _window: tauri::Window,
    enabled: bool,
) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.midi_scrub_audio_enabled = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

fn download_pref_to_str(pref: &DownloadPreference) -> String {
    match pref {
        DownloadPreference::OpenInNewTab => "open_in_new_tab".into(),
        DownloadPreference::SaveToDisk => "save_to_disk".into(),
    }
}

fn download_pref_from_str(s: &str) -> DownloadPreference {
    match s {
        "save_to_disk" => DownloadPreference::SaveToDisk,
        _ => DownloadPreference::OpenInNewTab,
    }
}

#[tauri::command]
pub fn set_download_preference(window: tauri::Window, preference: String) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.download_preference = download_pref_from_str(&preference);
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_auto_save_settings(window: tauri::Window, 
    enabled: bool,
    interval_secs: u32,
) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.auto_save_enabled = enabled;
    p.auto_save_interval_secs = interval_secs.clamp(30, 600);
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn auto_save_active_document(window: tauri::Window, state: State<AppState>) -> Result<String, String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no active tab")?;
    if tab.document.is_none() {
        return Ok(String::new());
    }
    let doc = tab.document.as_mut().unwrap();
    if !doc.backend.is_dirty() {
        return Ok(String::new());
    }
    let backend_id = doc.backend.backend_id();
    if backend_id != base_text::BACKEND_ID && backend_id != base_svg::BACKEND_ID {
        return Ok(String::new());
    }
    state
        .registry
        .lock()
        .map_err(|e| e.to_string())?
        .execute(TEXT_SAVE, doc)
        .map_err(|e| e.to_string())?;
    Ok("saved".into())
}

#[tauri::command]
pub fn begin_file_compare(window: tauri::Window, 
    other_path: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let left = active_document_path(&state, &window)?;
    let right = resolve_open_path(&other_path, &state.cwd)?;
    if !right.is_file() {
        return Err("compare target is not a file".into());
    }
    {
        let _sess_cmp = ws(&state, &window)?;
        let mut cmp = _sess_cmp.file_compare.lock().map_err(|e| e.to_string())?;
        *cmp = Some(FileCompareState {
            left_path: left,
            right_path: right.to_string_lossy().into_owned(),
        });
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub async fn pick_and_compare(window: tauri::Window, 
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TabContentDto, String> {
    use tauri_plugin_dialog::DialogExt;
    let _ = active_document_path(&state, &window)?;
    let picked = app
        .dialog()
        .file()
        .set_title("Compare with file")
        .blocking_pick_file();
    let path = picked.map(|p| p.to_string()).ok_or("no file chosen")?;
    begin_file_compare(window, path, state)
}

#[tauri::command]
pub fn close_file_compare(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    {
        let _sess_cmp = ws(&state, &window)?;
        let mut cmp = _sess_cmp.file_compare.lock().map_err(|e| e.to_string())?;
        *cmp = None;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn pin_current_location(window: tauri::Window, state: State<AppState>) -> Result<Vec<PinnedLocationDto>, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let omnibox = tabs
        .active_tab()
        .map(|t| location_to_omnibox(&t.location))
        .ok_or("no active tab")?;
    let label = tabs
        .active_tab()
        .map(|t| t.location.display_label())
        .unwrap_or_else(|| "Pinned".into());
    let mut prefs = load_preferences();
    if prefs.pinned_locations.is_empty() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        prefs.pinned_locations = default_pinned_locations(&home);
    }
    if !prefs.pinned_locations.iter().any(|p| p.location == omnibox) {
        prefs.pinned_locations.push(PinnedLocation {
            id: format!("pin-{}", prefs.pinned_locations.len()),
            label,
            location: omnibox,
        });
        if prefs.pinned_locations.len() > 12 {
            prefs.pinned_locations.remove(1);
        }
        save_preferences(&prefs)?;
    }
    Ok(pinned_locations_dto(&prefs))
}

#[tauri::command]
pub fn set_restore_session_on_launch(enabled: bool) -> Result<PreferencesDto, String> {
    crate::session::set_restore_session_enabled(enabled)?;
    Ok(preferences_dto())
}

fn reduce_motion_to_str(pref: &crate::preferences::ReduceMotionPref) -> String {
    match pref {
        crate::preferences::ReduceMotionPref::System => "system".into(),
        crate::preferences::ReduceMotionPref::On => "on".into(),
        crate::preferences::ReduceMotionPref::Off => "off".into(),
    }
}

fn parse_reduce_motion(s: &str) -> Result<crate::preferences::ReduceMotionPref, String> {
    match s.to_ascii_lowercase().as_str() {
        "system" => Ok(crate::preferences::ReduceMotionPref::System),
        "on" => Ok(crate::preferences::ReduceMotionPref::On),
        "off" => Ok(crate::preferences::ReduceMotionPref::Off),
        _ => Err("reduce_motion must be system, on, or off".into()),
    }
}

fn theme_to_str(theme: &UiTheme) -> String {
    match theme {
        UiTheme::Light => "light".into(),
        UiTheme::Dark => "dark".into(),
        UiTheme::System => "system".into(),
    }
}

fn parse_ui_theme(theme: &str) -> Result<UiTheme, String> {
    match theme.to_ascii_lowercase().as_str() {
        "light" => Ok(UiTheme::Light),
        "dark" => Ok(UiTheme::Dark),
        "system" => Ok(UiTheme::System),
        _ => Err("theme must be light, dark, or system".into()),
    }
}

#[tauri::command]
pub fn set_ui_theme(theme: String) -> Result<PreferencesDto, String> {
    let parsed = parse_ui_theme(&theme)?;
    let mut p = load_preferences();
    p.theme = parsed;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn get_epub_chapter(window: tauri::Window, path: String, index: usize) -> Result<EpubChapterDto, String> {
    crate::epub_preview::preview_chapter(Path::new(&path), index)
}

#[tauri::command]
pub fn search_epub_in_book(
    path: String,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<EpubSearchHitDto>, String> {
    let cap = limit.unwrap_or(48);
    crate::epub_preview::search_in_book(Path::new(&path), &query, cap)
}

#[tauri::command]
pub async fn export_unibrow_project(window: tauri::Window, 
    app: AppHandle,
    project_path: String,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let project = PathBuf::from(&project_path);
    if !unibrowser_project::is_unibrow_path(&project) {
        return Err("not a .unibrow project".into());
    }
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(&project, &cache).map_err(|e| e.to_string())?;
    let pkg = unibrowser_project::open_package(&root).map_err(|e| e.to_string())?;
    let default_name = format!(
        "{}.unibrow",
        unibrowser_project::sanitize_label_for_filename(&pkg.manifest.title)
    );
    let picked = app
        .dialog()
        .file()
        .set_title("Export .unibrow package")
        .set_file_name(&default_name)
        .blocking_save_file();
    let dest = picked.ok_or("export cancelled")?;
    let dest_path = PathBuf::from(dest.to_string());
    unibrowser_project::export_package_to_zip(&root, &dest_path).map_err(|e| e.to_string())?;
    Ok(dest_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn export_unibrow_share_package(
    window: tauri::Window,
    app: AppHandle,
    project_path: String,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let project = PathBuf::from(&project_path);
    if !unibrowser_project::is_unibrow_path(&project) {
        return Err("not a .unibrow project".into());
    }
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(&project, &cache).map_err(|e| e.to_string())?;
    let pkg = unibrowser_project::open_package(&root).map_err(|e| e.to_string())?;
    let default_name = format!(
        "{}-share.zip",
        unibrowser_project::sanitize_label_for_filename(&pkg.manifest.title)
    );
    let picked = app
        .dialog()
        .file()
        .set_title("Export share package (no derived/)")
        .set_file_name(&default_name)
        .blocking_save_file();
    let dest = picked.ok_or("export cancelled")?;
    let dest_path = PathBuf::from(dest.to_string());
    unibrowser_project::export_share_package_to_zip(&root, &dest_path)
        .map_err(|e| e.to_string())?;
    Ok(dest_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn import_unibrow_share_package(
    window: tauri::Window,
    app: AppHandle,
    parent_dir: String,
    name: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Import share package")
        .add_filter("Share zip", &["zip", "unibrow"])
        .blocking_pick_file();
    let zip_path = PathBuf::from(picked.ok_or("import cancelled")?.to_string());
    let parent = PathBuf::from(parent_dir);
    let root = unibrowser_project::import_share_package_zip(&zip_path, &parent, &name)
        .map_err(|e| e.to_string())?;
    let loc = BrowserLocation::File { path: root.clone() };
    let doc = crate::state::open_file_document(&root);
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = false;
        tab.document = doc;
    }
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn autosave_unibrow_active_project(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<String, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    let path = match &tab.location {
        BrowserLocation::File { path } => path.clone(),
        _ => return Ok(String::new()),
    };
    if !unibrowser_project::is_unibrow_path(&path) {
        return Ok(String::new());
    }
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(&path, &cache).map_err(|e| e.to_string())?;
    let dest = unibrowser_project::snapshot_project_autosave(&root).map_err(|e| e.to_string())?;
    crate::project_recovery::maybe_backup_project(
        &path.to_string_lossy(),
        &root,
    );
    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn set_project_autosave_settings(
    _window: tauri::Window,
    enabled: bool,
    interval_secs: u32,
) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.project_autosave_enabled = enabled;
    p.project_autosave_interval_secs = interval_secs.clamp(30, 600);
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[derive(serde::Deserialize)]
pub struct AccessibilityPrefsInput {
    pub high_contrast: Option<bool>,
    pub reduce_motion: Option<String>,
}

#[tauri::command]
pub fn set_accessibility_prefs(input: AccessibilityPrefsInput) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    if let Some(hc) = input.high_contrast {
        p.high_contrast = hc;
    }
    if let Some(rm) = input.reduce_motion {
        p.reduce_motion = parse_reduce_motion(&rm)?;
    }
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_command_manager_dense(dense: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.command_manager_dense = dense;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn complete_onboarding_checklist_step(step_id: String) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    crate::preferences::complete_checklist_step(&mut p, &step_id);
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_onboarding_checklist_skip_forever(skip: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.onboarding_checklist_skip_forever = skip;
    if skip {
        p.onboarding_dismissed = true;
    }
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn dismiss_onboarding(window: tauri::Window) -> Result<(), String> {
    let mut p = load_preferences();
    p.onboarding_dismissed = true;
    save_preferences(&p)
}

#[tauri::command]
pub fn skip_onboarding_tour_forever(_window: tauri::Window) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.onboarding_tour_skipped_forever = true;
    p.onboarding_dismissed = true;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_ui_locale(locale: String) -> Result<PreferencesDto, String> {
    let locale = locale.trim();
    if locale != "en-NZ" && locale != "en-US" {
        return Err("unsupported locale (en-NZ or en-US)".into());
    }
    let mut p = load_preferences();
    p.ui_locale = locale.to_string();
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn save_window_geometry(window: tauri::Window, 
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
pub fn create_unibrow_project(window: tauri::Window, 
    parent_dir: String,
    name: String,
    template: Option<String>,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let parent = PathBuf::from(parent_dir);
    let root = parent.join(format!("{}.unibrow", name));
    let mut package = unibrowser_project::create_package(&root, &name).map_err(|e| e.to_string())?;
    let tpl = template.as_deref().unwrap_or("notes");
    unibrowser_project::apply_project_template(&mut package, tpl).map_err(|e| e.to_string())?;
    let loc = BrowserLocation::File { path: root.clone() };
    let doc = crate::state::open_file_document(&root);
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = false;
        tab.document = doc;
    }
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn search_open_vsx_catalog(window: tauri::Window, 
    query: String,
    network_opt_in: bool,
) -> Result<Vec<crate::open_vsx::OpenVsxHitDto>, String> {
    crate::open_vsx::search_open_vsx(&query, network_opt_in)
}

#[tauri::command]
pub fn open_mod_portal(window: tauri::Window, 
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
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = is_dir;
        tab.document = doc;
    }
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[derive(Serialize)]
pub struct InstalledVsixDto {
    pub extension_id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub license: Option<String>,
    pub command_count: usize,
    pub keybinding_count: usize,
    pub enabled: bool,
}

fn map_installed(rec: unibrowser_vsix::InstalledVsixRecord) -> InstalledVsixDto {
    InstalledVsixDto {
        extension_id: rec.extension_id,
        name: rec.name,
        version: rec.version,
        publisher: rec.publisher,
        license: rec.license,
        command_count: rec.contributions.commands.len(),
        keybinding_count: rec.contributions.keybindings.len(),
        enabled: rec.enabled,
    }
}

fn install_vsix_bytes_and_register(
    state: &AppState,
    bytes: &[u8],
    license: Option<&str>,
) -> Result<InstalledVsixDto, String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    let rec = unibrowser_vsix::install_vsix_bytes(&root, bytes, license)
        .map_err(|e| e.to_string())?;
    state.refresh_vsix_catalog();
    Ok(map_installed(rec))
}

#[tauri::command]
pub fn set_vsix_extension_enabled(window: tauri::Window, 
    extension_id: String,
    enabled: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    unibrowser_vsix::set_extension_enabled(&root, &extension_id, enabled)
        .map_err(|e| e.to_string())?;
    state.refresh_extension_catalog()?;
    Ok(())
}

#[derive(Serialize)]
pub struct InstalledNativeModDto {
    pub name: String,
    pub title: String,
    pub version: String,
    pub enabled: bool,
    pub library_present: bool,
    pub signature_ok: bool,
    pub load_block_reason: Option<String>,
}

#[tauri::command]
pub fn list_installed_native_mods(
    _window: tauri::Window,
) -> Result<Vec<InstalledNativeModDto>, String> {
    let root = crate::config::user_mods_dir().ok_or("config dir unavailable")?;
    let list = unibrowser_mod_host::native::list_installed(&root).map_err(|e| e.to_string())?;
    Ok(list
        .into_iter()
        .map(|rec| InstalledNativeModDto {
            name: rec.name,
            title: rec.title,
            version: rec.version,
            enabled: rec.enabled,
            library_present: rec.library_path.is_some(),
            signature_ok: rec.signature_ok,
            load_block_reason: rec.load_block_reason,
        })
        .collect())
}

#[tauri::command]
pub fn set_native_mod_enabled(
    _window: tauri::Window,
    name: String,
    enabled: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let root = crate::config::user_mods_dir().ok_or("config dir unavailable")?;
    unibrowser_mod_host::set_native_mod_enabled(&root, &name, enabled)
        .map_err(|e| e.to_string())?;
    state.refresh_extension_catalog()?;
    Ok(())
}

#[tauri::command]
pub fn uninstall_vsix_extension(window: tauri::Window, extension_id: String, state: State<AppState>) -> Result<(), String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    unibrowser_vsix::uninstall_extension(&root, &extension_id).map_err(|e| e.to_string())?;
    state.refresh_vsix_catalog();
    Ok(())
}

#[tauri::command]
pub fn list_installed_vsix(window: tauri::Window) -> Result<Vec<InstalledVsixDto>, String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    let list = unibrowser_vsix::list_installed(&root).map_err(|e| e.to_string())?;
    Ok(list.into_iter().map(map_installed).collect())
}

#[tauri::command]
pub fn install_vsix_from_path(window: tauri::Window, path: String, state: State<AppState>) -> Result<InstalledVsixDto, String> {
    let bytes = std::fs::read(PathBuf::from(path)).map_err(|e| e.to_string())?;
    install_vsix_bytes_and_register(state.inner(), &bytes, None)
}

#[tauri::command]
pub fn download_install_vsix(window: tauri::Window, 
    url: String,
    network_opt_in: bool,
    license: Option<String>,
    state: State<AppState>,
) -> Result<InstalledVsixDto, String> {
    let bytes = crate::open_vsx::download_vsix_bytes(&url, network_opt_in)?;
    install_vsix_bytes_and_register(state.inner(), &bytes, license.as_deref())
}

#[tauri::command]
pub async fn pick_and_install_vsix(window: tauri::Window, 
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<InstalledVsixDto, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter("VSIX", &["vsix"])
        .set_title("Install extension (.vsix)")
        .blocking_pick_file();
    let path = match picked {
        Some(p) => p.to_string(),
        None => return Err("cancelled".into()),
    };
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    install_vsix_bytes_and_register(state.inner(), &bytes, None)
}

#[tauri::command]
pub fn sync_svg_document(window: tauri::Window, content: String, state: State<AppState>) -> Result<(), String> {
    if content.len() > crate::document_limits::MAX_SYNC_TEXT_CHARS {
        return Err(crate::document_limits::sync_limit_error(content.len()));
    }
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let backend = base_svg::try_svg_backend(&mut *doc.backend).ok_or("not an SVG document")?;
    let before = backend.content.clone();
    let path = backend.path.clone();
    backend.set_content(content);
    let _sess_stacks = ws(&state, &window)?;
    let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
    stacks.record_change(&path, before, &backend.content);
    if backend.dirty {
        crate::crash_recovery::maybe_backup_dirty(&path, base_svg::BACKEND_ID, &backend.content);
    }
    Ok(())
}

#[tauri::command]
pub fn document_undo(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    {
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        let path = doc.path.clone();
        let current = if let Some(backend) = try_text_backend(&mut *doc.backend) {
            backend.content.clone()
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.content.clone()
        } else {
            return Err("undo not available for this document".into());
        };
        let _sess_stacks = ws(&state, &window)?;
        let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
        let prev = stacks.undo(&path, &current).ok_or("nothing to undo")?;
        if let Some(backend) = try_text_backend(&mut *doc.backend) {
            backend.set_content(prev);
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.set_content(prev);
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn document_redo(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    {
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        let path = doc.path.clone();
        let current = if let Some(backend) = try_text_backend(&mut *doc.backend) {
            backend.content.clone()
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.content.clone()
        } else {
            return Err("redo not available for this document".into());
        };
        let _sess_stacks = ws(&state, &window)?;
        let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
        let next = stacks.redo(&path, &current).ok_or("nothing to redo")?;
        if let Some(backend) = try_text_backend(&mut *doc.backend) {
            backend.set_content(next);
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.set_content(next);
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn toggle_split_view(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    let _sess_split = ws(&state, &window)?;
    let mut split = _sess_split.split_view.lock().map_err(|e| e.to_string())?;
    split.enabled = !split.enabled;
    if split.enabled && split.secondary_index.is_none() {
        let _sess_tabs = ws(&state, &window)?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let active = tabs.active_index();
        split.secondary_index = Some(if active > 0 { active - 1 } else { active });
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_split_tab(window: tauri::Window, index: usize, state: State<AppState>) -> Result<TabContentDto, String> {
    let _sess_split = ws(&state, &window)?;
    let mut split = _sess_split.split_view.lock().map_err(|e| e.to_string())?;
    split.enabled = true;
    split.secondary_index = Some(index);
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn omnibox_completions(window: tauri::Window, 
    query: String,
    state: State<AppState>,
) -> Result<Vec<OmniboxSuggestionDto>, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let cwd = listing_cwd(&state, &tabs);
    let mut out: Vec<OmniboxSuggestionDto> = Vec::new();
    let prefs = load_preferences();
    for entry in &prefs.recent_files {
        let score = fuzzy_score(&query, &entry.label)
            .max(fuzzy_score(&query, &entry.path));
        if score < 0 && !query.is_empty() {
            continue;
        }
        out.push(OmniboxSuggestionDto {
            label: entry.label.clone(),
            value: format!("file://{}", entry.path),
            kind: "recent".into(),
            score: score.max(0) + 50,
        });
    }
    if let Ok(entries) = list_dir(&cwd) {
        for entry in entries {
            let name = entry.name.clone();
            let path = entry.path.clone();
            let score = fuzzy_score(&query, &name);
            if score < 0 && !query.is_empty() {
                continue;
            }
            let value = format!("file://{}", path.display());
            out.push(OmniboxSuggestionDto {
                label: if entry.is_dir {
                    format!("[dir] {}", name)
                } else {
                    name
                },
                value,
                kind: if entry.is_dir { "dir" } else { "file" }.into(),
                score,
            });
        }
    }
    out.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.label.cmp(&b.label)));
    out.truncate(16);
    Ok(out)
}

#[tauri::command]
pub fn get_csv_preview(
    path: String,
    max_rows: Option<usize>,
    state: State<AppState>,
) -> Result<crate::csv_preview::CsvPreviewDto, String> {
    let limit = max_rows.unwrap_or(150).min(200);
    let p = resolve_open_path(&path, &state.cwd)?;
    crate::csv_preview::preview_file(&p, limit)
}

fn update_content_webview(
    app: &AppHandle,
    window_label: &str,
    loc: &BrowserLocation,
) -> Result<(), String> {
    sync_content_webview_for_window(app, window_label, loc)
}
