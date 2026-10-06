use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

use base_js::JsMonacoBackend;
use base_hex::{try_hex_backend, HexBackend};
use base_text::{TextBackend, try_text_backend};

use crate::config::toolbars_file_path;
use crate::downloads::{
    apply_download_disposition, copy_file_to_path, download_http_to_file,
    suggested_download_filename,
};
use crate::state::{AppState, list_dir};
use crate::mod_portal::{ModSuggestionDto, suggest_mods_for_unknown_path};
use crate::preferences::{
    default_pinned_locations, epub_read_chapter_index, epub_reading_notes, load_preferences,
    push_recent_file,
    export_epub_reading_notes_file, remember_epub_chapter, remember_epub_reading_notes,
    save_preferences, DownloadPreference, PinnedLocation, UiTheme,
    WindowState,
};
use crate::state::{FileCompareState, ImageCompareState};
use crate::epub_preview::{EpubChapterDto, EpubChapterListItem, EpubSearchHitDto};
use crate::user_backup::{self, GrandmaBackupV1};
use crate::path_security::{resolve_open_path, resolve_unibrow_asset_file};
use crate::webview::{
    adjust_page_font_for_url, adjust_page_zoom_for_url, inject_page_zoom_percent,
    reset_page_font_for_url, reset_page_zoom_for_url, sync_content_bounds,
    sync_content_webview_for_window,
};
/// Avoid main-thread deadlock: self-test runs `open_path` on the UI thread while JS `refreshState` would `invoke` back.
pub(crate) fn maybe_emit_browser_state_updated(app: &AppHandle) {
    if std::env::var("UNIBROWSER_SELF_TEST").as_deref() == Ok("1") {
        return;
    }
    let _ = app.emit("browser-state-updated", ());
}

use unibrowser_mod_api::handlers::{
    CONTEXT_HELP_ID, DOCUMENT_REDO_ID, DOCUMENT_UNDO_ID, EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID,
    FONT_DECREASE_ID, FONT_INCREASE_ID, FORMAT_DOCUMENT_ID, PAGE_ZOOM_DECREASE_ID,
    PAGE_ZOOM_FIT_ID, PAGE_ZOOM_INCREASE_ID,
    CAPTURE_VISIBLE_TAB_ID, COMPARE_IMAGES_ID, COMPARE_TEXT_ID, PICTURE_IN_PICTURE_ID,
    COPY_TEXT_FROM_IMAGE_ID, SHOW_KEYBOARD_SHORTCUTS_ID, TOGGLE_READER_MODE_ID,
    TOGGLE_TERMINAL_PANEL_ID, TOGGLE_WEB_DEVTOOLS_ID,
};
use unibrowser_engine::document::DocumentSession;
use unibrowser_engine::{
    host_context_for_browse, navigation::parse_user_input, save_command_manager_file,
    CommandManagerFile, ToolbarAnchor, ToolbarCoordSpace, ToolbarPhase, ToolbarPresentation,
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
    pub pinned: bool,
    /// Folder path when this tab can receive file drops (browse / directory tabs).
    pub browse_path: Option<String>,
    pub muted: bool,
    pub mute_available: bool,
}

#[derive(Serialize)]
pub struct TabSearchHitDto {
    pub window_label: String,
    pub tab_index: usize,
    pub label: String,
    pub omnibox: String,
    pub score: i32,
}

pub fn releases_page_url() -> String {
    std::env::var("UNIBROWSER_RELEASES_URL").unwrap_or_else(|_| {
        "https://github.com/unibrowser/unibrowser/releases".to_string()
    })
}

pub fn report_issue_page_url() -> String {
    std::env::var("UNIBROWSER_REPORT_ISSUE_URL").unwrap_or_else(|_| {
        "https://github.com/unibrowser/unibrowser/issues/new/choose".to_string()
    })
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
    pub families: Vec<unibrowser_mod_api::ResolvedFamily>,
    pub resolved_family_count: usize,
    pub resolved_control_count: usize,
}

#[derive(Serialize)]
pub struct FloatingBarDto {
    pub id: String,
    pub label: String,
    pub anchor: String,
    pub x: f64,
    pub y: f64,
    pub coord_space: String,
    pub presentation: String,
    pub commands: Vec<UiCommandDto>,
}

#[derive(Serialize)]
pub struct CommandManagerDto {
    pub auto_context: String,
    pub selected_tab: String,
    pub context_display_label: String,
    pub context_pinned: bool,
    pub tabs: Vec<ManagerTabDto>,
    pub floating_bars: Vec<FloatingBarDto>,
}

#[derive(Serialize)]
pub struct ExtensionTipDto {
    pub message: String,
    pub mod_id: String,
    pub mod_title: String,
}

#[derive(Serialize)]
pub struct PaletteEntryDto {
    pub command_id: String,
    pub title: String,
    pub chord: Option<String>,
    pub enabled: bool,
    pub reason: Option<String>,
    /// When set, the shell opens this path instead of `execute_command`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_path: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clip_markers: Vec<ClipMarkerDto>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub duck_under_video: bool,
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub fade_in_sec: f32,
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub fade_out_sec: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_text: Option<String>,
}

fn is_zero_f32(v: &f32) -> bool {
    *v <= 0.0 || !v.is_finite()
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize)]
pub struct ClipMarkerDto {
    pub id: String,
    pub offset_sec: f32,
    pub label: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_text: Option<String>,
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

#[derive(Clone, Serialize)]
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
    pub orphan_assets: Option<Vec<String>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_font_pt: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_zoom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_muted: Option<bool>,
}

#[derive(Serialize)]
pub struct TextDocumentDto {
    pub path: String,
    pub content: String,
    pub font_size_pt: f32,
    pub language: String,
    pub dirty: bool,
    pub encoding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reveal_line: Option<u32>,
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
    pub unknown_file_banner: Option<UnknownFileBannerDto>,
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
    pub image_compare_view: Option<ImageCompareViewDto>,
    pub pinned_locations: Vec<PinnedLocationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome_notice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_tip: Option<ExtensionTipDto>,
    pub close_undo_available: bool,
    pub recovery_bar: Vec<crate::crash_launch::RecoveryBarItemDto>,
    pub ntp_sections: NtpSectionsDto,
    pub files_view: FilesViewDto,
    pub folder_favorites: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    pub git_is_repo: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git_dirty_paths: Vec<String>,
    pub download_history: Vec<DownloadHistoryEntryDto>,
    pub bookmark_bar_enabled: bool,
    pub bookmark_quick_links: Vec<BookmarkQuickLinkDto>,
}

#[derive(Serialize, serde::Deserialize)]
pub struct BookmarkQuickLinkDto {
    pub label: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct KeyboardShortcutDto {
    pub chord: String,
    pub title: String,
    pub command_id: String,
}

#[derive(Serialize)]
pub struct DownloadHistoryEntryDto {
    pub id: String,
    pub url: String,
    pub label: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    pub at_ms: u64,
}

fn download_history_dto() -> Vec<DownloadHistoryEntryDto> {
    crate::download_history::list_entries()
        .into_iter()
        .map(|e| DownloadHistoryEntryDto {
            id: e.id,
            url: e.url,
            label: e.label,
            status: match e.status {
                crate::download_history::DownloadHistoryStatus::OpenedTab => "opened_tab".into(),
                crate::download_history::DownloadHistoryStatus::Saved => "saved".into(),
                crate::download_history::DownloadHistoryStatus::Failed => "failed".into(),
            },
            error: e.error,
            destination: e.destination,
            at_ms: e.at_ms,
        })
        .collect()
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
pub struct ImageCompareViewDto {
    pub left_path: String,
    pub right_path: String,
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

#[derive(Serialize)]
pub struct UnknownFileBannerDto {
    pub path: String,
    pub extension: String,
    pub suggestions: Vec<ModSuggestionDto>,
}


fn ws_state(state: &AppState, window: &tauri::Window) -> Result<crate::window_session::SharedWindowSession, String> {
    state.session(window.label())
}

fn ws(state: &State<AppState>, window: &tauri::Window) -> Result<crate::window_session::SharedWindowSession, String> {
    ws_state(state.inner(), window)
}

pub(crate) fn ws_label(
    state: &AppState,
    label: &str,
) -> Result<crate::window_session::SharedWindowSession, String> {
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
        pinned: tab.pinned,
        browse_path: tab_browse_path(state, tabs, tab),
        muted: tab.muted,
        mute_available: tab_mute_available(tab),
    }
}

fn tab_mute_available(tab: &unibrowser_engine::tabs::Tab) -> bool {
    match &tab.location {
        BrowserLocation::Http { .. } => true,
        BrowserLocation::File { path } => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            matches!(
                ext.as_str(),
                "mp4" | "webm" | "mkv" | "mov" | "wav" | "mp3" | "flac" | "ogg" | "m4a" | "aac"
                    | "opus"
            )
        }
        BrowserLocation::Internal { .. } => false,
    }
}

fn record_closed_tab(sess: &crate::window_session::WindowSession, loc: BrowserLocation) {
    if let Ok(mut stack) = sess.closed_tabs.lock() {
        stack.push(loc);
    }
}

fn collect_open_file_tabs(tabs: &unibrowser_engine::TabManager) -> Vec<OpenFileTabDto> {
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

fn open_file_tabs_dto(state: &AppState, window_label: &str) -> Vec<OpenFileTabDto> {
    let session = ws_label(state, window_label).ok();
    let Some(sess) = session else {
        return Vec::new();
    };
    let tabs = match sess.tabs.lock() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    collect_open_file_tabs(&tabs)
}

fn open_unibrow_package(path: &Path) -> Result<unibrowser_project::ProjectPackage, String> {
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(path, &cache).map_err(|e| e.to_string())?;
    unibrowser_project::open_package(&root).map_err(|e| e.to_string())
}

fn unibrow_package_root(path: &Path) -> Result<PathBuf, String> {
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    unibrowser_project::resolve_package_root(path, &cache).map_err(|e| e.to_string())
}

fn record_unibrow_timeline_undo(
    state: &AppState,
    window_label: &str,
    project_path_key: &str,
    package_root: &Path,
    before_json: String,
) -> Result<(), String> {
    let _sess = ws_label(state, window_label)?;
    let mut stacks = _sess.undo_stacks.lock().map_err(|e| e.to_string())?;
    crate::project_undo::record_manifest_undo(
        &mut *stacks,
        project_path_key,
        before_json,
        package_root,
    )
}

fn document_viewer_dto(
    doc: &mut unibrowser_engine::DocumentSession,
    state: &AppState,
    window_label: &str,
    tab_muted: bool,
    open_file_tabs: &[OpenFileTabDto],
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
                    let node = pkg.manifest.nodes.iter().find(|n| n.id == c.node_id);
                    let node_type = node
                        .map(|n| n.node_type.clone())
                        .unwrap_or_else(|| "asset".into());
                    let asset = node.and_then(|n| n.asset.clone());
                    ProjectClipDto {
                        id: c.id.clone(),
                        node_id: c.node_id.clone(),
                        node_type,
                        start_sec: c.start_sec,
                        duration_sec: c.duration_sec,
                        label: c.label.clone(),
                        color: c.color.clone(),
                        clip_markers: c
                            .clip_markers
                            .iter()
                            .map(|m| ClipMarkerDto {
                                id: m.id.clone(),
                                offset_sec: m.offset_sec,
                                label: m.label.clone(),
                            })
                            .collect(),
                        duck_under_video: c.duck_under_video,
                        fade_in_sec: c.fade_in_sec,
                        fade_out_sec: c.fade_out_sec,
                        asset,
                        pdf_page: c.pdf_page,
                        title_text: c.title_text.clone(),
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
        let orphan_assets = opened
            .as_ref()
            .and_then(|pkg| unibrowser_project::list_orphan_assets(pkg).ok());
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
            orphan_assets,
            open_file_tabs: Some(open_file_tabs.to_vec()),
            project_clips: clips,
            project_transitions: transitions,
            project_markers: markers,
            midi_summary: None,
            board_snap_to_grid: Some(prefs.board_snap_to_grid),
            midi_scrub_audio_enabled: Some(prefs.midi_scrub_audio_enabled),
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
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
            orphan_assets: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
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
            orphan_assets: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
        });
    }
    if doc.backend.backend_id() == crate::downloads_stub::BACKEND_ID {
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::downloads_stub::VIEWER_KIND.into(),
            title: "Downloads".into(),
            pdf_scale: None,
            pdf_page: None,
            media_note: None,
            project_nodes: None,
            orphan_assets: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
        });
    }
    if doc.backend.backend_id() == crate::diagnostics_stub::BACKEND_ID {
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::diagnostics_stub::VIEWER_KIND.into(),
            title: "Diagnostics".into(),
            pdf_scale: None,
            pdf_page: None,
            media_note: None,
            project_nodes: None,
            orphan_assets: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
        });
    }
    if doc.backend.backend_id() == crate::docs_stub::BACKEND_ID {
        let slug = crate::docs_stub::parse_slug_from_location(&doc.path)
            .unwrap_or_else(|| "getting-started".into());
        return Some(DocumentViewerDto {
            path: doc.path.clone(),
            viewer: crate::docs_stub::VIEWER_KIND.into(),
            title: format!("Help: {}", slug.replace('-', " ")),
            pdf_scale: None,
            pdf_page: None,
            media_note: Some(slug),
            project_nodes: None,
            orphan_assets: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
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
            orphan_assets: None,
            open_file_tabs: None,
            project_clips: None,
            project_transitions: None,
            project_markers: None,
            midi_summary: None,
            board_snap_to_grid: None,
            midi_scrub_audio_enabled: None,
            midi_note_preview: None,
            pdf_engines: None,
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
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
            orphan_assets: None,
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
            reader_font_pt: None,
            preview_zoom: None,
            tab_muted: None,
        });
    }
    let viewer = match doc.backend.backend_id() {
        base_docx::BACKEND_ID => base_docx::VIEWER_KIND,
        base_pdf::BACKEND_ID => base_pdf::VIEWER_KIND,
        base_dxf::BACKEND_ID => base_dxf::VIEWER_KIND,
        base_video::BACKEND_ID => base_video::VIEWER_KIND,
        base_audio::BACKEND_ID => base_audio::VIEWER_KIND,
        base_svg::BACKEND_ID => base_svg::VIEWER_KIND,
        base_png::BACKEND_ID => base_png::VIEWER_KIND,
        base_epub::BACKEND_ID => base_epub::VIEWER_KIND,
        base_csv::BACKEND_ID => base_csv::VIEWER_KIND,
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
    let reader_font_pt = reader_font_pt_for_doc(doc);
    Some(DocumentViewerDto {
        path: doc.path.clone(),
        viewer: viewer.into(),
        title: doc.backend.display_name().into(),
        pdf_scale,
        pdf_page,
        media_note,
        project_nodes: None,
        orphan_assets: None,
        open_file_tabs: None,
        project_clips: None,
        project_transitions: None,
        project_markers: None,
        midi_summary: None,
        board_snap_to_grid: None,
        midi_scrub_audio_enabled: None,
        midi_note_preview: None,
        pdf_engines,
        reader_font_pt,
        preview_zoom: preview_zoom_for_doc(doc),
        tab_muted: Some(tab_muted),
    })
}

fn preview_zoom_for_doc(doc: &mut unibrowser_engine::DocumentSession) -> Option<f32> {
    if let Some(b) = base_pdf::try_pdf_backend(&mut *doc.backend) {
        return Some(b.render_scale);
    }
    if let Some(b) = base_png::try_png_backend(&mut *doc.backend) {
        return Some(b.preview_zoom);
    }
    if let Some(b) = base_svg::try_svg_backend(&mut *doc.backend) {
        return Some(b.preview_zoom);
    }
    if let Some(b) = base_dxf::try_dxf_backend(&mut *doc.backend) {
        return Some(b.preview_zoom);
    }
    if let Some(b) = base_epub::try_epub_backend(&mut *doc.backend) {
        return Some(b.reader_scale);
    }
    None
}

