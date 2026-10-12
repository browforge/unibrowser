use base_text::{TextBackend, try_text_backend};
use serde::Serialize;
use unibrowser_engine::{DocumentSession, TabManager};
use unibrowser_mod_api::BrowserLocation;

use crate::host::WebHost;
use crate::prefs::WebPrefs;

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
pub struct TabDto {
    pub id: u64,
    pub label: String,
    pub location_kind: String,
    pub is_directory: bool,
    pub is_http: bool,
    pub dirty: bool,
    pub pinned: bool,
    pub browse_path: Option<String>,
    pub muted: bool,
    pub mute_available: bool,
}

#[derive(Serialize)]
pub struct FilesViewDto {
    pub sort: String,
    pub sort_ascending: bool,
    pub show_hidden: bool,
}

#[derive(Serialize)]
pub struct NtpSectionsDto {
    pub recent_files: bool,
    pub pinned_places: bool,
    pub demo_pack: bool,
    pub tips: bool,
}

#[derive(Serialize)]
pub struct TabContentDto {
    pub tabs: Vec<TabDto>,
    pub active_index: usize,
    pub omnibox: String,
    pub mods_loaded: String,
    pub status: String,
    pub directory_entries: Option<serde_json::Value>,
    pub document_status: Option<String>,
    pub text_document: Option<TextDocumentDto>,
    pub mod_portal: Option<serde_json::Value>,
    pub unknown_file_banner: Option<serde_json::Value>,
    pub document_viewer: Option<serde_json::Value>,
    pub show_onboarding: bool,
    pub show_onboarding_tour: bool,
    pub recent_files: Vec<serde_json::Value>,
    pub split_view_enabled: bool,
    pub split_index: Option<usize>,
    pub split_text_document: Option<TextDocumentDto>,
    pub split_document_viewer: Option<serde_json::Value>,
    pub split_status: Option<String>,
    pub show_new_tab_page: bool,
    pub browse_folder_path: String,
    pub diff_view: Option<serde_json::Value>,
    pub image_compare_view: Option<serde_json::Value>,
    pub pinned_locations: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome_notice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_tip: Option<serde_json::Value>,
    pub close_undo_available: bool,
    pub recovery_bar: Vec<serde_json::Value>,
    pub ntp_sections: NtpSectionsDto,
    pub files_view: FilesViewDto,
    pub folder_favorites: Vec<String>,
    pub workspace_root: Option<String>,
    pub git_is_repo: bool,
    pub git_dirty_paths: Vec<String>,
    pub download_history: Vec<serde_json::Value>,
    pub bookmark_bar_enabled: bool,
    pub bookmark_bar_pins: Vec<serde_json::Value>,
    pub bookmark_quick_links: Vec<serde_json::Value>,
}

pub fn location_to_omnibox(loc: &BrowserLocation) -> String {
    match loc {
        BrowserLocation::File { path } => path.to_string_lossy().into_owned(),
        BrowserLocation::Http { url } => url.clone(),
        BrowserLocation::Internal { page } => format!("unibrowser://{}", page),
    }
}

fn tab_is_home(tab: &unibrowser_engine::tabs::Tab) -> bool {
    matches!(
        &tab.location,
        BrowserLocation::Internal { page } if page == "home"
    )
}

pub fn tab_label(tab: &unibrowser_engine::tabs::Tab) -> String {
    match &tab.location {
        BrowserLocation::Internal { page } if page == "home" => "New tab".into(),
        BrowserLocation::Internal { page } if page == "settings" => "Settings".into(),
        BrowserLocation::File { path } => path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned()),
        BrowserLocation::Http { url } => url.clone(),
        BrowserLocation::Internal { page } => page.clone(),
    }
}

fn tab_location_kind(tab: &unibrowser_engine::tabs::Tab) -> String {
    if tab_is_home(tab) {
        return "New tab".into();
    }
    if tab.is_directory {
        return "Folder".into();
    }
    match &tab.location {
        BrowserLocation::Http { .. } => "Web".into(),
        BrowserLocation::Internal { page } if page == "settings" => "Settings".into(),
        BrowserLocation::Internal { .. } => "Page".into(),
        BrowserLocation::File { path } => path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_uppercase())
            .unwrap_or_else(|| "File".into()),
    }
}

fn text_document_dto(
    doc: &mut DocumentSession,
    reveal_line: Option<u32>,
) -> Option<TextDocumentDto> {
    if let Some(backend) = try_text_backend(&mut *doc.backend) {
        return Some(TextDocumentDto {
            path: backend.path.clone(),
            content: backend.content.clone(),
            font_size_pt: backend.font_size_pt,
            language: TextBackend::language_id(&backend.path).into(),
            dirty: backend.dirty,
            encoding: "UTF-8".into(),
            reveal_line,
        });
    }
    None
}