fn reader_font_pt_for_doc(doc: &mut unibrowser_engine::DocumentSession) -> Option<f32> {
    if let Some(b) = base_docx::try_docx_backend(&mut *doc.backend) {
        return Some(b.reader_font_pt);
    }
    if let Some(b) = base_epub::try_epub_backend(&mut *doc.backend) {
        return Some(b.reader_font_pt);
    }
    if let Some(b) = base_svg::try_svg_backend(&mut *doc.backend) {
        return Some(b.font_size_pt);
    }
    if let Some(b) = base_dxf::try_dxf_backend(&mut *doc.backend) {
        return Some(b.effective_text_height());
    }
    if let Some(b) = base_png::try_png_backend(&mut *doc.backend) {
        if let Some(t) = &b.text_layer {
            return Some(t.font_size_pt);
        }
    }
    None
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
    let open_file_tabs = collect_open_file_tabs(tabs);
    let tab = match tabs.tab_mut(index) {
        Some(t) => t,
        None => return (None, None, None),
    };
    let status = tab
        .document
        .as_ref()
        .map(|d| d.backend.status_line());
    let tab_muted = tab.muted;
    let viewer = tab
        .document
        .as_mut()
        .and_then(|d| document_viewer_dto(d, state, window_label, tab_muted, &open_file_tabs));
    let reveal_line = tab.reveal_line;
    if reveal_line.is_some() {
        tab.reveal_line = None;
    }
    let text = if viewer.is_some() {
        None
    } else {
        tab.document
            .as_mut()
            .and_then(|d| text_document_dto(d, reveal_line))
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

fn text_document_dto(
    doc: &mut unibrowser_engine::DocumentSession,
    reveal_line: Option<u32>,
) -> Option<TextDocumentDto> {
    if let Some(backend) = try_hex_backend(&mut *doc.backend) {
        return Some(TextDocumentDto {
            path: backend.path.clone(),
            content: backend.content.clone(),
            font_size_pt: backend.font_size_pt,
            language: "hex".into(),
            dirty: false,
            encoding: "binary".into(),
            reveal_line,
        });
    }
    if let Some(backend) = try_text_backend(&mut *doc.backend) {
        return Some(TextDocumentDto {
            path: backend.path.clone(),
            content: backend.content.clone(),
            font_size_pt: backend.font_size_pt,
            language: TextBackend::language_id(&backend.path).into(),
            dirty: backend.dirty,
            encoding: detect_text_encoding(&backend.path),
            reveal_line,
        });
    }
    let backend = base_js::try_js_backend(&mut *doc.backend)?;
    Some(TextDocumentDto {
        path: backend.path.clone(),
        content: backend.content.clone(),
        font_size_pt: backend.font_size_pt,
        language: JsMonacoBackend::language_id(&backend.path).into(),
        dirty: backend.dirty,
        encoding: detect_text_encoding(&backend.path),
        reveal_line,
    })
}

fn tab_is_new_tab_home(tab: &unibrowser_engine::tabs::Tab) -> bool {
    matches!(
        &tab.location,
        BrowserLocation::Internal { page } if page == "home"
    )
}

fn snapshot(state: &AppState, window_label: &str) -> Result<TabContentDto, String> {
    crate::tabs_mutex::assert_tabs_lock_free();
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
            None
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
    let open_file_tabs = collect_open_file_tabs(&tabs);
    let (document_viewer, text_document) = match tabs.active_tab_mut() {
        Some(tab) => {
            let tab_muted = tab.muted;
            let reveal_line = tab.reveal_line;
            if reveal_line.is_some() {
                tab.reveal_line = None;
            }
            let viewer = tab
                .document
                .as_mut()
                .and_then(|d| document_viewer_dto(d, state, window_label, tab_muted, &open_file_tabs));
            let text = if viewer.is_some() {
                None
            } else {
                tab.document
                    .as_mut()
                    .and_then(|d| text_document_dto(d, reveal_line))
            };
            (viewer, text)
        }
        None => (None, None),
    };
    let mod_portal = active_mod_portal(tabs.active_tab());
    let unknown_file_banner = active_unknown_file_banner(tabs.active_tab());
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
    let image_compare_view = build_image_compare_view(state, window_label)?;
    let (text_document, document_viewer) = if diff_view.is_some() || image_compare_view.is_some() {
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
    let extension_tip = scripting_extension_tip(tabs.active_tab(), &prefs);
    let close_undo_available = sess
        .close_undo
        .lock()
        .map(|s| s.peek_available())
        .unwrap_or(false);
    let recovery_bar = sess
        .recovery_bar
        .lock()
        .map(|b| b.items.clone())
        .unwrap_or_default();
    let (git_is_repo, git_dirty_paths) =
        crate::git_status::status_for_folder(Path::new(&browse_folder_path));
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
        unknown_file_banner,
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
        image_compare_view,
        pinned_locations,
        chrome_notice,
        extension_tip,
        close_undo_available,
        recovery_bar,
        ntp_sections: ntp_sections_dto(&prefs),
        files_view: files_view_dto(&prefs),
        folder_favorites: prefs.folder_favorites.clone(),
        workspace_root: prefs.workspace_root.clone(),
        git_is_repo,
        git_dirty_paths: git_dirty_paths.into_iter().collect(),
        download_history: download_history_dto(),
        bookmark_bar_enabled: prefs.bookmark_bar_enabled,
        bookmark_quick_links: prefs
            .bookmark_quick_links
            .iter()
            .map(|l| BookmarkQuickLinkDto {
                label: l.label.clone(),
                url: l.url.clone(),
            })
            .collect(),
    })
}

fn build_image_compare_view(
    state: &AppState,
    window_label: &str,
) -> Result<Option<ImageCompareViewDto>, String> {
    let sess = ws_label(state, window_label)?;
    let guard = sess
        .image_compare
        .lock()
        .map_err(|e| e.to_string())?;
    let (left_path, right_path) = match guard.as_ref() {
        Some(c) => (c.left_path.clone(), c.right_path.clone()),
        None => return Ok(None),
    };
    let left = resolve_open_path(&left_path, &state.cwd)?;
    let right = resolve_open_path(&right_path, &state.cwd)?;
    if !crate::capture::is_raster_image_path(&left) || !crate::capture::is_raster_image_path(&right) {
        return Ok(None);
    }
    Ok(Some(ImageCompareViewDto {
        left_path,
        right_path,
    }))
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

fn scripting_extension_tip(
    tab: Option<&unibrowser_engine::tabs::Tab>,
    prefs: &crate::preferences::Preferences,
) -> Option<ExtensionTipDto> {
    if prefs.scripting_extension_tip_dismissed_forever {
        return None;
    }
    let tab = tab?;
    let doc = tab.document.as_ref()?;
    if doc.backend.backend_id() != base_text::BACKEND_ID {
        return None;
    }
    let path = match &tab.location {
        BrowserLocation::File { path } => path,
        _ => return None,
    };
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx") {
        return None;
    }
    Some(ExtensionTipDto {
        message: format!(
            "This file opened in the plain-text editor. For script tooling, try the bundled {} mod or install a VSIX extension from the mod portal.",
            "base-js"
        ),
        mod_id: "base-js".into(),
        mod_title: "Run user scripts safely".into(),
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
                message: "Install support for formats you need. Open VSX search + VSIX install (V1 command metadata; no extension host).".into(),
                suggestions,
                sideload_hint: "Install from file… (.vsix, .ubmod) — always allowed.".into(),
            });
        }
    }
    None
}

fn active_unknown_file_banner(
    tab: Option<&unibrowser_engine::tabs::Tab>,
) -> Option<UnknownFileBannerDto> {
    let tab = tab?;
    if let BrowserLocation::File { path } = &tab.location {
        if tab.document.is_none() && path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let suggestions = suggest_mods_for_unknown_path(&path.to_string_lossy());
            return Some(UnknownFileBannerDto {
                path: path.to_string_lossy().into_owned(),
                extension: if ext.is_empty() {
                    "(no extension)".into()
                } else {
                    format!(".{}", ext)
                },
                suggestions,
            });
        }
    }
    None
}

#[tauri::command]
pub fn open_active_file_as_text(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    reopen_active_file_mode(&state, &window, crate::state::FileOpenMode::PlainText)
}

#[tauri::command]
pub fn open_active_file_as_hex(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    reopen_active_file_mode(&state, &window, crate::state::FileOpenMode::Hex)
}

fn reopen_active_file_mode(
    state: &AppState,
    window: &tauri::Window,
    mode: crate::state::FileOpenMode,
) -> Result<TabContentDto, String> {
    let _sess = ws_state(state, window)?;
    {
        let mut tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no tab")?;
        let path = match &tab.location {
            BrowserLocation::File { path } => path.clone(),
            _ => return Err("not a file tab".into()),
        };
        if !path.is_file() {
            return Err("not a file".into());
        }
        let doc = crate::state::open_file_document_with_mode(&path, mode)
            .ok_or("could not open file")?;
        tab.document = Some(doc);
        tab.is_directory = false;
    }
    snapshot(state, window.label())
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
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
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
    let loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.set_active(index);
        tabs
            .active_tab()
            .map(|t| t.location.clone())
            .ok_or("no tab")?
    };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn close_tab(window: tauri::Window, 
    index: usize,
    _force: bool,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    {
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        if index >= tabs.tab_count() {
            return Err("invalid tab index".into());
        }
        let tab = tabs.tabs().get(index).ok_or("invalid tab")?;
        let was_dirty = tab_is_dirty(tab);
        let closed_loc = tab.location.clone();
        let autosave = if let Some(tab) = tabs.tab_mut(index) {
            tab.document
                .as_mut()
                .and_then(|doc| flush_document_autosave(doc))
        } else {
            None
        };
        if let Some(tab) = tabs.tabs().get(index) {
            _sess_tabs.purge_tab_resources(tab);
        }
        if !tabs.close_tab(index) {
            return Err("cannot close last tab".into());
        }
        record_closed_tab(&_sess_tabs, closed_loc.clone());
        if was_dirty || autosave.is_some() {
            if let Ok(mut undo) = _sess_tabs.close_undo.lock() {
                undo.push(crate::close_undo::CloseUndoEntry {
                    location: closed_loc,
                    insert_index: index,
                    autosave,
                });
            }
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
pub fn undo_close_tab(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let entry = _sess_tabs
        .close_undo
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("nothing to undo")?;
    let (loc, is_dir, doc) = state.resolve_location(entry.location.clone());
    let loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let insert_at = entry.insert_index.min(tabs.tab_count());
        tabs.insert_tab(insert_at, loc.clone());
        if let Some(tab) = tabs.tab_mut(insert_at) {
            tab.is_directory = is_dir;
            tab.document = doc;
            tab.location = loc.clone();
            if let (Some(doc), Some(record)) = (tab.document.as_mut(), entry.autosave.as_ref()) {
                let _ = apply_autosave_record(doc, record);
            }
        }
        tabs.set_active(insert_at);
        loc
    };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn reorder_tabs(
    window: tauri::Window,
    from_index: usize,
    to_index: usize,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        if !tabs.move_tab(from_index, to_index) {
            return Err("invalid tab reorder".into());
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn toggle_tab_pin(
    window: tauri::Window,
    index: usize,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let pinned = tabs
            .tabs()
            .get(index)
            .map(|t| t.pinned)
            .ok_or("invalid tab")?;
        if !tabs.set_tab_pinned(index, !pinned) {
            return Err("invalid tab".into());
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn search_open_tabs(
    _window: tauri::Window,
    query: String,
    state: State<AppState>,
) -> Result<Vec<TabSearchHitDto>, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let mut hits: Vec<TabSearchHitDto> = Vec::new();
    for (window_label, sess) in sessions.iter() {
        let tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
        for (index, tab) in tabs.tabs().iter().enumerate() {
            let label = tab.location.display_label();
            let omnibox = location_to_omnibox(&tab.location);
            let score = fuzzy_score(&query, &label).max(fuzzy_score(&query, &omnibox));
            if score < 0 && !query.is_empty() {
                continue;
            }
            hits.push(TabSearchHitDto {
                window_label: window_label.clone(),
                tab_index: index,
                label,
                omnibox,
                score: score.max(0),
            });
        }
    }
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.window_label.cmp(&b.window_label))
            .then_with(|| a.label.cmp(&b.label))
    });
    Ok(hits)
}

#[tauri::command]
pub fn activate_tab_in_window(
    window_label: String,
    tab_index: usize,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    let sess = ws_label(&state, &window_label)?;
    let loc = {
        let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
        tabs.set_active(tab_index);
        tabs
            .active_tab()
            .map(|t| t.location.clone())
            .ok_or("no tab")?
    };
    update_content_webview(&app, &window_label, &loc)?;
    if let Some(w) = app.get_webview_window(&window_label) {
        let _ = w.set_focus();
    }
    maybe_emit_browser_state_updated(&app);
    Ok(())
}

#[tauri::command]
pub fn open_crash_logs_folder(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let dir = crate::crash_log::ensure_crash_logs_dir().ok_or("config dir unavailable")?;
    app.opener()
        .reveal_item_in_dir(dir)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_releases_page(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let url = releases_page_url();
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_report_issue_page(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let url = report_issue_page_url();
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())?;
    Ok(())
}

fn apply_session_to_window(
    state: &AppState,
    app: &AppHandle,
    window_label: &str,
    entry: &crate::session::WindowSessionEntry,
) -> Result<(), String> {
    let pending_recovery = crate::project_recovery::pending_recovery_path_set(&state.cwd);
    let mut locations = Vec::new();
    for input in &entry.tab_inputs {
        if crate::project_recovery::session_tab_deferred_for_recovery(
            input,
            &state.cwd,
            &pending_recovery,
        ) {
            continue;
        }
        let loc = parse_user_input(input, &state.cwd);
        let (loc, _, _) = state.resolve_location(loc);
        locations.push(loc);
    }
    if locations.is_empty() {
        return Ok(());
    }
    let active_index = entry
        .active_index
        .min(locations.len().saturating_sub(1));
    {
        let _sess_tabs = ws_label(state, window_label)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.load_locations(locations, active_index);
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
        let _sess_tabs = ws_label(state, window_label)?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs
            .active_tab()
            .map(|t| t.location.clone())
            .ok_or("no tab")?
    };
    update_content_webview(app, window_label, &loc)?;
    if let Some(geom) = &entry.geometry {
        if let Some(window) = app.get_window(window_label) {
            let _ = crate::preferences::apply_window_state(&window, geom);
        }
    }
    Ok(())
}

pub fn ensure_browser_window(app: &AppHandle, label: &str) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    if app.get_webview_window(label).is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("Unibrowser")
        .inner_size(1200.0, 800.0)
        .build()
        .map_err(|e| e.to_string())?;
    crate::webview::ensure_content_webview(app, label)?;
    Ok(())
}

pub fn restore_session_on_launch(state: &AppState, app: &AppHandle) -> Result<(), String> {
    let prefs = load_preferences();
    if !prefs.restore_session_on_launch {
        return Ok(());
    }
    let snap = crate::session::load_session_snapshot();
    let snap = match snap {
        Some(s) => s,
        None => return Ok(()),
    };
    let entries = snap.normalized_windows();
    if entries.is_empty() {
        return Ok(());
    }
    for (i, entry) in entries.iter().enumerate() {
        let label = if i == 0 {
            "main".to_string()
        } else {
            entry.label.clone()
        };
        if i > 0 {
            ensure_browser_window(app, &label)?;
        }
        apply_session_to_window(state, app, &label, entry)?;
        let _ = crate::crash_launch::apply_crash_backups_after_session(state, app, &label);
    }
    Ok(())
}

pub fn open_dropped_paths(
    app: &AppHandle,
    state: &AppState,
    window_label: &str,
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
        let loc = BrowserLocation::File { path: safe.clone() };
        let (loc, is_dir, doc) = if unibrowser_project::is_unibrow_path(&safe) {
            let doc = crate::state::try_open_unibrow_document(&safe)?;
            (loc, false, Some(doc))
        } else {
            state.resolve_location(loc)
        };
        {
            let _sess_tabs = ws_label(state, window_label)?;
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
        update_content_webview(app, window_label, &loc)?;
    }
    maybe_emit_browser_state_updated(app);
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
pub fn open_path(
    window: tauri::Window,
    path: String,
    line: Option<u32>,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let file_path = resolve_open_path(&path, &state.cwd)?;
    if unibrowser_project::is_unibrow_path(&file_path) {
        allow_asset_protocol_file(&app, &file_path);
        let doc = crate::state::try_open_unibrow_document(&file_path)?;
        let loc = BrowserLocation::File {
            path: file_path.clone(),
        };
        let _sess_tabs = ws(&state, &window)?;
        {
            let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
            tabs.navigate_active(loc.clone(), false, Some(doc));
        }
        touch_recent_file(&file_path);
        update_content_webview(&app, window.label(), &loc)?;
        return snapshot(&state, window.label());
    }
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
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let open_in_new_tab = tabs
            .active_tab()
            .map(tab_is_new_tab_home)
            .unwrap_or(false)
            && !is_dir;
        if open_in_new_tab {
            tabs.open_tab(loc.clone());
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
                if let Some(line) = line.filter(|l| *l > 0) {
                    tab.reveal_line = Some(line);
                }
            }
        } else {
            tabs.navigate_active(loc.clone(), is_dir, doc);
            if let Some(line) = line.filter(|l| *l > 0) {
                if let Some(tab) = tabs.active_tab_mut() {
                    tab.reveal_line = Some(line);
                }
            }
        }
    }
    if let BrowserLocation::File { path } = &loc {
        if path.is_file() {
            touch_recent_file(path);
        }
    }
    update_content_webview(&app, window.label(), &loc)?;
    let dto = snapshot(&state, window.label())?;
    maybe_emit_browser_state_updated(&app);
    Ok(dto)
}

#[tauri::command]
fn open_resolved_path_on_tab(
    tabs: &mut unibrowser_engine::TabManager,
    file_path: &std::path::Path,
    state: &AppState,
    new_tab: bool,
) -> Result<BrowserLocation, String> {
    let loc = BrowserLocation::File {
        path: file_path.to_path_buf(),
    };
    let (loc, is_dir, doc) = if unibrowser_project::is_unibrow_path(file_path) {
        let doc = crate::state::try_open_unibrow_document(file_path)?;
        (loc, false, Some(doc))
    } else {
        state.resolve_location(loc)
    };
    if new_tab {
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
    } else {
        tabs.navigate_active(loc.clone(), is_dir, doc);
    }
    Ok(loc)
}

#[tauri::command]
pub fn open_selected_paths(
    window: tauri::Window,
    paths: Vec<String>,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    if paths.is_empty() {
        return Err("no paths selected".into());
    }
    let mut resolved = Vec::new();
    for path in paths {
        let file_path = resolve_open_path(&path, &state.cwd)?;
        if file_path.is_file() || file_path.is_dir() {
            resolved.push(file_path);
        }
    }
    if resolved.is_empty() {
        return Err("no openable paths in selection".into());
    }
    for file_path in &resolved {
        allow_asset_protocol_file(&app, file_path);
    }
    let _sess_tabs = ws(&state, &window)?;
    let last_loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let mut last_loc = None;
        for (i, file_path) in resolved.iter().enumerate() {
            let loc = open_resolved_path_on_tab(&mut tabs, file_path, &state, i > 0)?;
            if let BrowserLocation::File { path } = &loc {
                if path.is_file() {
                    touch_recent_file(path);
                }
            }
            last_loc = Some(loc);
        }
        last_loc
    };
    if let Some(loc) = last_loc {
        update_content_webview(&app, window.label(), &loc)?;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn web_find_in_page(
    window: tauri::Window,
    query: String,
    forward: bool,
    app: AppHandle,
) -> Result<(), String> {
    crate::webview_find::find_in_content_webview(&app, window.label(), &query, forward)
}

#[tauri::command]
pub fn focus_content_dom_selector(
    window: tauri::Window,
    selector: String,
    app: AppHandle,
) -> Result<(), String> {
    crate::webview_find::focus_dom_in_content_webview(&app, window.label(), &selector)
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
    let (loc, is_dir, doc) = if unibrowser_project::is_unibrow_path(&file_path) {
        let doc = crate::state::try_open_unibrow_document(&file_path)?;
        (loc, false, Some(doc))
    } else {
        state.resolve_location(loc)
    };
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
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

#[tauri::command]
pub fn set_workspace_root(
    window: tauri::Window,
    path: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let resolved = resolve_open_path(&path, &state.cwd)?;
    if !resolved.is_dir() {
        return Err("workspace root must be a folder".into());
    }
    let mut prefs = load_preferences();
    prefs.workspace_root = Some(resolved.to_string_lossy().into_owned());
    save_preferences(&prefs)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn clear_workspace_root(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    let mut prefs = load_preferences();
    prefs.workspace_root = None;
    save_preferences(&prefs)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn list_workspace_tree(
    max_entries: Option<usize>,
    state: State<AppState>,
) -> Result<Vec<crate::workspace_tree::WorkspaceTreeEntryDto>, String> {
    let prefs = load_preferences();
    let root = prefs
        .workspace_root
        .as_deref()
        .ok_or("no workspace root set")?;
    let resolved = resolve_open_path(root, &state.cwd)?;
    crate::workspace_tree::list_workspace_tree(
        &resolved,
        max_entries.unwrap_or(crate::workspace_tree::DEFAULT_MAX_ENTRIES),
        crate::workspace_tree::DEFAULT_MAX_DEPTH,
    )
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
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.navigate_active(loc.clone(), is_dir, doc);
    }
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
    let nav_loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        if let Some(loc) = tabs.go_back() {
            let (loc, is_dir, doc) = state.resolve_location(loc);
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
            }
            Some(loc)
        } else {
            None
        }
    };
    if let Some(loc) = nav_loc {
        update_content_webview(&app, window.label(), &loc)?;
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn go_forward(window: tauri::Window, app: AppHandle, state: State<AppState>) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let nav_loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        if let Some(loc) = tabs.go_forward() {
            let (loc, is_dir, doc) = state.resolve_location(loc);
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
            }
            Some(loc)
        } else {
            None
        }
    };
    if let Some(loc) = nav_loc {
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
        {
            let _sess_tabs = ws(&state, &window)?;
            let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
            }
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
pub fn clear_all_recent_files(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let mut prefs = load_preferences();
    crate::preferences::clear_all_recent(&mut prefs);
    save_preferences(&prefs)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn reset_onboarding_prefs_cmd() -> Result<PreferencesDto, String> {
    let mut prefs = load_preferences();
    crate::preferences::reset_onboarding_prefs(&mut prefs);
    save_preferences(&prefs)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn reset_floating_toolbars(
    app: tauri::AppHandle,
    window: tauri::Window,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    crate::toolbar_windows::close_all_toolbar_windows(&app, state.inner());
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let mut file = cm.to_file();
    file.floating_bars.clear();
    apply_command_manager_layout(&file, &state)?;
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

fn patch_capture_visible_tab_enablement(
    command_id: &str,
    enabled: bool,
    reason: Option<String>,
    location: Option<&BrowserLocation>,
) -> (bool, Option<String>) {
    if command_id != CAPTURE_VISIBLE_TAB_ID {
        return (enabled, reason);
    }
    if !matches!(location, Some(BrowserLocation::Http { .. })) {
        return (enabled, reason);
    }
    let status = crate::webview_capture::web_capture_status();
    if status.native_available {
        return (enabled, reason);
    }
    (false, Some(status.detail))
}

fn patch_tab_shell_enablement(
    command_id: &str,
    enabled: bool,
    reason: Option<String>,
    tab_count: usize,
    active_index: usize,
    closed_count: usize,
    mute_available: bool,
) -> (bool, Option<String>) {
    use unibrowser_mod_api::commands::{
        CLOSE_OTHER_TABS, CLOSE_TABS_TO_RIGHT, DUPLICATE_TAB, REOPEN_CLOSED_TAB, TOGGLE_TAB_MUTE,
    };
    if command_id == REOPEN_CLOSED_TAB {
        if closed_count == 0 {
            return (false, Some("No recently closed tabs".into()));
        }
        return (true, None);
    }
    if command_id == TOGGLE_TAB_MUTE {
        if !mute_available {
            return (
                false,
                Some("Mute is only available on https and media tabs".into()),
            );
        }
        return (true, None);
    }
    if command_id == DUPLICATE_TAB {
        return (true, None);
    }
    if command_id == CLOSE_OTHER_TABS {
        if tab_count <= 1 {
            return (false, Some("Only one tab open".into()));
        }
        return (true, None);
    }
    if command_id == CLOSE_TABS_TO_RIGHT {
        if active_index + 1 >= tab_count {
            return (
                false,
                Some("No tabs to the right of the active tab".into()),
            );
        }
        return (true, None);
    }
    (enabled, reason)
}

fn patch_undo_redo_enablement(
    command_id: &str,
    enabled: bool,
    reason: Option<String>,
    can_undo: bool,
    can_redo: bool,
) -> (bool, Option<String>) {
    if command_id == DOCUMENT_UNDO_ID {
        if !can_undo {
            return (false, Some("Nothing to undo".into()));
        }
        return (true, None);
    }
    if command_id == DOCUMENT_REDO_ID {
        if !can_redo {
            return (false, Some("Nothing to redo".into()));
        }
        return (true, None);
    }
    (enabled, reason)
}

fn patch_web_find_enablement(
    command_id: &str,
    enabled: bool,
    reason: Option<String>,
    location: Option<&BrowserLocation>,
) -> (bool, Option<String>) {
    if command_id != FIND_IN_DOCUMENT_ID {
        return (enabled, reason);
    }
    if !matches!(location, Some(BrowserLocation::Http { .. })) {
        return (enabled, reason);
    }
    let status = crate::webview_find::web_find_status();
    if status.available {
        return (enabled, reason);
    }
    (false, Some(status.detail))
}

fn map_ui_commands(
    cmds: Vec<unibrowser_engine::ResolvedUiCommand>,
    location: Option<&BrowserLocation>,
    tab_count: usize,
    active_index: usize,
    closed_count: usize,
    mute_available: bool,
    can_undo: bool,
    can_redo: bool,
) -> Vec<UiCommandDto> {
    cmds.into_iter()
        .map(|c| {
            let (enabled, reason) = patch_capture_visible_tab_enablement(
                &c.command_id,
                c.enabled,
                c.reason,
                location,
            );
            let (enabled, reason) =
                patch_web_find_enablement(&c.command_id, enabled, reason, location);
            let (enabled, reason) = patch_tab_shell_enablement(
                &c.command_id,
                enabled,
                reason,
                tab_count,
                active_index,
                closed_count,
                mute_available,
            );
            let (enabled, reason) = patch_undo_redo_enablement(
                &c.command_id,
                enabled,
                reason,
                can_undo,
                can_redo,
            );
            UiCommandDto {
                command_id: c.command_id,
                title: c.title,
                icon: c.icon,
                enabled,
                reason,
            }
        })
        .collect()
}

#[tauri::command]
pub fn get_command_manager(window: tauri::Window, state: State<AppState>) -> Result<CommandManagerDto, String> {
    state.refresh_command_manager_context(window.label());
    let _sess_tabs_guard = ws(&state, &window)?;
    let tabs_guard = _sess_tabs_guard.tabs.lock().map_err(|e| e.to_string())?;
    let tab_count = tabs_guard.tab_count();
    let active_index = tabs_guard.active_index();
    let active = tabs_guard.active_tab();
    let host = active
        .and_then(|t| t.document.as_ref())
        .map(|d| d.host_context())
        .unwrap_or_else(host_context_for_browse);
    let doc_ref = active.and_then(|t| t.document.as_ref());
    let active_location = active.map(|t| &t.location);
    let closed_count = _sess_tabs_guard
        .closed_tabs
        .lock()
        .map_err(|e| e.to_string())?
        .len();
    let mute_available = active.map(tab_mute_available).unwrap_or(false);
    let doc_path = active
        .and_then(|t| t.document.as_ref())
        .map(|d| d.path.clone());
    let (can_undo, can_redo) = {
        let stacks = _sess_tabs_guard
            .undo_stacks
            .lock()
            .map_err(|e| e.to_string())?;
        match doc_path.as_deref() {
            Some(path) => (stacks.can_undo(path), stacks.can_redo(path)),
            None => (false, false),
        }
    };
    let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let selected = cm.selected_tab.clone();
    let auto_context = cm.auto_context.tab_id().to_string();
    let label_path_buf = doc_ref.map(|d| d.path.clone()).or_else(|| {
        active.and_then(|t| match &t.location {
            BrowserLocation::File { path } => Some(path.to_string_lossy().into_owned()),
            _ => None,
        })
    });
    let label_path_ref = label_path_buf.as_deref();
    let mut tab_dtos: Vec<ManagerTabDto> = cm
        .tabs
        .iter()
        .map(|tab| {
            let families =
                cm.resolve_families_for_tab(&registry, &host, doc_ref, &tab.id, label_path_ref);
            let resolved_family_count = families.len();
            let resolved_control_count = families
                .iter()
                .flat_map(|family| &family.groups)
                .map(|group| group.controls.len())
                .sum();
            let resolved_cmds = if families.is_empty() {
                cm.resolve_tab_commands(&registry, &host, doc_ref, &tab.id)
            } else {
                unibrowser_engine::CommandManagerState::flatten_resolved_controls(&families)
            };
            let commands = map_ui_commands(
                resolved_cmds,
                active_location,
                tab_count,
                active_index,
                closed_count,
                mute_available,
                can_undo,
                can_redo,
            );
            ManagerTabDto {
                id: tab.id.clone(),
                label: tab.label.clone(),
                phase: phase_name(tab.phase).into(),
                selected: tab.id == selected,
                commands,
                families,
                resolved_family_count,
                resolved_control_count,
            }
        })
        .collect();
    let window_label = window.label();
    let toolbar_only_id = crate::toolbar_windows::bar_id_from_toolbar_window(window_label);

    let floating_bars = cm
        .floating_bars
        .iter()
        .filter(|bar| match bar.presentation {
            ToolbarPresentation::DetachedWindow => toolbar_only_id == Some(bar.id.as_str()),
            ToolbarPresentation::InWindow => toolbar_only_id.is_none(),
        })
        .map(|bar| {
            let commands = map_ui_commands(
                cm.resolve_floating_bar(&registry, &host, doc_ref, &bar.id)
                    .unwrap_or_default(),
                active_location,
                tab_count,
                active_index,
                closed_count,
                mute_available,
                can_undo,
                can_redo,
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
                presentation: match bar.presentation {
                    ToolbarPresentation::InWindow => "in_window",
                    ToolbarPresentation::DetachedWindow => "detached_window",
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
                    .map(|d| {
                        let (enabled, reason) =
                            crate::vsix_host::vsix_command_palette_state(&vsix_root, &d.command_id);
                        UiCommandDto {
                            command_id: d.command_id,
                            title: d.title,
                            icon: "vsix".into(),
                            enabled,
                            reason,
                        }
                    })
                    .collect(),
                families: vec![],
                resolved_family_count: 0,
                resolved_control_count: 0,
            });
        }
    }
    let context_display_label = unibrowser_engine::command_context_display_label(
        cm.auto_context,
        host.mode,
        label_path_ref,
    )
    .to_string();
    Ok(CommandManagerDto {
        auto_context,
        selected_tab: selected,
        context_display_label,
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
    *cm = unibrowser_engine::CommandManagerState::with_layout_path(
        layout,
        cm.auto_context,
        Some(path.as_path()),
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
pub fn detach_floating_toolbar_window(
    app: tauri::AppHandle,
    window: tauri::Window,
    bar_id: String,
    state: State<AppState>,
) -> Result<CommandManagerDto, String> {
    let (x, y) = {
        let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
        let bar = cm
            .floating_bars
            .iter_mut()
            .find(|b| b.id == bar_id)
            .ok_or_else(|| format!("toolbar not found: {}", bar_id))?;
        bar.presentation = ToolbarPresentation::DetachedWindow;
        if bar.anchor != ToolbarAnchor::Floating {
            bar.anchor = ToolbarAnchor::Floating;
            bar.coord_space = ToolbarCoordSpace::Screen;
            if bar.x == 0.0 && bar.y == 0.0 {
                bar.x = 96.0;
                bar.y = 96.0;
            }
        }
        (bar.x, bar.y)
    };
    let file = {
        let cm = state.command_manager.lock().map_err(|e| e.to_string())?;
        cm.to_file()
    };
    apply_command_manager_layout(&file, state.inner())?;
    crate::toolbar_windows::ensure_toolbar_window(&app, &bar_id, x, y)?;
    get_command_manager(window, state)
}

#[tauri::command]
pub fn list_workspace_personas() -> Vec<crate::workspace_personas::WorkspacePersonaDto> {
    crate::workspace_personas::list_personas()
}

#[tauri::command]
pub fn apply_workspace_persona(
    window: tauri::Window,
    state: State<AppState>,
    persona_id: String,
) -> Result<TabContentDto, String> {
    crate::workspace_personas::apply_persona(state.inner(), &persona_id)?;
    snapshot(state.inner(), window.label())
}

#[tauri::command]
pub fn dock_floating_toolbar_window(
    app: tauri::AppHandle,
    bar_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    crate::toolbar_windows::set_bar_presentation(
        state.inner(),
        &bar_id,
        ToolbarPresentation::InWindow,
    )?;
    crate::toolbar_windows::close_toolbar_window(&app, &bar_id);
    Ok(())
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
    cm.select_tab_manual(&tab_id);
    Ok(())
}

#[tauri::command]
pub fn execute_command(
    window: tauri::Window,
    command_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    let result = execute_command_impl(window, command_id.clone(), app, state, false);
    if result.is_ok() {
        crate::command_history::record(&command_id);
    }
    result
}

enum CloseTabsMode {
    Others,
    ToRight,
}

fn reset_all_web_zoom_prefs(app: &AppHandle) -> Result<String, String> {
    let count = crate::web_zoom_prefs::reset_all_origin_zoom_prefs()?;
    for (label, _) in app.webview_windows() {
        let _ = inject_page_zoom_percent(app, &label, 100);
    }
    Ok(format!(
        "Cleared saved zoom for {} site(s); open web tabs set to 100%",
        count
    ))
}

fn reopen_closed_tab(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<AppState>,
) -> Result<String, String> {
    let _sess_tabs = ws(state, window)?;
    let loc = _sess_tabs
        .closed_tabs
        .lock()
        .map_err(|e| e.to_string())?
        .pop()
        .ok_or("No recently closed tabs")?;
    let (loc, is_dir, doc) = state.resolve_location(loc);
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
            tab.location = loc.clone();
        }
    }
    update_content_webview(app, window.label(), &loc)?;
    Ok("Tab reopened".into())
}

fn toggle_active_tab_muted(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<AppState>,
) -> Result<String, String> {
    let _sess_tabs = ws(state, window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    if !tab_mute_available(tab) {
        return Err("Mute is only available on https and media tabs".into());
    }
    tab.muted = !tab.muted;
    let muted = tab.muted;
    let is_http = matches!(tab.location, BrowserLocation::Http { .. });
    if is_http {
        crate::webview::apply_tab_mute(app, window.label(), muted)?;
    }
    Ok(if muted {
        "Tab muted".into()
    } else {
        "Tab unmuted".into()
    })
}

#[tauri::command]
pub fn toggle_tab_mute(
    window: tauri::Window,
    index: usize,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let (muted, loc, apply_mute) = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.tab_mut(index).ok_or("invalid tab")?;
        if !tab_mute_available(tab) {
            return Err("Mute is only available on https and media tabs".into());
        }
        tab.muted = !tab.muted;
        let muted = tab.muted;
        let loc = tab.location.clone();
        let apply_mute = index == tabs.active_index() && matches!(loc, BrowserLocation::Http { .. });
        (muted, loc, apply_mute)
    };
    if apply_mute {
        crate::webview::apply_tab_mute(&app, window.label(), muted)?;
    }
    snapshot(&state, window.label())
}

fn duplicate_active_tab(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<AppState>,
) -> Result<String, String> {
    let _sess_tabs = ws(state, window)?;
    let loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let index = tabs.active_index();
        let src = tabs.active_tab().ok_or("no tab")?;
        let loc = src.location.clone();
        let (loc, is_dir, doc) = state.resolve_location(loc);
        tabs.duplicate_tab(index, doc).ok_or("could not duplicate tab")?;
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.location = loc.clone();
        }
        loc
    };
    update_content_webview(app, window.label(), &loc)?;
    Ok("Tab duplicated".into())
}

fn close_tabs_relative(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<AppState>,
    mode: CloseTabsMode,
) -> Result<String, String> {
    let _sess_tabs = ws(state, window)?;
    let (closed, loc) = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let active = tabs.active_index();
        let candidates: Vec<usize> = match mode {
            CloseTabsMode::Others => (0..tabs.tab_count()).filter(|i| *i != active).collect(),
            CloseTabsMode::ToRight => ((active + 1)..tabs.tab_count()).collect(),
        };
        if candidates.is_empty() {
            return Err("no tabs to close".into());
        }
        let mut closed = 0usize;
        for index in candidates.into_iter().rev() {
            if tabs.tab_count() <= 1 {
                break;
            }
            let tab = tabs.tabs().get(index).ok_or("invalid tab")?;
            if tab.pinned {
                continue;
            }
            let closed_loc = tab.location.clone();
            let was_dirty = tab_is_dirty(tab);
            let autosave = if let Some(tab) = tabs.tab_mut(index) {
                tab.document
                    .as_mut()
                    .and_then(|doc| flush_document_autosave(doc))
            } else {
                None
            };
            if let Some(tab) = tabs.tabs().get(index) {
                _sess_tabs.purge_tab_resources(tab);
            }
            if tabs.close_tab(index) {
                record_closed_tab(&_sess_tabs, closed_loc.clone());
                if was_dirty || autosave.is_some() {
                    if let Ok(mut undo) = _sess_tabs.close_undo.lock() {
                        undo.push(crate::close_undo::CloseUndoEntry {
                            location: closed_loc,
                            insert_index: index,
                            autosave,
                        });
                    }
                }
                closed += 1;
            }
        }
        if closed == 0 {
            return Err("Could not close tabs (pinned only)".into());
        }
        let loc = tabs
            .active_tab()
            .map(|t| t.location.clone())
            .ok_or("no tab")?;
        (closed, loc)
    };
    update_content_webview(app, window.label(), &loc)?;
    Ok(format!("Closed {} tab(s)", closed))
}

pub(crate) fn execute_builtin_from_extension_host(
    window: tauri::Window,
    command_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    execute_command_impl(window, command_id, app, state, true)
}

fn execute_command_impl(
    window: tauri::Window,
    command_id: String,
    app: AppHandle,
    state: State<AppState>,
    skip_vsix_host: bool,
) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Ok("palette".into());
    }
    if command_id == CONTEXT_HELP_ID {
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab().ok_or("no tab")?;
        let slug = crate::context_help::slug_for_tab(tab);
        let page = format!("docs/{}", slug);
        let loc = BrowserLocation::Internal { page };
        let doc = DocumentSession::new(
            crate::docs_stub::location_for_slug(&slug),
            crate::docs_stub::open(slug.as_str()),
        );
        tabs.open_tab(loc);
        if let Some(tab) = tabs.active_tab_mut() {
            tab.document = Some(doc);
            tab.is_directory = false;
        }
        return Ok("context-help-opened".into());
    }
    if command_id == unibrowser_mod_api::commands::GO_TO_TAB {
        return Ok("tab-search".into());
    }
    if command_id == unibrowser_mod_api::commands::DUPLICATE_TAB {
        return duplicate_active_tab(&window, &app, &state);
    }
    if command_id == unibrowser_mod_api::commands::CLOSE_OTHER_TABS {
        return close_tabs_relative(&window, &app, &state, CloseTabsMode::Others);
    }
    if command_id == unibrowser_mod_api::commands::CLOSE_TABS_TO_RIGHT {
        return close_tabs_relative(&window, &app, &state, CloseTabsMode::ToRight);
    }
    if command_id == unibrowser_mod_api::commands::REOPEN_CLOSED_TAB {
        return reopen_closed_tab(&window, &app, &state);
    }
    if command_id == unibrowser_mod_api::commands::RESTORE_AUTOSAVE {
        return Ok("autosave-restore-prompt".into());
    }
    if command_id == unibrowser_mod_api::commands::TOGGLE_PROBLEMS_PANEL {
        return Ok("problems-panel-toggle".into());
    }
    if command_id == unibrowser_mod_api::commands::TOGGLE_TAB_MUTE {
        return toggle_active_tab_muted(&window, &app, &state);
    }
    if command_id == unibrowser_mod_api::commands::PAGE_ZOOM_RESET_ALL {
        return reset_all_web_zoom_prefs(&app);
    }
    if command_id == unibrowser_mod_api::commands::WORKSPACE_QUICK_OPEN {
        return Ok("workspace-quick-open".into());
    }
    if command_id == unibrowser_mod_api::commands::START_OPEN_FILE {
        return Ok("start-open-file".into());
    }
    if command_id == unibrowser_mod_api::commands::START_OPEN_FOLDER {
        return Ok("start-open-folder".into());
    }
    if command_id == unibrowser_mod_api::commands::START_NEW_PROJECT {
        return Ok("start-new-project".into());
    }
    if command_id == unibrowser_mod_api::commands::START_RECENT {
        return Ok("start-recent".into());
    }
    if command_id == unibrowser_mod_api::commands::PDF_TOOL_SELECT {
        return Ok("pdf-select".into());
    }
    if command_id == unibrowser_mod_api::commands::PDF_ANNOTATE_NOTE {
        return Ok("pdf-note".into());
    }
    if command_id == unibrowser_mod_api::commands::PDF_ANNOTATE_AREA {
        return Ok("pdf-area".into());
    }
    if command_id == unibrowser_mod_api::commands::PDF_PAGE_NAV {
        return Ok("pdf-page-nav".into());
    }
    if command_id == unibrowser_mod_api::commands::PDF_VIEW_ZOOM {
        return Ok("pdf-view-zoom".into());
    }
    if command_id == unibrowser_mod_api::commands::FIND_FILES_DEEP {
        return Ok("find-files-deep".into());
    }
    if command_id == PICTURE_IN_PICTURE_ID {
        return Ok("media-pip".into());
    }
    if command_id == unibrowser_mod_api::commands::NEW_WINDOW {
        let label = open_new_browser_window(window, app)?;
        return Ok(format!("window:{}", label));
    }
    if command_id == SHOW_KEYBOARD_SHORTCUTS_ID {
        return Ok("keys-overlay".into());
    }
    if command_id == CAPTURE_VISIBLE_TAB_ID {
        return Ok("capture-visible".into());
    }
    if command_id == COMPARE_IMAGES_ID {
        return Ok("image-compare-pick".into());
    }
    if command_id == COMPARE_TEXT_ID {
        return Ok("text-compare-pick".into());
    }
    if command_id == TOGGLE_READER_MODE_ID {
        let _sess_tabs = ws(&state, &window)?;
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no tab")?;
        let doc = tab.document.as_mut().ok_or("reader mode needs an https tab")?;
        let web = base_web::try_web_backend_mut(&mut *doc.backend)
            .ok_or("reader mode is only for https pages")?;
        web.reader_mode = !web.reader_mode;
        let loc = tab.location.clone();
        if let BrowserLocation::Http { url } = &loc {
            crate::web_reader_prefs::set_origin_reader_mode(url, web.reader_mode)?;
            crate::webview::apply_reader_mode(&app, window.label(), web.reader_mode)?;
        }
        return Ok(if web.reader_mode {
            "reader-on".into()
        } else {
            "reader-off".into()
        });
    }
    if command_id == TOGGLE_TERMINAL_PANEL_ID {
        return Ok("terminal-toggle".into());
    }
    if command_id == COPY_TEXT_FROM_IMAGE_ID {
        return ocr_active_tab(&state, &window);
    }
    if command_id == TOGGLE_WEB_DEVTOOLS_ID {
        let _sess_tabs = ws(&state, &window)?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab().ok_or("no tab")?;
        let is_web = tab
            .document
            .as_ref()
            .map(|d| d.backend.backend_id() == base_web::BACKEND_ID)
            .unwrap_or(false);
        if !is_web {
            return Err("Developer tools are only available on https tabs".into());
        }
        let opened = crate::webview::toggle_content_webview_devtools(&app, window.label())?;
        return Ok(if opened {
            "devtools-open".into()
        } else {
            "devtools-close".into()
        });
    }
    if command_id == "mod.hello" {
        return Ok("Hello from native mod (stub handler)".into());
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
    if command_id == DOCUMENT_UNDO_ID {
        document_undo(window, state)?;
        return Ok("Undone".into());
    }
    if command_id == DOCUMENT_REDO_ID {
        document_redo(window, state)?;
        return Ok("Redone".into());
    }
    if command_id == FONT_INCREASE_ID || command_id == FONT_DECREASE_ID {
        let _sess_tabs = ws(&state, &window)?;
        let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab().ok_or("no tab")?;
        if let Some(doc) = &tab.document {
            if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
                let path = doc.path.clone();
                let project = std::path::PathBuf::from(&path);
                let delta = if command_id == FONT_INCREASE_ID { 2.0 } else { -2.0 };
                let mut package = open_unibrow_package(&project)?;
                let pt = unibrowser_project::adjust_timeline_title_font_pt(&mut package, delta)
                    .map_err(|e| e.to_string())?;
                let label = window.label().to_string();
                reload_unibrow_project_tab(state.inner(), &label, &project, path)?;
                return Ok(format!("Title card font {}pt", pt));
            }
        }
    }
    if command_id == FORMAT_DOCUMENT_ID {
        return crate::vsix_host::format_active_document(window, state);
    }
    {
        let host_cmds = state
            .vsix_host_commands
            .lock()
            .map_err(|e| e.to_string())?;
        if !skip_vsix_host && host_cmds.contains(&command_id) {
            let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
            let (enabled, reason) =
                crate::vsix_host::vsix_command_palette_state(&root, &command_id);
            if !enabled {
                return Err(reason.unwrap_or_else(|| "VSIX command unavailable".into()));
            }
            let ctx = unibrowser_vsix::lookup_vsix_command_context(&root, &command_id)
                .ok_or_else(|| format!("VSIX command not found: {}", command_id))?;
            let extension_id_log = ctx.extension_id.clone();
            let main = unibrowser_vsix::read_extension_main(&ctx.install_dir).ok_or_else(|| {
                unibrowser_extension_host::VSIX_NODE_HOST_NO_MAIN.to_string()
            })?;
            let activated = unibrowser_extension_host::record_command_activation(
                &ctx.extension_id,
                &command_id,
                &ctx.activation_events,
            );
            let result = crate::vsix_host::run_vsix_node_command(
                window.clone(),
                app.clone(),
                state.clone(),
                ctx,
                main,
                command_id.clone(),
            )?;
            if let Some((rec, true)) = activated {
                crate::diagnostic_log::record_info(
                    format!("VSIX activated: {}", extension_id_log),
                    Some(format!("matched `{}`", rec.matched_event)),
                );
            }
            if let Some(toast) = &result.toast {
                let severity = result.toast_severity.as_deref().unwrap_or("info");
                let _ = app.emit(
                    "unibrowser-extension-toast",
                    serde_json::json!({ "message": toast, "severity": severity }),
                );
            }
            return Ok(result.toast.unwrap_or(result.message));
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
    if doc.backend.backend_id() == base_web::BACKEND_ID
        && (command_id == PAGE_ZOOM_INCREASE_ID || command_id == PAGE_ZOOM_DECREASE_ID)
    {
        let step = if command_id == PAGE_ZOOM_INCREASE_ID { 10 } else { -10 };
        let url = match &tab.location {
            BrowserLocation::Http { url } => url.clone(),
            _ => return Err("not a web tab".into()),
        };
        let pct = adjust_page_zoom_for_url(&app, &url, step)?;
        return Ok(format!("Page zoom {}%", pct));
    }
    if doc.backend.backend_id() == base_web::BACKEND_ID && command_id == PAGE_ZOOM_FIT_ID {
        let url = match &tab.location {
            BrowserLocation::Http { url } => url.clone(),
            _ => return Err("not a web tab".into()),
        };
        let pct = reset_page_zoom_for_url(&app, &url)?;
        return Ok(format!("Page zoom {}% (fit)", pct));
    }
    if command_id == FIND_IN_DOCUMENT_ID {
        if doc.backend.backend_id() == base_web::BACKEND_ID {
            let status = crate::webview_find::web_find_status();
            if !status.available {
                return Err(status.detail);
            }
            return Ok("web-find".into());
        }
        let en = state
            .registry
            .lock()
            .map_err(|e| e.to_string())?
            .enablement(&command_id, doc);
        if !en.is_enabled() {
            return Err(en.disabled_reason().unwrap_or("disabled").into());
        }
        return Ok("document-find".into());
    }
    if command_id == EXPORT_PRINT_PDF_ID {
        let en = state
            .registry
            .lock()
            .map_err(|e| e.to_string())?
            .enablement(&command_id, doc);
        if !en.is_enabled() {
            return Err(en.disabled_reason().unwrap_or("disabled").into());
        }
        return Ok("export-print-pdf".into());
    }
    let (result, relocate_path) = {
        let relocate_path = if command_id == unibrowser_mod_api::commands::TEXT_SAVE
            && crate::autosave::document_needs_save_picker(&doc.path)
        {
            write_active_text_autosave_slot(doc)?;
            let new_path = pick_text_save_path(&app, &doc.path)?;
            doc.path = new_path.clone();
            if let Some(tb) = base_text::try_text_backend(&mut *doc.backend) {
                tb.path = new_path.clone();
            } else if let Some(js) = base_js::try_js_backend(&mut *doc.backend) {
                js.path = new_path.clone();
            } else if let Some(svg) = base_svg::try_svg_backend(&mut *doc.backend) {
                svg.path = new_path.clone();
            }
            Some(new_path)
        } else {
            None
        };
        let result = state
            .registry
            .lock()
            .map_err(|e| e.to_string())?
            .execute(&command_id, doc)?;
        (result, relocate_path)
    };
    if let Some(new_path) = relocate_path {
        tab.location = BrowserLocation::File {
            path: PathBuf::from(new_path),
        };
    }
    Ok(result)
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
    if let Some(backend) = try_text_backend(&mut *doc.backend) {
        let before = backend.content.clone();
        let path = backend.path.clone();
        backend.set_content(content);
        backend.set_selection_range(selection_start, selection_end);
        let _sess_stacks = ws(&state, &window)?;
        let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
        stacks.record_change(&path, before, &backend.content);
        if backend.dirty {
            crate::crash_recovery::maybe_backup_dirty(
                &path,
                base_text::BACKEND_ID,
                &backend.content,
            );
        }
        return Ok(());
    }
    let backend = base_js::try_js_backend(&mut *doc.backend).ok_or("not a text document")?;
    let before = backend.content.clone();
    let path = backend.path.clone();
    backend.set_content(content);
    backend.set_selection_range(selection_start, selection_end);
    let _sess_stacks = ws(&state, &window)?;
    let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
    stacks.record_change(&path, before, &backend.content);
    if backend.dirty {
        crate::crash_recovery::maybe_backup_dirty(&path, base_js::BACKEND_ID, &backend.content);
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
pub fn get_keyboard_shortcuts(state: State<AppState>) -> Result<Vec<KeyboardShortcutDto>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let kb = state.keybindings.lock().map_err(|e| e.to_string())?;
    let mut rows: Vec<KeyboardShortcutDto> = kb
        .bindings_for_display()
        .into_iter()
        .map(|(command_id, chord)| {
            let title = registry
                .command_meta(&command_id)
                .map(|m| m.title.to_string())
                .unwrap_or_else(|| command_id.clone());
            KeyboardShortcutDto {
                chord,
                title,
                command_id,
            }
        })
        .collect();
    rows.sort_by(|a, b| a.chord.cmp(&b.chord).then_with(|| a.title.cmp(&b.title)));
    Ok(rows)
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
    use unibrowser_engine::palette::fuzzy_score;

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
    let closed_count = _sess_tabs
        .closed_tabs
        .lock()
        .map_err(|e| e.to_string())?
        .len();
    let mute_available = tab_mute_available(tab);
    let (can_undo, can_redo) = {
        let stacks = _sess_tabs.undo_stacks.lock().map_err(|e| e.to_string())?;
        match tab.document.as_ref().map(|d| d.path.as_str()) {
            Some(path) => (stacks.can_undo(path), stacks.can_redo(path)),
            None => (false, false),
        }
    };
    let history = crate::command_history::list();
    let mut scored = search_commands(&registry, &ctx, &query);
    for entry in &mut scored {
        entry.score =
            unibrowser_engine::palette::apply_history_boost(&entry.command_id, entry.score, &history);
    }
    scored.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
    let mut out: Vec<PaletteEntryDto> = scored
        .into_iter()
        .map(|e| {
            let (enabled, reason) = if host_cmds.contains(&e.command_id) {
                crate::config::vsix_extensions_dir()
                    .map(|root| crate::vsix_host::vsix_command_palette_state(&root, &e.command_id))
                    .unwrap_or((false, Some("VSIX directory unavailable".into())))
            } else {
                (
                    e.enablement.is_enabled(),
                    e.enablement.disabled_reason().map(String::from),
                )
            };
            let (enabled, reason) = patch_capture_visible_tab_enablement(
                &e.command_id,
                enabled,
                reason,
                Some(&tab.location),
            );
            let (enabled, reason) = patch_web_find_enablement(
                &e.command_id,
                enabled,
                reason,
                Some(&tab.location),
            );
            let (enabled, reason) = patch_tab_shell_enablement(
                &e.command_id,
                enabled,
                reason,
                tabs.tab_count(),
                tabs.active_index(),
                closed_count,
                mute_available,
            );
            let (enabled, reason) = patch_undo_redo_enablement(
                &e.command_id,
                enabled,
                reason,
                can_undo,
                can_redo,
            );
            let chord = keybindings
                .chord_for_command(&e.command_id)
                .map(|c| c.display());
            PaletteEntryDto {
                command_id: e.command_id,
                title: e.title,
                chord,
                enabled,
                reason,
                open_path: None,
            }
        })
        .collect();

    let prefs = load_preferences();
    let q = query.trim();
    if !q.is_empty() {
        for entry in &prefs.recent_files {
            let score = fuzzy_score(q, &entry.label).max(fuzzy_score(q, &entry.path));
            if score > 0 {
                out.push(PaletteEntryDto {
                    command_id: "unibrowser.places.open".into(),
                    title: format!("Open recent: {}", entry.label),
                    chord: None,
                    enabled: true,
                    reason: Some(entry.path.clone()),
                    open_path: Some(entry.path.clone()),
                });
            }
        }
        for path in &prefs.folder_favorites {
            let label = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let score = fuzzy_score(q, &label).max(fuzzy_score(q, path));
            if score > 0 {
                out.push(PaletteEntryDto {
                    command_id: "unibrowser.places.open".into(),
                    title: format!("Open folder: {}", label),
                    chord: None,
                    enabled: true,
                    reason: Some(path.clone()),
                    open_path: Some(path.clone()),
                });
            }
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| state.cwd.to_string_lossy().to_string());
        let pins = if prefs.pinned_locations.is_empty() {
            default_pinned_locations(&home)
        } else {
            prefs.pinned_locations.clone()
        };
        for pin in pins {
            let score = fuzzy_score(q, &pin.label).max(fuzzy_score(q, &pin.location));
            if score > 0 {
                out.push(PaletteEntryDto {
                    command_id: "unibrowser.places.open".into(),
                    title: format!("Place: {}", pin.label),
                    chord: None,
                    enabled: true,
                    reason: Some(pin.location.clone()),
                    open_path: Some(pin.location.clone()),
                });
            }
        }
    }

    let mut recent: Vec<PaletteEntryDto> = Vec::new();
    for id in history {
        if let Some(meta) = registry.command_meta(&id) {
            if !q.is_empty() && fuzzy_score(q, meta.title) <= 0 && fuzzy_score(q, &id) <= 0 {
                continue;
            }
            let (enabled, reason) = if host_cmds.contains(&id) {
                crate::config::vsix_extensions_dir()
                    .map(|root| crate::vsix_host::vsix_command_palette_state(&root, &id))
                    .unwrap_or((false, Some("VSIX directory unavailable".into())))
            } else if let Some(doc) = tab.document.as_ref() {
                let en = registry.enablement(&id, doc);
                (
                    en.is_enabled(),
                    en.disabled_reason().map(String::from),
                )
            } else {
                (true, None)
            };
            let chord = keybindings.chord_for_command(&id).map(|c| c.display());
            recent.push(PaletteEntryDto {
                command_id: id.clone(),
                title: format!("Recent: {}", meta.title),
                chord,
                enabled,
                reason,
                open_path: None,
            });
        }
    }
    let recent_ids: std::collections::HashSet<String> =
        recent.iter().map(|e| e.command_id.clone()).collect();
    out.retain(|e| !recent_ids.contains(&e.command_id));
    out = recent.into_iter().chain(out).collect();

    Ok(out)
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
    let trimmed = url.trim();
    if prefs.download_preference == DownloadPreference::OpenInNewTab
        && has_dest_hint
        && (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
    {
        let dest = hinted.expect("has_dest_hint");
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let written = download_http_to_file(trimmed, &dest)?;
        let resolved = resolve_open_path(&written.to_string_lossy(), &state.cwd)?;
        let loc = BrowserLocation::File { path: resolved };
        let (loc, is_dir, doc) = state.resolve_location(loc);
        let _sess_tabs = ws(&state, &window)?;
        {
            let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
            tabs.open_tab(loc.clone());
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = is_dir;
                tab.document = doc;
            }
        }
        update_content_webview(&app, window.label(), &loc)?;
        crate::download_history::record_success(
            trimmed,
            crate::download_history::DownloadHistoryStatus::OpenedTab,
            Some(written.to_string_lossy().as_ref()),
        );
        return snapshot(&state, window.label());
    }
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
            {
                let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
                tabs.open_tab(loc.clone());
                if let Some(tab) = tabs.active_tab_mut() {
                    tab.is_directory = is_dir;
                    tab.document = doc;
                }
            }
            update_content_webview(&app, window.label(), &loc)?;
            crate::download_history::record_success(
                &url,
                crate::download_history::DownloadHistoryStatus::OpenedTab,
                None,
            );
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
                let written = download_http_to_file(trimmed, &dest)?;
                crate::download_history::record_success(
                    &url,
                    crate::download_history::DownloadHistoryStatus::Saved,
                    Some(written.to_string_lossy().as_ref()),
                );
                *ws(&state, &window)?
                    .chrome_notice
                    .lock()
                    .map_err(|e| e.to_string())? = Some(format!("Saved to {}", written.display()));
                return snapshot(&state, window.label());
            } else {
                let source = if trimmed.starts_with("file://") {
                    resolve_open_path(trimmed.trim_start_matches("file://"), &state.cwd)?
                } else {
                    resolve_open_path(trimmed, &state.cwd)?
                };
                copy_file_to_path(&source, &dest)?;
            }
            crate::download_history::record_success(
                &url,
                crate::download_history::DownloadHistoryStatus::Saved,
                Some(dest.to_string_lossy().as_ref()),
            );
            *ws(&state, &window)?
                .chrome_notice
                .lock()
                .map_err(|e| e.to_string())? = Some(format!("Saved to {}", dest.display()));
            snapshot(&state, window.label())
        }
    }
}

#[tauri::command]
pub fn record_download_failure(url: String, error: String) {
    crate::download_history::record_failure(&url, &error);
}

#[tauri::command]
pub fn clear_download_history() -> Result<Vec<DownloadHistoryEntryDto>, String> {
    crate::download_history::clear_entries()?;
    Ok(download_history_dto())
}

#[tauri::command]
pub async fn retry_download(
    window: tauri::Window,
    url: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TabContentDto, String> {
    handle_download_url(window, url, None, app, state).await
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
    pub editor_theme_id: String,
    pub command_manager_dense: bool,
    pub cm_pin_tip_seen: bool,
    pub high_contrast: bool,
    pub reduce_motion: String,
    pub restore_session_on_launch: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval_secs: u32,
    pub auto_save_slot_count: u32,
    pub download_preference: String,
    pub board_snap_to_grid: bool,
    pub midi_scrub_audio_enabled: bool,
    pub fluidsynth_soundfont_path: Option<String>,
    pub project_autosave_enabled: bool,
    pub project_autosave_interval_secs: u32,
    pub crash_recovery_enabled: bool,
    pub ntp_sections: NtpSectionsDto,
    pub advanced_webview_devtools: bool,
    pub advanced_experimental_spellcheck: bool,
    pub proofing_squiggles_enabled: bool,
    pub proofing_grammar_enabled: bool,
    pub advanced_node_extension_host: bool,
    pub open_downloads_in_system_browser: bool,
    pub files_view: FilesViewDto,
    pub bookmark_bar_enabled: bool,
    pub bookmark_quick_links: Vec<BookmarkQuickLinkDto>,
    pub workspace_root: Option<String>,
    pub disable_user_mods: bool,
    pub safe_mode_env: bool,
    pub chrome_focus_mode: bool,
    pub larger_hit_targets: bool,
    pub web_proxy: WebProxyPrefsDto,
    pub warn_insecure_http: bool,
    pub offline_mode: bool,
    pub vsix_trusted_extensions: Vec<String>,
}

#[derive(Serialize)]
pub struct WebProxyPrefsDto {
    pub mode: String,
    pub host: String,
    pub port: u16,
}

#[tauri::command]
pub fn set_disable_user_mods(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.disable_user_mods = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_bookmark_bar_enabled(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.bookmark_bar_enabled = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_bookmark_quick_links(links: Vec<BookmarkQuickLinkDto>) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.bookmark_quick_links = links
        .into_iter()
        .filter(|l| !l.url.trim().is_empty())
        .map(|l| crate::preferences::BookmarkQuickLink {
            label: l.label.trim().to_string(),
            url: l.url.trim().to_string(),
        })
        .collect();
    save_preferences(&p)?;
    Ok(preferences_dto())
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
    unibrowser_fs::atomic_write(&p, &bytes).map_err(|e| e.to_string())
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
pub async fn pick_save_html_path(
    _window: tauri::Window,
    app: AppHandle,
    suggested_name: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Save HTML")
        .set_file_name(suggested_name)
        .add_filter("HTML", &["html", "htm"])
        .blocking_save_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn export_print_pdf(
    app: AppHandle,
    html_body: String,
    suggested_title: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    if html_body.len() > crate::document_limits::MAX_SYNC_TEXT_CHARS {
        return Err(crate::document_limits::sync_limit_error(html_body.len()));
    }
    let suggested = crate::pdf_export::suggest_pdf_path(&suggested_title, &state.cwd);
    let picked = app
        .dialog()
        .file()
        .set_title("Export PDF")
        .set_file_name(
            suggested
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "document.pdf".into()),
        )
        .add_filter("PDF", &["pdf"])
        .blocking_save_file();
    let Some(dest) = picked else {
        return Ok("Cancelled".into());
    };
    let dest_path = PathBuf::from(dest.to_string());
    crate::pdf_export::export_html_to_pdf(&html_body, &dest_path)?;
    Ok(format!("Saved PDF to {}", dest_path.display()))
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
pub fn open_new_browser_window(_window: tauri::Window, app: AppHandle) -> Result<String, String> {
    let mut n = app.webview_windows().len().max(1) + 1;
    let mut label = format!("browser-{}", n);
    while app.get_webview_window(&label).is_some() {
        n += 1;
        label = format!("browser-{}", n);
    }
    ensure_browser_window(&app, &label)?;
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.set_focus();
    }
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

#[derive(Serialize)]
pub struct HelpDocDto {
    pub slug: String,
    pub title: String,
    pub markdown: String,
}

#[tauri::command]
pub fn get_help_doc_markdown(slug: String) -> Result<HelpDocDto, String> {
    let (title, markdown) = crate::docs_content::read_markdown(&slug)?;
    Ok(HelpDocDto {
        slug,
        title,
        markdown,
    })
}

#[tauri::command]
pub fn get_help_doc_toc() -> Result<Vec<crate::docs_content::DocTocEntry>, String> {
    crate::docs_content::toc_from_index()
}

#[tauri::command]
pub fn list_pdf_highlights(path: String) -> Result<Vec<crate::pdf_annotations::PdfHighlightDto>, String> {
    let p = Path::new(&path);
    crate::pdf_annotations::list_highlights(p)
}

#[tauri::command]
pub fn add_pdf_highlight(
    path: String,
    page: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Result<Vec<crate::pdf_annotations::PdfHighlightDto>, String> {
    let p = Path::new(&path);
    crate::pdf_annotations::add_highlight(
        p,
        crate::pdf_annotations::PdfHighlightDto {
            page,
            x,
            y,
            w,
            h,
            color: "rgba(255, 230, 0, 0.35)".into(),
            kind: "highlight".into(),
            author: None,
            created_at_ms: None,
            label: None,
        },
    )
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
    let step = if delta_px >= 0 { 10 } else { -10 };
    adjust_page_zoom_for_url(&app, &url, step)?;
    Ok(())
}

const DEFAULT_TEXT_FONT_PT: f32 = 12.0;
const DEFAULT_READER_FONT_PT: f32 = 12.0;
const DEFAULT_SVG_FONT_PT: f32 = 14.0;
const DEFAULT_PDF_RENDER_SCALE: f32 = 1.25;
const DEFAULT_IMAGE_PREVIEW_ZOOM: f32 = 1.0;

#[tauri::command]
pub fn reset_active_page_zoom(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no tab")?;
        match &tab.location {
            BrowserLocation::Http { url } => {
                reset_page_zoom_for_url(&app, url)?;
            }
            BrowserLocation::File { .. } => {
                let doc = tab.document.as_mut().ok_or("no document")?;
                let backend_id = doc.backend.backend_id();
                if backend_id == base_pdf::BACKEND_ID {
                    let pdf = base_pdf::try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
                    pdf.set_render_scale(DEFAULT_PDF_RENDER_SCALE);
                } else if backend_id == base_png::BACKEND_ID {
                    let img = base_png::try_png_backend(&mut *doc.backend).ok_or("not an image")?;
                    img.preview_zoom = DEFAULT_IMAGE_PREVIEW_ZOOM;
                } else {
                    return Err("zoom reset is not available for this tab".into());
                }
            }
            _ => return Err("zoom reset is not available for this tab".into()),
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn reset_active_context_font(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no tab")?;
        match &tab.location {
            BrowserLocation::Http { url } => {
                reset_page_font_for_url(&app, url)?;
            }
            BrowserLocation::File { .. } => {
                let doc = tab.document.as_mut().ok_or("no document")?;
                let path = doc.path.clone();
                let backend_id = doc.backend.backend_id();
                if backend_id == base_text::BACKEND_ID || backend_id == base_js::BACKEND_ID {
                    if let Some(text) = try_text_backend(&mut *doc.backend) {
                        text.font_size_pt = DEFAULT_TEXT_FONT_PT;
                    } else if let Some(text) = base_js::try_js_backend(&mut *doc.backend) {
                        text.font_size_pt = DEFAULT_TEXT_FONT_PT;
                    } else {
                        return Err("not a text document".into());
                    }
                } else if backend_id == base_docx::BACKEND_ID {
                    let b = base_docx::try_docx_backend(&mut *doc.backend).ok_or("not DOCX")?;
                    b.reader_font_pt = DEFAULT_READER_FONT_PT;
                    b.font_dirty = true;
                    crate::preferences::remember_docx_reader_font(&path, b.reader_font_pt);
                } else if backend_id == base_epub::BACKEND_ID {
                    let b = base_epub::try_epub_backend(&mut *doc.backend).ok_or("not EPUB")?;
                    b.reader_font_pt = DEFAULT_READER_FONT_PT;
                } else if backend_id == base_svg::BACKEND_ID {
                    let b = base_svg::try_svg_backend(&mut *doc.backend).ok_or("not SVG")?;
                    b.font_size_pt = DEFAULT_SVG_FONT_PT;
                } else {
                    return Err("font reset is not available for this tab".into());
                }
            }
            _ => return Err("font reset is not available for this tab".into()),
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn remove_unibrow_orphan_asset(
    window: tauri::Window,
    project_path: String,
    asset_file: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::remove_orphan_asset(&mut package, &asset_file).map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
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
    let root = unibrow_package_root(&project)?;
    let before = crate::project_undo::read_manifest_json(&root)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::update_timeline_clip_timing(
        &mut package,
        &clip_id,
        start_sec,
        duration_sec,
    )
    .map_err(|e| e.to_string())?;
    record_unibrow_timeline_undo(
        state.inner(),
        window.label(),
        &project_path,
        &root,
        before,
    )?;
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

fn open_unibrow_package_for_export(project_path: &Path) -> Result<unibrowser_project::ProjectPackage, String> {
    if !unibrowser_project::is_unibrow_path(project_path) {
        return Err("not a .unibrow project".into());
    }
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root = unibrowser_project::resolve_package_root(project_path, &cache).map_err(|e| e.to_string())?;
    let pkg = unibrowser_project::open_package(&root).map_err(|e| e.to_string())?;
    Ok(pkg)
}

#[tauri::command]
pub async fn export_unibrow_mp4_h264(
    window: tauri::Window,
    project_path: String,
    preset: Option<String>,
) -> Result<Mp4PreviewExportDto, String> {
    use tauri::Emitter;
    let preset = preset.unwrap_or_else(|| "fast".to_string());
    let pkg = open_unibrow_package_for_export(Path::new(&project_path))?;
    let intent = unibrowser_project::build_timeline_encode_intent(&pkg);
    if unibrowser_project::video_segments(&intent).is_empty() {
        let path = unibrowser_project::write_mp4_export_stub(
            &pkg,
            "timeline has no visual clips (save your project anytime; add clips to export video)",
        )
        .map_err(|e| e.to_string())?;
        return Ok(Mp4PreviewExportDto {
            path: path.to_string_lossy().into_owned(),
            mode: "stub".into(),
            message: "Nothing to render yet. Your project is saved; add visual clips or use Save to keep editing.".into(),
        });
    }
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
    let intent_copy = intent.clone();
    let prefs = load_preferences();
    let encode_options = unibrowser_media::TimelineEncodeOptions {
        soundfont_path: prefs
            .fluidsynth_soundfont_path
            .as_ref()
            .map(std::path::PathBuf::from),
    };
    let encode_result = tauri::async_runtime::spawn_blocking(move || {
        unibrowser_media::encode_h264_timeline_intents(
            &intent_copy,
            &dest_encode,
            &preset_copy,
            &encode_options,
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
    let report = match encode_result {
        Err(err) => {
            let reason = err.to_string();
            let path = unibrowser_project::write_mp4_export_stub(&pkg, &reason)
                .map_err(|e| e.to_string())?;
            return Ok(Mp4PreviewExportDto {
                path: path.to_string_lossy().into_owned(),
                mode: "stub".into(),
                message: format!("H.264 export failed: {}", reason),
            });
        }
        Ok(r) => r,
    };
    let mut message = format!(
        "Exported H.264 ({}) to derived/timeline-export-h264.mp4.",
        preset
    );
    if report.honored_fade {
        message.push_str(" Fade transition applied (approx).");
    }
    if report.honored_clip_fade {
        message.push_str(" Clip fade in/out applied (approx).");
    }
    if report.honored_duck {
        message.push_str(" Duck-under-video mix applied (approx).");
    }
    if !report.ignored.is_empty() {
        let note = report.ignored.join(" · ");
        message.push_str(format!(" Notes: {}.", note).as_str());
    }
    Ok(Mp4PreviewExportDto {
        path: dest_display,
        mode: "encoded".into(),
        message,
    })
}

#[tauri::command]
pub fn probe_midi_audio_render() -> unibrowser_media::MidiAudioRenderProbe {
    unibrowser_media::probe_midi_audio_render()
}

#[tauri::command]
pub fn search_workspace_quick_files(
    query: String,
    max_results: Option<usize>,
) -> Result<Vec<crate::workspace_quick_search::WorkspaceQuickHitDto>, String> {
    let prefs = load_preferences();
    let root = prefs
        .workspace_root
        .as_ref()
        .ok_or("Set a workspace root in Files (Set workspace) first.")?;
    crate::workspace_quick_search::fuzzy_find_workspace_files(
        Path::new(root),
        &query,
        max_results.unwrap_or(40),
    )
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
fn timeline_clip_preview_at(
    project: &Path,
    package: &unibrowser_project::ProjectPackage,
    clip_id: &str,
    offset_sec: f64,
) -> Result<TimelineScrubPreviewDto, String> {
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
    let node_type = node.node_type.to_ascii_lowercase();
    if node_type == "title" || clip.title_text.is_some() {
        let text = clip
            .title_text
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| clip.label.clone());
        return Ok(TimelineScrubPreviewDto {
            kind: "title".into(),
            image_path: None,
            image_mime: None,
            midi_notes: None,
            label_text: Some(text),
        });
    }
    let asset_name = node
        .asset
        .as_ref()
        .ok_or("clip has no asset")?;
    let asset_path = resolve_unibrow_asset_file(project, asset_name)?;
    let ext = asset_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if node_type == "video" {
        let frame = unibrowser_media::video_frame_thumbnail(&asset_path, offset_sec)
            .map_err(|e| e.to_string())?;
        return Ok(TimelineScrubPreviewDto {
            kind: "video".into(),
            image_path: Some(frame.path),
            image_mime: Some(frame.mime),
            midi_notes: None,
            label_text: None,
        });
    }
    if node_type == "pdf" || ext == "pdf" {
        let page = clip.pdf_page.unwrap_or(1).max(1);
        let path = crate::pdf_preview::pdf_page_preview(&asset_path, page)?;
        return Ok(TimelineScrubPreviewDto {
            kind: "pdf".into(),
            image_path: Some(path.to_string_lossy().into_owned()),
            image_mime: Some("image/jpeg".into()),
            midi_notes: None,
            label_text: None,
        });
    }
    if ext == "svg" {
        return Ok(TimelineScrubPreviewDto {
            kind: "svg".into(),
            image_path: Some(asset_path.to_string_lossy().into_owned()),
            image_mime: Some("image/svg+xml".into()),
            midi_notes: None,
            label_text: None,
        });
    }
    if node_type == "midi" || ext == "mid" || ext == "midi" {
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
            label_text: None,
        });
    }
    if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" || ext == "gif" {
        return Ok(TimelineScrubPreviewDto {
            kind: "image".into(),
            image_path: Some(asset_path.to_string_lossy().into_owned()),
            image_mime: None,
            midi_notes: None,
            label_text: None,
        });
    }
    if node_type == "audio" {
        return Ok(TimelineScrubPreviewDto {
            kind: "audio".into(),
            image_path: None,
            image_mime: None,
            midi_notes: None,
            label_text: None,
        });
    }
    Err(format!("scrub preview not available for {}", node_type))
}

#[tauri::command]
pub fn timeline_clip_scrub_preview(
    _window: tauri::Window,
    state: State<AppState>,
    project_path: String,
    clip_id: String,
    offset_sec: f64,
) -> Result<TimelineScrubPreviewDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let package = open_unibrow_package(&project)?;
    timeline_clip_preview_at(&project, &package, &clip_id, offset_sec)
}

#[tauri::command]
pub fn timeline_clip_block_preview(
    _window: tauri::Window,
    state: State<AppState>,
    project_path: String,
    clip_id: String,
) -> Result<TimelineScrubPreviewDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let package = open_unibrow_package(&project)?;
    timeline_clip_preview_at(&project, &package, &clip_id, 0.0)
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
pub fn set_unibrow_timeline_clip_fade(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    fade_in_sec: f32,
    fade_out_sec: f32,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let root = unibrow_package_root(&project)?;
    let before = crate::project_undo::read_manifest_json(&root)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::set_timeline_clip_fade(
        &mut package,
        &clip_id,
        fade_in_sec,
        fade_out_sec,
    )
    .map_err(|e| e.to_string())?;
    record_unibrow_timeline_undo(
        state.inner(),
        window.label(),
        &project_path,
        &root,
        before,
    )?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn dismiss_scripting_extension_tip() -> Result<(), String> {
    let mut prefs = load_preferences();
    prefs.scripting_extension_tip_dismissed_forever = true;
    save_preferences(&prefs)?;
    Ok(())
}

#[tauri::command]
pub fn set_unibrow_timeline_clip_duck(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    duck_under_video: bool,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let root = unibrow_package_root(&project)?;
    let before = crate::project_undo::read_manifest_json(&root)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::set_timeline_clip_duck_under_video(
        &mut package,
        &clip_id,
        duck_under_video,
    )
    .map_err(|e| e.to_string())?;
    record_unibrow_timeline_undo(
        state.inner(),
        window.label(),
        &project_path,
        &root,
        before,
    )?;
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
pub fn add_unibrow_clip_marker(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    offset_sec: f32,
    label: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::add_clip_marker(&mut package, &clip_id, offset_sec, label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn remove_unibrow_clip_marker(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    marker_id: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::remove_clip_marker(&mut package, &clip_id, &marker_id)
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
pub fn rename_unibrow_timeline_clip(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    label: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::rename_timeline_clip_label(&mut package, &clip_id, &label)
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
pub fn get_home_dir(_window: tauri::Window) -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())
}

#[tauri::command]
pub fn get_downloads_dir(_window: tauri::Window) -> String {
    crate::paths::user_downloads_dir().to_string_lossy().into_owned()
}

#[derive(Serialize)]
pub struct BuildInfoDto {
    pub version: String,
    pub version_line: String,
    pub git_hash: Option<String>,
    pub bundled_mods_count: u32,
    pub registry_commands_count: u32,
}

#[tauri::command]
pub fn get_build_info(state: State<AppState>) -> BuildInfoDto {
    BuildInfoDto {
        version: unibrowser_build_info::VERSION.into(),
        version_line: unibrowser_build_info::version_line(),
        git_hash: unibrowser_build_info::GIT_HASH.map(|s| s.to_string()),
        bundled_mods_count: state.bundled_mod_count() as u32,
        registry_commands_count: state.registry_command_count() as u32,
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
        editor_theme_id: p.editor_theme_id.clone(),
        command_manager_dense: p.command_manager_dense,
        cm_pin_tip_seen: p.cm_pin_tip_seen,
        high_contrast: p.high_contrast,
        reduce_motion: reduce_motion_to_str(&p.reduce_motion),
        restore_session_on_launch: p.restore_session_on_launch,
        auto_save_enabled: p.auto_save_enabled,
        auto_save_interval_secs: p.auto_save_interval_secs,
        auto_save_slot_count: p.auto_save_slot_count,
        download_preference: download_pref_to_str(&p.download_preference),
        board_snap_to_grid: p.board_snap_to_grid,
        midi_scrub_audio_enabled: p.midi_scrub_audio_enabled,
        fluidsynth_soundfont_path: p.fluidsynth_soundfont_path.clone(),
        project_autosave_enabled: p.project_autosave_enabled,
        project_autosave_interval_secs: p.project_autosave_interval_secs,
        crash_recovery_enabled: p.crash_recovery_enabled,
        ntp_sections: ntp_sections_dto(&p),
        advanced_webview_devtools: p.advanced_webview_devtools,
        advanced_experimental_spellcheck: p.advanced_experimental_spellcheck,
        proofing_squiggles_enabled: p.proofing_squiggles_enabled,
        proofing_grammar_enabled: p.proofing_grammar_enabled,
        advanced_node_extension_host: p.advanced_node_extension_host,
        open_downloads_in_system_browser: p.open_downloads_in_system_browser,
        files_view: files_view_dto(&p),
        bookmark_bar_enabled: p.bookmark_bar_enabled,
        bookmark_quick_links: p
            .bookmark_quick_links
            .iter()
            .map(|l| BookmarkQuickLinkDto {
                label: l.label.clone(),
                url: l.url.clone(),
            })
            .collect(),
        workspace_root: p.workspace_root.clone(),
        disable_user_mods: p.disable_user_mods,
        safe_mode_env: std::env::var("UNIBROWSER_SAFE_MODE").as_deref() == Ok("1"),
        chrome_focus_mode: p.chrome_focus_mode,
        larger_hit_targets: p.larger_hit_targets,
        web_proxy: web_proxy_prefs_dto(&p.web_proxy),
        warn_insecure_http: p.warn_insecure_http,
        offline_mode: p.offline_mode,
        vsix_trusted_extensions: p.vsix_trusted_extensions.clone(),
    }
}

#[tauri::command]
pub fn set_offline_mode(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.offline_mode = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_warn_insecure_http(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.warn_insecure_http = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

fn web_proxy_prefs_dto(p: &crate::preferences::WebProxyPrefs) -> WebProxyPrefsDto {
    WebProxyPrefsDto {
        mode: web_proxy_mode_to_str(&p.mode),
        host: p.host.clone(),
        port: p.port,
    }
}

fn web_proxy_mode_to_str(mode: &crate::preferences::WebProxyMode) -> String {
    match mode {
        crate::preferences::WebProxyMode::System => "system",
        crate::preferences::WebProxyMode::Direct => "direct",
        crate::preferences::WebProxyMode::Manual => "manual",
    }
    .into()
}

fn web_proxy_mode_from_str(s: &str) -> crate::preferences::WebProxyMode {
    match s {
        "direct" => crate::preferences::WebProxyMode::Direct,
        "manual" => crate::preferences::WebProxyMode::Manual,
        _ => crate::preferences::WebProxyMode::System,
    }
}

#[tauri::command]
pub fn set_chrome_focus_mode(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.chrome_focus_mode = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_larger_hit_targets(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.larger_hit_targets = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
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
pub fn set_proofing_squiggles_enabled(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.proofing_squiggles_enabled = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_proofing_grammar_enabled(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.proofing_grammar_enabled = enabled;
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[tauri::command]
pub fn set_advanced_node_extension_host(enabled: bool) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.advanced_node_extension_host = enabled;
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

#[derive(Deserialize)]
pub struct WebProxyPrefsInput {
    pub mode: String,
    pub host: String,
    pub port: u16,
}

#[tauri::command]
pub fn set_web_proxy_prefs(input: WebProxyPrefsInput) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.web_proxy.mode = web_proxy_mode_from_str(input.mode.trim());
    p.web_proxy.host = input.host.trim().to_string();
    p.web_proxy.port = input.port.min(65535);
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
    let loc = BrowserLocation::File { path: file_path.clone() };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let existing = tabs.tabs().iter().position(|t| {
            match &t.location {
                BrowserLocation::File { path } => {
                    resolve_open_path(&path.to_string_lossy(), &state.cwd).ok().as_ref()
                        == Some(&file_path)
                }
                _ => false,
            }
        });
        if let Some(index) = existing {
            tabs.set_active(index);
            if let Some(tab) = tabs.tab_mut(index) {
                tab.location = loc.clone();
                tab.is_directory = is_dir;
                tab.document = doc;
            }
        } else {
            tabs.navigate_active(loc.clone(), is_dir, doc);
        }
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
        crate::autosave::apply_saved_content(doc, &record.backend_id, &record.content)?;
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
    let loc = BrowserLocation::File { path: file_path.clone() };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let existing = tabs.tabs().iter().position(|t| {
            match &t.location {
                BrowserLocation::File { path } => {
                    resolve_open_path(&path.to_string_lossy(), &state.cwd).ok().as_ref()
                        == Some(&file_path)
                }
                _ => false,
            }
        });
        if let Some(index) = existing {
            tabs.set_active(index);
            if let Some(tab) = tabs.tab_mut(index) {
                tab.location = loc.clone();
                tab.is_directory = is_dir;
                tab.document = doc;
            }
        } else {
            tabs.navigate_active(loc.clone(), is_dir, doc);
        }
        if let BrowserLocation::File { path } = &loc {
            if path.is_file() {
                touch_recent_file(path);
            }
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

#[tauri::command]
pub fn set_fluidsynth_soundfont_path(path: Option<String>) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.fluidsynth_soundfont_path = path
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
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
pub fn set_auto_save_settings(
    _window: tauri::Window,
    enabled: bool,
    interval_secs: u32,
    slot_count: Option<u32>,
) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.auto_save_enabled = enabled;
    p.auto_save_interval_secs = crate::preferences::normalize_auto_save_interval_secs(interval_secs);
    if let Some(n) = slot_count {
        p.auto_save_slot_count = n.clamp(1, 32);
    }
    save_preferences(&p)?;
    Ok(preferences_dto())
}

#[derive(Serialize)]
pub struct AutosaveTickDto {
    pub wrote_slot: bool,
    pub last_autosave_at_ms: u64,
}

#[tauri::command]
pub fn get_autosave_status() -> AutosaveTickDto {
    AutosaveTickDto {
        wrote_slot: false,
        last_autosave_at_ms: crate::autosave::last_autosave_at_ms(),
    }
}

#[tauri::command]
pub fn reload_active_tab_from_disk(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no active tab")?;
        let loc = tab.location.clone();
        let (loc, is_dir, doc) = state.resolve_location(loc);
        tab.location = loc.clone();
        tab.is_directory = is_dir;
        tab.document = doc;
        loc
    };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn dismiss_recovery_bar_item(
    window: tauri::Window,
    state: State<AppState>,
    backup_id: String,
) -> Result<TabContentDto, String> {
    crate::crash_recovery::dismiss_backup(&backup_id)?;
    let _sess_tabs = ws(&state, &window)?;
    if let Ok(mut bar) = _sess_tabs.recovery_bar.lock() {
        bar.items.retain(|i| i.backup_id != backup_id);
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn list_autosave_slots_for_active_tab(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<Vec<crate::autosave::AutosaveSlotSummary>, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    let path = tab
        .document
        .as_ref()
        .map(|d| d.path.clone())
        .ok_or("no document")?;
    crate::autosave::list_autosave_slots(&path)
}

#[tauri::command]
pub fn restore_autosave_from_slot(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
    slot_path: String,
) -> Result<TabContentDto, String> {
    let text = std::fs::read_to_string(&slot_path).map_err(|e| e.to_string())?;
    let record: crate::autosave::AutosaveSlotRecord =
        serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let _sess_tabs = ws(&state, &window)?;
    let loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no active tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        if doc.path != record.document_path {
            return Err("autosave slot is for a different document".into());
        }
        apply_autosave_record(doc, &record)?;
        tab.location.clone()
    };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn restore_latest_autosave_for_active_tab(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let loc = {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs.active_tab_mut().ok_or("no active tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        let path = doc.path.clone();
        let record = crate::autosave::read_latest_autosave_content(&path)?
            .ok_or("no autosave slots for this document")?;
        if record.backend_id != doc.backend.backend_id() {
            return Err("autosave backend does not match active tab".into());
        }
        if record.backend_id == base_text::BACKEND_ID {
            let backend = try_text_backend(&mut *doc.backend).ok_or("not a text document")?;
            backend.set_content(record.content);
        } else if record.backend_id == base_js::BACKEND_ID {
            let backend = base_js::try_js_backend(&mut *doc.backend).ok_or("not a script document")?;
            backend.set_content(record.content);
        } else if record.backend_id == base_svg::BACKEND_ID {
            let backend = base_svg::try_svg_backend(&mut *doc.backend).ok_or("not an SVG document")?;
            backend.set_content(record.content);
        } else {
            return Err("autosave restore supports text, script, and SVG only".into());
        }
        tab.location.clone()
    };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

fn flush_document_autosave(doc: &mut DocumentSession) -> Option<crate::autosave::AutosaveSlotRecord> {
    if !doc.backend.is_dirty() {
        return None;
    }
    let path = doc.path.clone();
    let backend_id = doc.backend.backend_id().to_string();
    let content = if backend_id == base_text::BACKEND_ID {
        base_text::try_text_backend(&mut *doc.backend).map(|b| b.content.clone())
    } else if backend_id == base_js::BACKEND_ID {
        base_js::try_js_backend(&mut *doc.backend).map(|b| b.content.clone())
    } else if backend_id == base_svg::BACKEND_ID {
        base_svg::try_svg_backend(&mut *doc.backend).map(|b| b.content.clone())
    } else if backend_id == crate::project_backend::BACKEND_ID {
        crate::project_backend::try_unibrow_backend(&mut *doc.backend).and_then(|ub| {
            serde_json::to_string(&ub.package.manifest).ok()
        })
    } else if backend_id == base_docx::BACKEND_ID {
        base_docx::try_docx_backend(&mut *doc.backend).map(|b| {
            serde_json::json!({
                "path": b.path,
                "reader_font_pt": b.reader_font_pt,
                "font_dirty": b.font_dirty,
            })
            .to_string()
        })
    } else if backend_id == base_png::BACKEND_ID {
        base_png::try_png_backend(&mut *doc.backend).map(|b| {
            serde_json::json!({
                "path": path,
                "text_layer": b.text_layer.as_ref().map(|t| t.label.clone()),
                "font_pt": b.text_layer.as_ref().map(|t| t.font_size_pt),
            })
            .to_string()
        })
    } else {
        None
    };
    if let Some(text) = content {
        if text.is_empty() {
            return None;
        }
        crate::crash_recovery::maybe_backup_dirty(&path, &backend_id, &text);
        if let Ok(slot_path) =
            crate::autosave::write_document_autosave_slot(&path, &backend_id, &text)
        {
            let saved = std::fs::read_to_string(&slot_path).ok();
            return saved.and_then(|s| serde_json::from_str(&s).ok());
        }
    }
    None
}

fn write_active_text_autosave_slot(doc: &mut DocumentSession) -> Result<(), String> {
    flush_document_autosave(doc);
    Ok(())
}

fn apply_autosave_record(doc: &mut DocumentSession, record: &crate::autosave::AutosaveSlotRecord) -> Result<(), String> {
    crate::autosave::apply_saved_content(doc, &record.backend_id, &record.content)
}

fn pick_text_save_path(app: &AppHandle, current: &str) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let suggested = if current.trim().is_empty() {
        "untitled.txt".to_string()
    } else {
        Path::new(current)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.txt".to_string())
    };
    let picked = app
        .dialog()
        .file()
        .set_title("Save")
        .set_file_name(suggested)
        .blocking_save_file();
    picked
        .map(|p| p.to_string())
        .ok_or_else(|| "save cancelled".into())
}

#[tauri::command]
pub fn auto_save_active_document(window: tauri::Window, state: State<AppState>) -> Result<AutosaveTickDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no active tab")?;
    if tab.document.is_none() {
        return Ok(AutosaveTickDto {
            wrote_slot: false,
            last_autosave_at_ms: crate::autosave::last_autosave_at_ms(),
        });
    }
    let doc = tab.document.as_mut().unwrap();
    if !doc.backend.is_dirty() {
        return Ok(AutosaveTickDto {
            wrote_slot: false,
            last_autosave_at_ms: crate::autosave::last_autosave_at_ms(),
        });
    }
    let path = doc.path.clone();
    let backend_id = doc.backend.backend_id().to_string();
    let content = if backend_id == base_text::BACKEND_ID {
        base_text::try_text_backend(&mut *doc.backend)
            .map(|b| b.content.clone())
    } else if backend_id == base_js::BACKEND_ID {
        base_js::try_js_backend(&mut *doc.backend).map(|b| b.content.clone())
    } else if backend_id == base_svg::BACKEND_ID {
        base_svg::try_svg_backend(&mut *doc.backend).map(|b| b.content.clone())
    } else {
        None
    };
    if let Some(text) = content {
        crate::autosave::write_document_autosave_slot(&path, &backend_id, &text)?;
        return Ok(AutosaveTickDto {
            wrote_slot: true,
            last_autosave_at_ms: crate::autosave::last_autosave_at_ms(),
        });
    }
    if backend_id == base_docx::BACKEND_ID {
        if let Some(b) = base_docx::try_docx_backend(&mut *doc.backend) {
            if b.font_dirty {
                crate::preferences::remember_docx_reader_font(&doc.path, b.reader_font_pt);
                b.font_dirty = false;
            }
        }
    }
    Ok(AutosaveTickDto {
        wrote_slot: false,
        last_autosave_at_ms: crate::autosave::last_autosave_at_ms(),
    })
}

fn is_text_compare_file(path: &Path) -> bool {
    if crate::capture::is_raster_image_path(path) {
        return false;
    }
    if std::fs::metadata(path)
        .map(|m| m.len() > 8 * 1024 * 1024)
        .unwrap_or(true)
    {
        return false;
    }
    match std::fs::read(path) {
        Ok(bytes) => !bytes.contains(&0),
        Err(_) => false,
    }
}

#[tauri::command]
pub fn begin_file_compare(window: tauri::Window, 
    other_path: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let left = active_document_path(&state, &window)?;
    let left_path = resolve_open_path(&left, &state.cwd)?;
    let right = resolve_open_path(&other_path, &state.cwd)?;
    if !right.is_file() {
        return Err("compare target is not a file".into());
    }
    if !is_text_compare_file(&left_path) || !is_text_compare_file(&right) {
        return Err("Text compare needs two readable text files (not images or binary)".into());
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

fn open_capture_path_in_new_tab(
    window: &tauri::Window,
    app: &AppHandle,
    state: &AppState,
    path: PathBuf,
    notice: String,
) -> Result<TabContentDto, String> {
    allow_asset_protocol_file(app, &path);
    let loc = BrowserLocation::File { path: path.clone() };
    let (loc, is_dir, doc) = state.resolve_location(loc);
    let _sess_tabs = ws_label(state, window.label())?;
    let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    tabs.open_tab(loc.clone());
    if let Some(tab) = tabs.active_tab_mut() {
        tab.is_directory = is_dir;
        tab.document = doc;
    }
    touch_recent_file(&path);
    update_content_webview(app, window.label(), &loc)?;
    {
        let sess = ws_label(state, window.label())?;
        let mut n = sess.chrome_notice.lock().map_err(|e| e.to_string())?;
        *n = Some(notice);
    }
    snapshot(state, window.label())
}

#[derive(Serialize)]
pub struct CaptureTabResultDto {
    pub saved_path: Option<String>,
    pub need_shell: bool,
}

#[tauri::command]
pub fn capture_visible_tab(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<CaptureTabResultDto, String> {
    let _sess_tabs = ws(&state, &window)?;
    let tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    if let BrowserLocation::Http { .. } = &tab.location {
        match crate::webview_capture::capture_content_webview_png(&app, window.label()) {
            Ok(bytes) => {
                let path = crate::capture::write_png_bytes(&bytes)?;
                let saved = path.to_string_lossy().into_owned();
                let notice = format!("Capture saved to {}", path.display());
                let _ = open_capture_path_in_new_tab(&window, &app, state.inner(), path, notice)?;
                return Ok(CaptureTabResultDto {
                    saved_path: Some(saved),
                    need_shell: false,
                });
            }
            Err(_) => {
                return Ok(CaptureTabResultDto {
                    saved_path: None,
                    need_shell: true,
                });
            }
        }
    }
    if let Some(doc) = &tab.document {
        let path = PathBuf::from(&doc.path);
        if crate::capture::is_raster_image_path(&path) {
            let bytes = crate::capture::encode_image_file_as_png(&path)?;
            let saved = crate::capture::write_png_bytes(&bytes)?;
            let notice = format!("Capture saved to {}", saved.display());
            let _ = open_capture_path_in_new_tab(
                &window,
                &app,
                state.inner(),
                saved.clone(),
                notice,
            )?;
            return Ok(CaptureTabResultDto {
                saved_path: Some(saved.to_string_lossy().into_owned()),
                need_shell: false,
            });
        }
    }
    Ok(CaptureTabResultDto {
        saved_path: None,
        need_shell: true,
    })
}

#[tauri::command]
pub fn save_tab_capture_png(
    window: tauri::Window,
    app: AppHandle,
    png_base64: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let bytes = crate::capture::decode_png_base64(&png_base64)?;
    let path = crate::capture::write_png_bytes(&bytes)?;
    let notice = format!("Capture saved to {}", path.display());
    open_capture_path_in_new_tab(&window, &app, state.inner(), path, notice)
}

#[tauri::command]
pub fn record_diagnostic_error(message: String, details: Option<String>) {
    crate::diagnostic_log::record_error(message, details);
}

/// Frontend shell bootstrap: window errors, unhandled rejections, optional subsystem failures.
#[tauri::command]
pub fn log_shell_ui_event(level: String, message: String, detail: Option<String>) {
    let tag = format!("[shell-ui:{}]", message.trim());
    let lvl = level.trim().to_ascii_lowercase();
    if message.trim() == "doc-capture-diagnostics" {
        if let Some(d) = &detail {
            eprintln!("[doc-capture] diagnostics {}", d);
        }
    }
    if lvl == "info" || lvl == "debug" {
        crate::diagnostic_log::record_info(tag, detail);
    } else {
        crate::diagnostic_log::record_error(tag, detail);
    }
}

#[derive(Serialize)]
pub struct VsixActivationDto {
    pub extension_id: String,
    pub matched_event: String,
    pub command_id: String,
    pub at_ms: u64,
}

#[derive(Serialize)]
pub struct ExtensionHostDiagnosticsDto {
    pub host_version: String,
    pub host_running: bool,
    pub host_message: String,
    pub node_pid: Option<u32>,
    pub node_alive: bool,
    pub node_restart_count: u32,
    pub node_last_error: Option<String>,
    pub activated_extensions: Vec<VsixActivationDto>,
}

#[derive(Serialize)]
pub struct DiagnosticsInfoDto {
    pub mods_loaded: String,
    pub mod_catalog_count: u32,
    pub ffmpeg: unibrowser_media::FfmpegProbe,
    pub webview_stack: String,
    pub web_capture_native: bool,
    pub web_capture_detail: String,
    pub web_download_hook_wired: bool,
    pub web_download_hook_detail: String,
    pub extension_host: ExtensionHostDiagnosticsDto,
    pub target_os: String,
    pub target_arch: String,
    pub recent_errors: Vec<crate::diagnostic_log::DiagnosticLogEntry>,
    pub user_mods_disabled: bool,
    pub safe_mode_env: bool,
    pub memory_rss_kb: Option<u64>,
    pub memory_detail: String,
    pub crash_logs_dir: Option<String>,
    pub wasm_proving_stub: bool,
    pub wasm_runtime_linked: bool,
}

#[tauri::command]
pub fn get_diagnostics_info(state: State<AppState>) -> DiagnosticsInfoDto {
    let webview_stack = if cfg!(target_os = "linux") {
        "WebKitGTK (WRY content webview)"
    } else if cfg!(target_os = "windows") {
        "WebView2 (WRY content webview)"
    } else if cfg!(target_os = "macos") {
        "WKWebView (WRY content webview)"
    } else {
        "WRY embedded webview"
    };
    let web_capture = crate::webview_capture::web_capture_status();
    let (download_detail, download_wired) = crate::webview_download::platform_download_hook_status();
    let host_status = unibrowser_extension_host::status();
    let node_runtime = unibrowser_extension_host::runtime_status();
    let activated = unibrowser_extension_host::activated_extensions()
        .into_iter()
        .map(|r| VsixActivationDto {
            extension_id: r.extension_id,
            matched_event: r.matched_event,
            command_id: r.command_id,
            at_ms: r.at_ms,
        })
        .collect();
    let process_mem = crate::process_status::process_memory();
    DiagnosticsInfoDto {
        mods_loaded: state.mods_loaded_line(),
        mod_catalog_count: state.bundled_mod_count() as u32,
        ffmpeg: unibrowser_media::probe_ffmpeg_install(),
        webview_stack: webview_stack.into(),
        web_capture_native: web_capture.native_available,
        web_capture_detail: web_capture.detail,
        web_download_hook_wired: download_wired,
        web_download_hook_detail: download_detail.into(),
        extension_host: ExtensionHostDiagnosticsDto {
            host_version: unibrowser_extension_host::HOST_VERSION.into(),
            host_running: host_status.running,
            host_message: host_status.message,
            node_pid: node_runtime.pid,
            node_alive: node_runtime.alive,
            node_restart_count: node_runtime.restart_count,
            node_last_error: node_runtime.last_error,
            activated_extensions: activated,
        },
        target_os: std::env::consts::OS.into(),
        target_arch: std::env::consts::ARCH.into(),
        recent_errors: crate::diagnostic_log::recent_entries(),
        user_mods_disabled: crate::preferences::user_mods_disabled(),
        safe_mode_env: std::env::var("UNIBROWSER_SAFE_MODE").as_deref() == Ok("1"),
        memory_rss_kb: process_mem.rss_kb,
        memory_detail: process_mem.detail,
        crash_logs_dir: crate::crash_log::crash_logs_dir()
            .map(|p| p.to_string_lossy().into_owned()),
        wasm_proving_stub: unibrowser_mod_host::wasm::proving_hello_wasm_available(),
        wasm_runtime_linked: unibrowser_mod_host::wasm::runtime_linked(),
    }
}

#[tauri::command]
pub fn begin_image_compare(
    window: tauri::Window,
    other_path: String,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let left = active_document_path(&state, &window)?;
    let left_path = resolve_open_path(&left, &state.cwd)?;
    if !crate::capture::is_raster_image_path(&left_path) {
        return Err("compare images needs a PNG or JPEG tab".into());
    }
    let right = resolve_open_path(&other_path, &state.cwd)?;
    if !right.is_file() || !crate::capture::is_raster_image_path(&right) {
        return Err("compare target must be a PNG or JPEG file".into());
    }
    {
        let sess = ws(&state, &window)?;
        {
            let mut cmp = sess.file_compare.lock().map_err(|e| e.to_string())?;
            *cmp = None;
        }
        {
            let mut img = sess.image_compare.lock().map_err(|e| e.to_string())?;
            *img = Some(ImageCompareState {
                left_path: left,
                right_path: right.to_string_lossy().into_owned(),
            });
        }
    }
    snapshot(&state, window.label())
}

#[tauri::command]
pub async fn pick_and_compare_images(
    window: tauri::Window,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TabContentDto, String> {
    use tauri_plugin_dialog::DialogExt;
    let left = active_document_path(&state, &window)?;
    let left_path = resolve_open_path(&left, &state.cwd)?;
    if !crate::capture::is_raster_image_path(&left_path) {
        return Err("compare images needs a PNG or JPEG tab".into());
    }
    let picked = app
        .dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
        .set_title("Compare with image")
        .blocking_pick_file();
    let path = picked.map(|p| p.to_string()).ok_or("no file chosen")?;
    begin_image_compare(window, path, state)
}

#[tauri::command]
pub fn close_image_compare(window: tauri::Window, state: State<AppState>) -> Result<TabContentDto, String> {
    {
        let _sess_cmp = ws(&state, &window)?;
        let mut cmp = _sess_cmp.image_compare.lock().map_err(|e| e.to_string())?;
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
pub fn list_editor_themes_cmd() -> Result<Vec<crate::editor_themes::EditorThemeSummaryDto>, String> {
    crate::editor_themes::list_editor_themes()
}

#[tauri::command]
pub fn get_editor_monaco_theme(
    theme_id: String,
) -> Result<Option<unibrowser_extension_host::MonacoThemeDto>, String> {
    crate::editor_themes::get_editor_monaco_theme(theme_id)
}

#[tauri::command]
pub fn list_monaco_grammars_cmd(
) -> Result<Vec<crate::editor_grammars::MonacoGrammarDtoOut>, String> {
    crate::editor_grammars::list_monaco_grammars()
}

#[derive(serde::Serialize)]
pub struct DocxSaveReportDto {
    pub warnings: Vec<String>,
    pub paragraphs_written: usize,
}

#[tauri::command]
pub fn save_docx_from_preview_html(
    path: String,
    html: String,
    state: State<AppState>,
) -> Result<DocxSaveReportDto, String> {
    let p = resolve_open_path(&path, &state.cwd)?;
    if !p.is_file() {
        return Err("not a file".into());
    }
    let report = unibrowser_docx::save_docx_from_html(&p, &html).map_err(|e| e.to_string())?;
    Ok(DocxSaveReportDto {
        warnings: report.warnings,
        paragraphs_written: report.paragraphs_written,
    })
}

#[tauri::command]
pub fn set_editor_theme(theme_id: String) -> Result<PreferencesDto, String> {
    let mut p = load_preferences();
    p.editor_theme_id = theme_id;
    save_preferences(&p)?;
    Ok(preferences_dto())
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
pub fn list_epub_chapters(path: String) -> Result<Vec<EpubChapterListItem>, String> {
    crate::epub_preview::list_chapters(Path::new(&path))
}

#[tauri::command]
pub fn get_epub_read_position(path: String) -> usize {
    epub_read_chapter_index(&path).unwrap_or(0)
}

#[tauri::command]
pub fn remember_epub_chapter_position(path: String, index: usize) -> Result<(), String> {
    remember_epub_chapter(&path, index)
}

#[tauri::command]
pub fn get_epub_reading_notes(path: String) -> String {
    epub_reading_notes(&path)
}

#[tauri::command]
pub fn set_epub_reading_notes(path: String, notes: String) -> Result<(), String> {
    remember_epub_reading_notes(&path, &notes)
}

#[tauri::command]
pub fn export_epub_reading_notes(path: String) -> Result<String, String> {
    export_epub_reading_notes_file(&path)
}

#[tauri::command]
pub async fn export_grandma_backup_file(
    app: AppHandle,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let backup = user_backup::build_backup()?;
    let json = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    let picked = app
        .dialog()
        .file()
        .set_title("Export settings backup")
        .add_filter("JSON", &["json"])
        .set_file_name("unibrowser-backup.json")
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
pub async fn import_grandma_backup_file(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Import settings backup")
        .add_filter("JSON", &["json"])
        .blocking_pick_file();
    let path = picked.ok_or_else(|| "import cancelled".to_string())?;
    let data = std::fs::read_to_string(path.to_string()).map_err(|e| e.to_string())?;
    let backup: GrandmaBackupV1 = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    user_backup::apply_backup(&backup)?;
    apply_command_manager_layout(&backup.toolbars, state.inner())?;
    if let Ok(mut kb) = state.keybindings.lock() {
        *kb = crate::state::load_merged_keybindings();
    }
    Ok(())
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
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = false;
            tab.document = doc;
        }
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
    p.project_autosave_interval_secs =
        crate::preferences::normalize_auto_save_interval_secs(interval_secs);
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
        p.restore_session_on_launch = false;
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
    p.restore_session_on_launch = false;
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
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = false;
            tab.document = doc;
        }
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
pub fn list_mod_catalog(query: String) -> Vec<crate::mod_portal::ModCatalogEntryDto> {
    crate::mod_portal::filter_mod_catalog(&query)
}

#[tauri::command]
pub fn paste_unibrow_still_clip(
    window: tauri::Window,
    project_path: String,
    source_path: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let source = resolve_open_path(&source_path, &state.cwd)?;
    if !source.is_file() {
        return Err("clipboard image path is not a file".into());
    }
    let mut package = open_unibrow_package(&project)?;
    let node_type = unibrowser_project::guess_asset_node_type(&source);
    let label = source
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "still".into());
    let node = unibrowser_project::add_asset_copy(&mut package, &source, node_type, &label)
        .map_err(|e| e.to_string())?;
    unibrowser_project::append_timeline_clip_for_node(&mut package, &node.id)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn add_unibrow_pdf_page_timeline_clip(
    window: tauri::Window,
    project_path: String,
    source_path: String,
    page: u32,
    label: Option<String>,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let source = resolve_open_path(&source_path, &state.cwd)?;
    let label = label.unwrap_or_else(|| {
        source
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "PDF page".into())
    });
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::add_pdf_page_timeline_clip(&mut package, &source, page, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn add_unibrow_svg_still_timeline_clip(
    window: tauri::Window,
    project_path: String,
    source_path: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let source = resolve_open_path(&source_path, &state.cwd)?;
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "svg" {
        return Err("SVG still clip needs a .svg file".into());
    }
    let label = source
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "SVG".into());
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::add_asset_timeline_clip(&mut package, &source, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn add_unibrow_title_timeline_clip(
    window: tauri::Window,
    project_path: String,
    title_text: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::add_title_card_timeline_clip(&mut package, &title_text)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn add_unibrow_midi_timeline_clip(
    window: tauri::Window,
    project_path: String,
    source_path: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let source = resolve_open_path(&source_path, &state.cwd)?;
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "mid" && ext != "midi" {
        return Err("MIDI track clip needs a .mid or .midi file".into());
    }
    let label = source
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "MIDI".into());
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::add_asset_timeline_clip(&mut package, &source, &label)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_unibrow_timeline_clip_pdf_page(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    page: u32,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::set_timeline_clip_pdf_page(&mut package, &clip_id, page)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn set_unibrow_timeline_clip_title_text(
    window: tauri::Window,
    project_path: String,
    clip_id: String,
    title_text: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<TabContentDto, String> {
    let project = resolve_open_path(&project_path, &state.cwd)?;
    let mut package = open_unibrow_package(&project)?;
    unibrowser_project::set_timeline_clip_title_text(&mut package, &clip_id, &title_text)
        .map_err(|e| e.to_string())?;
    reload_unibrow_project_tab(state.inner(), window.label(), &project, project_path)?;
    let loc = BrowserLocation::File { path: project };
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[tauri::command]
pub fn write_temp_png(bytes: Vec<u8>) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("empty image".into());
    }
    if bytes.len() > crate::document_limits::MAX_WRITE_BYTES {
        return Err(crate::document_limits::write_limit_error(bytes.len()));
    }
    let path = std::env::temp_dir().join(format!(
        "unibrowser-clip-{}.png",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn dismiss_cm_pin_tip() -> Result<PreferencesDto, String> {
    let mut prefs = load_preferences();
    prefs.cm_pin_tip_seen = true;
    save_preferences(&prefs)?;
    Ok(preferences_dto())
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
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = is_dir;
            tab.document = doc;
        }
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
    pub compatibility_level: String,
    pub compatibility_missing_apis: Vec<String>,
    pub theme_notes: Vec<String>,
    pub grammar_notes: Vec<String>,
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
        compatibility_level: if rec.compatibility_level.is_empty() {
            "works".into()
        } else {
            rec.compatibility_level
        },
        compatibility_missing_apis: rec.compatibility_missing_apis,
        theme_notes: rec.theme_notes,
        grammar_notes: rec.grammar_notes,
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

#[derive(serde::Serialize)]
pub struct ModToggleResultDto {
    /// When false, stub/metadata commands were refreshed in-process; no restart needed for palette changes.
    pub restart_recommended: bool,
    /// Node extension host: user must confirm trust inline before enable proceeds.
    #[serde(default)]
    pub needs_trust: bool,
}

#[tauri::command]
pub fn set_vsix_extension_enabled(
    _window: tauri::Window,
    extension_id: String,
    enabled: bool,
    _app: AppHandle,
    state: State<AppState>,
) -> Result<ModToggleResultDto, String> {
    if enabled && crate::vsix_host::node_extension_host_enabled() {
        let prefs = load_preferences();
        if !prefs.vsix_trusted_extensions.iter().any(|id| id == &extension_id) {
            return Ok(ModToggleResultDto {
                restart_recommended: false,
                needs_trust: true,
            });
        }
    }
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    unibrowser_vsix::set_extension_enabled(&root, &extension_id, enabled)
        .map_err(|e| e.to_string())?;
    if !enabled {
        let _ = unibrowser_extension_host::deactivate_extension(&extension_id);
    }
    state.refresh_extension_catalog()?;
    Ok(ModToggleResultDto {
        restart_recommended: false,
        needs_trust: false,
    })
}

#[tauri::command]
pub fn trust_vsix_extension(extension_id: String) -> Result<PreferencesDto, String> {
    let mut prefs = load_preferences();
    if !prefs
        .vsix_trusted_extensions
        .iter()
        .any(|id| id == &extension_id)
    {
        prefs.vsix_trusted_extensions.push(extension_id);
        save_preferences(&prefs)?;
    }
    Ok(preferences_dto())
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
) -> Result<ModToggleResultDto, String> {
    let root = crate::config::user_mods_dir().ok_or("config dir unavailable")?;
    unibrowser_mod_host::set_native_mod_enabled(&root, &name, enabled)
        .map_err(|e| e.to_string())?;
    state.refresh_extension_catalog()?;
    Ok(ModToggleResultDto {
        restart_recommended: !enabled,
        needs_trust: false,
    })
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
        } else if let Some(backend) = base_js::try_js_backend(&mut *doc.backend) {
            backend.content.clone()
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.content.clone()
        } else if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
            let project = std::path::PathBuf::from(&path);
            let root = unibrow_package_root(&project)?;
            crate::project_undo::read_manifest_json(&root)?
        } else {
            return Err("undo not available for this document".into());
        };
        let _sess_stacks = ws(&state, &window)?;
        let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
        let prev = stacks.undo(&path, &current).ok_or("nothing to undo")?;
        if let Some(backend) = try_text_backend(&mut *doc.backend) {
            backend.set_content(prev);
        } else if let Some(backend) = base_js::try_js_backend(&mut *doc.backend) {
            backend.set_content(prev);
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.set_content(prev);
        } else if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
            let project = std::path::PathBuf::from(&path);
            let root = unibrow_package_root(&project)?;
            crate::project_undo::write_manifest_json(&root, &prev)?;
            reload_unibrow_project_tab(state.inner(), window.label(), &project, path.clone())?;
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
        } else if let Some(backend) = base_js::try_js_backend(&mut *doc.backend) {
            backend.content.clone()
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.content.clone()
        } else if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
            let project = std::path::PathBuf::from(&path);
            let root = unibrow_package_root(&project)?;
            crate::project_undo::read_manifest_json(&root)?
        } else {
            return Err("redo not available for this document".into());
        };
        let _sess_stacks = ws(&state, &window)?;
        let mut stacks = _sess_stacks.undo_stacks.lock().map_err(|e| e.to_string())?;
        let next = stacks.redo(&path, &current).ok_or("nothing to redo")?;
        if let Some(backend) = try_text_backend(&mut *doc.backend) {
            backend.set_content(next);
        } else if let Some(backend) = base_js::try_js_backend(&mut *doc.backend) {
            backend.set_content(next);
        } else if let Some(backend) = base_svg::try_svg_backend(&mut *doc.backend) {
            backend.set_content(next);
        } else if doc.backend.backend_id() == crate::project_backend::BACKEND_ID {
            let project = std::path::PathBuf::from(&path);
            let root = unibrow_package_root(&project)?;
            crate::project_undo::write_manifest_json(&root, &next)?;
            reload_unibrow_project_tab(state.inner(), window.label(), &project, path.clone())?;
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
    for pin in &prefs.pinned_locations {
        let score = fuzzy_score(&query, &pin.label)
            .max(fuzzy_score(&query, &pin.location));
        if score < 0 && !query.is_empty() {
            continue;
        }
        out.push(OmniboxSuggestionDto {
            label: format!("★ {}", pin.label),
            value: pin.location.clone(),
            kind: "place".into(),
            score: score.max(0) + 60,
        });
    }
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
    let trimmed = query.trim();
    if !trimmed.is_empty() {
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            out.push(OmniboxSuggestionDto {
                label: trimmed.to_string(),
                value: trimmed.to_string(),
                kind: "url".into(),
                score: 90,
            });
        } else if !trimmed.contains(' ') && (trimmed.contains('.') || trimmed.starts_with("localhost")) {
            let guess = if trimmed.contains("://") {
                trimmed.to_string()
            } else {
                format!("https://{}", trimmed)
            };
            out.push(OmniboxSuggestionDto {
                label: format!("Open {}", guess),
                value: guess,
                kind: "url".into(),
                score: 70,
            });
        }
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
    out.truncate(20);
    Ok(out)
}

#[tauri::command]
pub fn audio_waveform_peaks(
    path: String,
    max_points: Option<usize>,
    state: State<AppState>,
) -> Result<Vec<f32>, String> {
    let p = resolve_open_path(&path, &state.cwd)?;
    let points = max_points.unwrap_or(320).clamp(32, 2048);
    unibrowser_media::audio::waveform_peaks(&p, points).map_err(|e| e.to_string())
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

#[tauri::command]
pub fn ocr_tesseract_available_cmd() -> bool {
    unibrowser_ocr::tesseract_available()
}

#[tauri::command]
pub fn terminal_spawn(
    app: AppHandle,
    cwd: Option<String>,
    state: State<AppState>,
) -> Result<u32, String> {
    let cwd = cwd.or_else(|| Some(state.cwd.to_string_lossy().into_owned()));
    crate::terminal_host::spawn_terminal(app, cwd)
}

#[tauri::command]
pub fn terminal_write(session_id: u32, data: String) -> Result<(), String> {
    crate::terminal_host::terminal_write(session_id, data)
}

#[tauri::command]
pub fn terminal_resize(session_id: u32, cols: u16, rows: u16) -> Result<(), String> {
    crate::terminal_host::terminal_resize(session_id, cols, rows)
}

fn ocr_active_tab(state: &AppState, window: &tauri::Window) -> Result<String, String> {
    if !unibrowser_ocr::tesseract_available() {
        return Err(
            "Install Tesseract OCR and add it to PATH (or set UNIBROWSER_TESSERACT_PATH).".into(),
        );
    }
    let path = active_document_path(state, window)?;
    let p = resolve_open_path(&path, &state.cwd)?;
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase());
    let text = match ext.as_deref() {
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("bmp") | Some("gif") => {
            unibrowser_ocr::ocr_image_file(&p).map_err(|e| e.to_string())?
        }
        Some("pdf") => ocr_pdf_page_one(&p)?,
        _ => return Err("OCR is available for image files and PDF documents.".into()),
    };
    if text.trim().is_empty() {
        return Err("OCR found no text in this document.".into());
    }
    Ok(format!("ocr-copy:{}", text))
}

fn ocr_pdf_page_one(pdf: &std::path::Path) -> Result<String, String> {
    use std::process::Command;
    let pdftoppm = unibrowser_media::find_binary("pdftoppm")
        .ok_or("PDF OCR needs pdftoppm (Poppler) on PATH.")?;
    let dir = std::env::temp_dir().join(format!("unibrowser-ocr-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let prefix = dir.join("page");
    let status = Command::new(&pdftoppm)
        .arg("-f")
        .arg("1")
        .arg("-l")
        .arg("1")
        .arg("-png")
        .arg(pdf)
        .arg(&prefix)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&dir);
        return Err("pdftoppm failed to rasterize PDF page 1".into());
    }
    let png = dir.join("page-1.png");
    let out = unibrowser_ocr::ocr_image_file(&png).map_err(|e| e.to_string());
    let _ = std::fs::remove_dir_all(&dir);
    out
}

fn active_web_reader_mode(state: &AppState, window_label: &str) -> bool {
    let sess = match state.session(window_label) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut tabs = match sess.tabs.lock() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let tab = match tabs.active_tab_mut() {
        Some(t) => t,
        None => return false,
    };
    let doc = match tab.document.as_mut() {
        Some(d) => d,
        None => return false,
    };
    base_web::try_web_backend_mut(&mut *doc.backend)
        .map(|w| w.reader_mode)
        .unwrap_or(false)
}

fn active_tab_muted(state: &AppState, window_label: &str) -> bool {
    let sess = ws_label(state, window_label).ok();
    let Some(sess) = sess else {
        return false;
    };
    let tabs = match sess.tabs.lock() {
        Ok(t) => t,
        Err(_) => return false,
    };
    tabs.active_tab().map(|t| t.muted).unwrap_or(false)
}

pub(crate) fn update_content_webview(
    app: &AppHandle,
    window_label: &str,
    loc: &BrowserLocation,
) -> Result<(), String> {
    if std::env::var("UNIBROWSER_SELF_TEST").as_deref() == Ok("1") {
        if matches!(loc, BrowserLocation::File { .. }) {
            return Ok(());
        }
    }
    let (reader, muted) = app
        .try_state::<AppState>()
        .map(|s| {
            (
                active_web_reader_mode(s.inner(), window_label),
                active_tab_muted(s.inner(), window_label),
            )
        })
        .unwrap_or((false, false));
    sync_content_webview_for_window(app, window_label, loc, reader, muted)
}