pub fn build_snapshot(host: &mut WebHost, prefs: &WebPrefs) -> TabContentDto {
    let tabs = &mut host.tabs;
    let omnibox = tabs
        .active_tab()
        .map(|t| location_to_omnibox(&t.location))
        .unwrap_or_else(|| "unibrowser://home".into());
    let show_new_tab_page = tabs.active_tab().map(tab_is_home).unwrap_or(true);
    let show_onboarding = !prefs.onboarding_dismissed;
    let show_onboarding_tour =
        !prefs.onboarding_tour_skipped_forever && !prefs.onboarding_dismissed;

    let (text_document, document_status) = match tabs.active_tab_mut() {
        Some(tab) => {
            let reveal = tab.reveal_line;
            if reveal.is_some() {
                tab.reveal_line = None;
            }
            if let Some(doc) = tab.document.as_mut() {
                let status = doc.backend.status_line();
                let text = text_document_dto(doc, reveal);
                (text, Some(status))
            } else {
                (None, None)
            }
        }
        None => (None, None),
    };

    let command_count = host.registry.commands().count();
    let fallback_status = format!("{} commands (web demo)", command_count);
    let status = document_status.unwrap_or(fallback_status);

    TabContentDto {
        tabs: tabs
            .tabs()
            .iter()
            .map(|t| {
                let dirty = t
                    .document
                    .as_ref()
                    .map(|d| d.backend.is_dirty())
                    .unwrap_or(false);
                TabDto {
                    id: t.id.0,
                    label: tab_label(t),
                    location_kind: tab_location_kind(t),
                    is_directory: t.is_directory,
                    is_http: matches!(&t.location, BrowserLocation::Http { .. }),
                    dirty,
                    pinned: t.pinned,
                    browse_path: None,
                    muted: t.muted,
                    mute_available: false,
                }
            })
            .collect(),
        active_index: tabs.active_index(),
        omnibox,
        mods_loaded: host.mods_line.clone(),
        status: status.clone(),
        directory_entries: None,
        document_status: Some(status),
        text_document,
        mod_portal: None,
        unknown_file_banner: host.last_unknown_banner.clone(),
        document_viewer: None,
        show_onboarding,
        show_onboarding_tour,
        recent_files: host
            .recent_files
            .iter()
            .map(|r| {
                serde_json::json!({
                    "path": r.path,
                    "label": r.label,
                    "pinned": r.pinned,
                    "opened_at_ms": r.opened_at_ms
                })
            })
            .collect(),
        split_view_enabled: false,
        split_index: None,
        split_text_document: None,
        split_document_viewer: None,
        split_status: None,
        show_new_tab_page,
        browse_folder_path: String::new(),
        diff_view: None,
        image_compare_view: None,
        pinned_locations: vec![],
        chrome_notice: host.chrome_notice.take(),
        extension_tip: None,
        close_undo_available: !host.closed_tabs.is_empty(),
        recovery_bar: vec![],
        ntp_sections: NtpSectionsDto {
            recent_files: true,
            pinned_places: true,
            demo_pack: true,
            tips: true,
        },
        files_view: FilesViewDto {
            sort: "name".into(),
            sort_ascending: true,
            show_hidden: false,
        },
        folder_favorites: vec![],
        workspace_root: None,
        git_is_repo: false,
        git_dirty_paths: vec![],
        download_history: vec![],
        bookmark_bar_enabled: false,
        bookmark_bar_pins: vec![],
        bookmark_quick_links: vec![],
    }
}

pub fn open_text_backend(path: String, content: String) -> DocumentSession {
    let backend = TextBackend {
        path: path.clone(),
        content,
        selection_start: 0,
        selection_end: 0,
        font_size_pt: 14.0,
        dirty: false,
    };
    DocumentSession::new(path, Box::new(backend))
}

pub fn is_text_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".txt")
        || lower.ends_with(".md")
        || lower.ends_with(".markdown")
        || lower.ends_with(".json")
        || lower.ends_with(".ts")
        || lower.ends_with(".rs")
        || lower.ends_with(".css")
        || lower.ends_with(".html")
}

pub fn navigate_tabs(tabs: &mut TabManager, loc: BrowserLocation, doc: Option<DocumentSession>) {
    let is_dir = false;
    tabs.navigate_active(loc, is_dir, doc);
}
