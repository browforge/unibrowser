use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::preferences_file_path;

pub const EPUB_POSITIONS_MAX: usize = 128;
pub const EPUB_NOTES_MAX: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UiTheme {
    Light,
    Dark,
    System,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReduceMotionPref {
    #[default]
    System,
    On,
    Off,
}

pub const RECENT_FILES_MAX: usize = 12;

/// How completed downloads are handled (ADR 0009). Default: open in a new tab.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DownloadPreference {
    #[default]
    OpenInNewTab,
    SaveToDisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentFileEntry {
    pub path: String,
    pub label: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub opened_at_ms: u64,
    /// NTP kind label (e.g. Project, PDF, Text).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    #[serde(default)]
    pub onboarding_dismissed: bool,
    /// First-run tour card; when true, tour never shows again.
    #[serde(default)]
    pub onboarding_tour_skipped_forever: bool,
    /// UI catalogue: `en-NZ` or `en-US`.
    #[serde(default = "default_ui_locale")]
    pub ui_locale: String,
    #[serde(default)]
    pub theme: UiTheme,
    /// Monaco editor theme id (`auto`, `vs`, `vs-dark`, or `vsix:…`).
    #[serde(default = "default_editor_theme_id")]
    pub editor_theme_id: String,
    #[serde(default)]
    pub recent_files: Vec<RecentFileEntry>,
    /// When true, last tab set is saved on exit and restored on launch.
    #[serde(default)]
    pub restore_session_on_launch: bool,
    /// Automatic save for text/SVG (on by default).
    #[serde(default = "default_document_default_mode")]
    pub document_default_mode: String,
    #[serde(default = "default_auto_save_enabled")]
    pub auto_save_enabled: bool,
    /// Seconds between auto-save attempts (Factorio-style presets when enabled).
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_secs: u32,
    /// Rotating autosave slots per document (sidecar dir).
    #[serde(default = "default_auto_save_slot_count")]
    pub auto_save_slot_count: u32,
    #[serde(default)]
    pub pinned_locations: Vec<PinnedLocation>,
    #[serde(default)]
    pub download_preference: DownloadPreference,
    /// Snap board card positions to a 24px grid in `.unibrow` projects.
    #[serde(default)]
    pub board_snap_to_grid: bool,
    /// Play short Web Audio beeps when MIDI clips are scrubbed on the timeline.
    #[serde(default)]
    pub midi_scrub_audio_enabled: bool,
    /// Optional `.sf2` for FluidSynth when timeline export renders MIDI to WAV.
    #[serde(default)]
    pub fluidsynth_soundfont_path: Option<String>,
    /// Backup `project.json` under `derived/` for open `.unibrow` tabs.
    #[serde(default)]
    pub project_autosave_enabled: bool,
    #[serde(default = "default_auto_save_interval")]
    pub project_autosave_interval_secs: u32,
    /// Backup dirty text/SVG buffers under config dir (off by default).
    #[serde(default)]
    pub crash_recovery_enabled: bool,
    #[serde(default)]
    pub ntp_sections: NtpSectionPrefs,
    /// When true, embedded https webviews may open WebKit/Chromium devtools (advanced).
    #[serde(default)]
    pub advanced_webview_devtools: bool,
    /// Read-only browser spellcheck mirror under the text editor (experimental).
    #[serde(default)]
    pub advanced_experimental_spellcheck: bool,
    /// Offline squiggle proofing (unibrowser-proofing engine).
    #[serde(default = "default_proofing_squiggles")]
    pub proofing_squiggles_enabled: bool,
    #[serde(default = "default_proofing_grammar")]
    pub proofing_grammar_enabled: bool,
    /// Run installed VSIX `main` modules via Node sidecar (experimental).
    #[serde(default)]
    pub advanced_node_extension_host: bool,
    /// Extension IDs the user trusted to run JavaScript (Node host).
    #[serde(default)]
    pub vsix_trusted_extensions: Vec<String>,
    /// Interim: hand https downloads to the system browser (WebView2 gap on Windows).
    #[serde(default)]
    pub open_downloads_in_system_browser: bool,
    #[serde(default)]
    pub files_view: FilesViewPrefs,
    /// Favourite folder paths (bookmarked from files view).
    #[serde(default)]
    pub folder_favorites: Vec<String>,
    /// Compact (40px) Command Manager row; false = Large (64px).
    #[serde(default = "default_command_manager_dense")]
    pub command_manager_dense: bool,
    /// User has seen the Command Manager pin tip (toast or Settings).
    #[serde(default)]
    pub cm_pin_tip_seen: bool,
    /// High-contrast chrome (strong borders, Audacity-style accents).
    #[serde(default)]
    pub high_contrast: bool,
    /// Persistent interface text scale percentage (100, 125, or 150).
    #[serde(default = "default_ui_scale_percent")]
    pub ui_scale_percent: u16,
    #[serde(default)]
    pub reduce_motion: ReduceMotionPref,
    /// Hide the scripting mod suggestion bar on JS/TS files opened as plain text.
    #[serde(default)]
    pub scripting_extension_tip_dismissed_forever: bool,
    /// Hide the Settings onboarding checklist permanently.
    #[serde(default)]
    pub onboarding_checklist_skip_forever: bool,
    #[serde(default)]
    pub onboarding_checklist_completed: Vec<String>,
    /// Last-opened spine chapter index per EPUB path.
    #[serde(default)]
    pub epub_chapter_by_path: HashMap<String, usize>,
    /// Reading notes textarea per EPUB path (local prefs only).
    #[serde(default)]
    pub epub_reading_notes_by_path: HashMap<String, String>,
    /// Bookmark bar under chrome (pins + quick links).
    #[serde(default = "default_bookmark_bar_enabled")]
    pub bookmark_bar_enabled: bool,
    /// Extra https/file links shown on the bookmark bar after pins.
    #[serde(default)]
    pub bookmark_quick_links: Vec<BookmarkQuickLink>,
    /// Workspace root for capped recursive Files tree ([docs/workspace-folders.md]).
    #[serde(default)]
    pub workspace_root: Option<String>,
    /// DOCX HTML preview font size per file path (autosave / prefs).
    #[serde(default)]
    pub docx_reader_font_pt: HashMap<String, f32>,
    /// Skip user `mods/` and dev cdylib loads (bundled mods only). Requires restart.
    #[serde(default)]
    pub disable_user_mods: bool,
    /// Denser chrome: compact Command Manager and smaller tabs.
    #[serde(default)]
    pub chrome_focus_mode: bool,
    /// Larger minimum hit targets on tabs and command chips.
    #[serde(default)]
    pub larger_hit_targets: bool,
    /// Web tab proxy preference (manual mode → `WebviewBuilder::proxy_url` on new content webviews).
    #[serde(default)]
    pub web_proxy: WebProxyPrefs,
    /// Confirm before omnibox navigates to plain `http://` (default on).
    #[serde(default = "default_true")]
    pub warn_insecure_http: bool,
    /// Block Open VSX and other catalog fetches; web/https tabs still load when you navigate.
    #[serde(default)]
    pub offline_mode: bool,
    /// User-selected default surface provider per `unibrowser.capability.*` id.
    #[serde(default)]
    pub capability_default_providers: HashMap<String, String>,
    /// Remembered Open-with provider mod name per file extension (no leading dot).
    #[serde(default)]
    pub open_with_by_extension: HashMap<String, String>,
    /// Remembered Open-with provider mod name per absolute file path.
    #[serde(default)]
    pub open_with_by_path: HashMap<String, String>,
}

/// How https/file web tabs should reach the network (prefs only until webview proxy API lands).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WebProxyMode {
    #[default]
    System,
    Direct,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WebProxyPrefs {
    #[serde(default)]
    pub mode: WebProxyMode,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
}

/// Safe mode: env wins for this process; prefs apply on next launch.
pub fn user_mods_disabled() -> bool {
    if std::env::var("UNIBROWSER_SAFE_MODE").as_deref() == Ok("1") {
        return true;
    }
    load_preferences().disable_user_mods
}

fn default_proofing_squiggles() -> bool {
    true
}

fn default_proofing_grammar() -> bool {
    true
}

fn default_bookmark_bar_enabled() -> bool {
    false
}

fn default_command_manager_dense() -> bool {
    true
}

fn default_ui_scale_percent() -> u16 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookmarkQuickLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PinnedLocation {
    pub id: String,
    pub label: String,
    pub location: String,
}

fn default_document_default_mode() -> String {
    "wysiwyg".into()
}

fn default_auto_save_enabled() -> bool {
    true
}

fn default_auto_save_interval() -> u32 {
    300
}

fn default_auto_save_slot_count() -> u32 {
    5
}

pub fn normalize_auto_save_interval_secs(secs: u32) -> u32 {
    const ALLOWED: [u32; 6] = [60, 120, 300, 600, 900, 1800];
    if ALLOWED.contains(&secs) {
        secs
    } else {
        default_auto_save_interval()
    }
}

fn default_editor_theme_id() -> String {
    "auto".into()
}

fn default_ui_locale() -> String {
    "en-NZ".into()
}

fn default_true() -> bool {
    true
}

/// Which blocks appear on the new tab page (`unibrowser://home`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NtpSectionPrefs {
    #[serde(default = "default_true")]
    pub show_hero: bool,
    #[serde(default = "default_true")]
    pub show_actions: bool,
    #[serde(default = "default_true")]
    pub show_browse: bool,
    #[serde(default = "default_true")]
    pub show_recent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FilesSortMode {
    #[default]
    Name,
    Date,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesViewPrefs {
    #[serde(default)]
    pub sort: FilesSortMode,
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
    #[serde(default)]
    pub show_hidden: bool,
}

impl Default for FilesViewPrefs {
    fn default() -> Self {
        Self {
            sort: FilesSortMode::Name,
            sort_ascending: true,
            show_hidden: false,
        }
    }
}

impl Default for NtpSectionPrefs {
    fn default() -> Self {
        Self {
            show_hero: true,
            show_actions: true,
            show_browse: false,
            show_recent: true,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            onboarding_dismissed: false,
            onboarding_tour_skipped_forever: false,
            ui_locale: default_ui_locale(),
            theme: UiTheme::default(),
            editor_theme_id: default_editor_theme_id(),
            document_default_mode: default_document_default_mode(),
            recent_files: Vec::new(),
            restore_session_on_launch: true,
            auto_save_enabled: true,
            auto_save_interval_secs: default_auto_save_interval(),
            auto_save_slot_count: default_auto_save_slot_count(),
            pinned_locations: Vec::new(),
            download_preference: DownloadPreference::default(),
            board_snap_to_grid: false,
            midi_scrub_audio_enabled: false,
            fluidsynth_soundfont_path: None,
            project_autosave_enabled: false,
            project_autosave_interval_secs: default_auto_save_interval(),
            crash_recovery_enabled: false,
            ntp_sections: NtpSectionPrefs::default(),
            advanced_webview_devtools: false,
            advanced_experimental_spellcheck: false,
            proofing_squiggles_enabled: default_proofing_squiggles(),
            proofing_grammar_enabled: default_proofing_grammar(),
            advanced_node_extension_host: false,
            vsix_trusted_extensions: Vec::new(),
            open_downloads_in_system_browser: false,
            files_view: FilesViewPrefs::default(),
            folder_favorites: Vec::new(),
            command_manager_dense: default_command_manager_dense(),
            cm_pin_tip_seen: false,
            high_contrast: false,
            ui_scale_percent: default_ui_scale_percent(),
            reduce_motion: ReduceMotionPref::default(),
            onboarding_checklist_skip_forever: false,
            onboarding_checklist_completed: Vec::new(),
            epub_chapter_by_path: HashMap::new(),
            epub_reading_notes_by_path: HashMap::new(),
            bookmark_bar_enabled: default_bookmark_bar_enabled(),
            bookmark_quick_links: Vec::new(),
            workspace_root: None,
            docx_reader_font_pt: HashMap::new(),
            disable_user_mods: false,
            chrome_focus_mode: false,
            larger_hit_targets: false,
            scripting_extension_tip_dismissed_forever: false,
            web_proxy: WebProxyPrefs::default(),
            warn_insecure_http: true,
            offline_mode: false,
            capability_default_providers: HashMap::new(),
            open_with_by_extension: HashMap::new(),
            open_with_by_path: HashMap::new(),
        }
    }
}

pub fn complete_checklist_step(prefs: &mut Preferences, step_id: &str) {
    let step_id = step_id.trim();
    if step_id.is_empty() {
        return;
    }
    if prefs.onboarding_checklist_completed.iter().any(|s| s == step_id) {
        return;
    }
    prefs.onboarding_checklist_completed.push(step_id.to_string());
}

pub fn toggle_folder_favorite(prefs: &mut Preferences, path: &str) {
    if let Some(idx) = prefs.folder_favorites.iter().position(|p| p == path) {
        prefs.folder_favorites.remove(idx);
        return;
    }
    prefs.folder_favorites.insert(0, path.to_string());
    if prefs.folder_favorites.len() > 24 {
        prefs.folder_favorites.truncate(24);
    }
}

pub fn default_pinned_locations(home: &str) -> Vec<PinnedLocation> {
    vec![
        PinnedLocation {
            id: "home".into(),
            label: "Home".into(),
            location: format!("file://{}", home),
        },
        PinnedLocation {
            id: "downloads".into(),
            label: "Downloads".into(),
            location: format!(
                "file://{}",
                crate::paths::user_downloads_dir().to_string_lossy()
            ),
        },
        PinnedLocation {
            id: "download-manager".into(),
            label: "Download manager".into(),
            location: "unibrowser://downloads".into(),
        },
        PinnedLocation {
            id: "new-project".into(),
            label: "New project".into(),
            location: "unibrowser://action/new-project".into(),
        },
        PinnedLocation {
            id: "settings".into(),
            label: "Settings".into(),
            location: "unibrowser://settings".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_preferences_source_has_no_doc_capture_env_fork() {
        let src = include_str!("preferences.rs");
        let body = src
            .split("pub fn load_preferences()")
            .nth(1)
            .and_then(|rest| rest.split("pub fn save_preferences").next())
            .unwrap_or("");
        assert!(
            !body.contains("UNIBROWSER_DOC_CAPTURE"),
            "load_preferences must not branch on doc-capture env; use isolated XDG config for captures"
        );
    }

    #[test]
    fn checklist_step_dedupes() {
        let mut p = Preferences::default();
        complete_checklist_step(&mut p, "palette");
        complete_checklist_step(&mut p, "palette");
        assert_eq!(p.onboarding_checklist_completed.len(), 1);
    }

    #[test]
    fn recent_files_dedupe_and_cap() {
        let mut p = Preferences::default();
        for i in 0..20 {
            push_recent_file(&mut p, format!("/tmp/f{i}.txt"), format!("f{i}"));
        }
        assert_eq!(p.recent_files.len(), RECENT_FILES_MAX);
        assert_eq!(p.recent_files[0].label, "f19");
        push_recent_file(&mut p, "/tmp/f5.txt".into(), "f5".into());
        assert_eq!(p.recent_files[0].path, "/tmp/f5.txt");
        assert!(p.recent_files.iter().filter(|e| e.path == "/tmp/f5.txt").count() == 1);
    }

    #[test]
    fn theme_change_reads_back() {
        let mut prefs = Preferences::default();
        assert_eq!(prefs.theme, UiTheme::System);
        prefs.theme = UiTheme::Dark;
        let json = serde_json::to_string(&prefs).expect("serialize");
        let loaded: Preferences = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(loaded.theme, UiTheme::Dark);
    }

    #[test]
    fn autosave_interval_change_reads_back() {
        let mut prefs = Preferences::default();
        assert_eq!(prefs.auto_save_interval_secs, 300);
        prefs.auto_save_interval_secs = normalize_auto_save_interval_secs(60);
        let json = serde_json::to_string(&prefs).expect("serialize");
        let loaded: Preferences = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(loaded.auto_save_interval_secs, 60);
        assert_eq!(normalize_auto_save_interval_secs(45), 300);
    }

    #[test]
    fn export_epub_notes_writes_sibling_txt() {
        let dir = std::env::temp_dir().join(format!("ub-epub-notes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let book = dir.join("sample.epub");
        std::fs::write(&book, b"PK").expect("epub stub");
        remember_epub_reading_notes(book.to_string_lossy().as_ref(), "line one")
            .expect("remember");
        let out = export_epub_reading_notes_file(book.to_string_lossy().as_ref())
            .expect("export");
        assert!(out.ends_with("sample.reading-notes.txt"));
        let text = std::fs::read_to_string(&out).expect("read");
        assert_eq!(text, "line one");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

pub fn epub_read_chapter_index(path: &str) -> Option<usize> {
    load_preferences()
        .epub_chapter_by_path
        .get(path)
        .copied()
}

pub fn remember_epub_chapter(path: &str, index: usize) -> Result<(), String> {
    let mut prefs = load_preferences();
    prefs.epub_chapter_by_path.insert(path.to_string(), index);
    while prefs.epub_chapter_by_path.len() > EPUB_POSITIONS_MAX {
        let key = prefs
            .epub_chapter_by_path
            .keys()
            .next()
            .cloned()
            .unwrap_or_default();
        prefs.epub_chapter_by_path.remove(&key);
    }
    save_preferences(&prefs)
}

pub fn epub_reading_notes(path: &str) -> String {
    load_preferences()
        .epub_reading_notes_by_path
        .get(path)
        .cloned()
        .unwrap_or_default()
}

pub fn remember_epub_reading_notes(path: &str, notes: &str) -> Result<(), String> {
    let mut prefs = load_preferences();
    let trimmed = notes.trim();
    if trimmed.is_empty() {
        prefs.epub_reading_notes_by_path.remove(path);
    } else {
        prefs
            .epub_reading_notes_by_path
            .insert(path.to_string(), notes.to_string());
    }
    while prefs.epub_reading_notes_by_path.len() > EPUB_NOTES_MAX {
        let key = prefs
            .epub_reading_notes_by_path
            .keys()
            .next()
            .cloned()
            .unwrap_or_default();
        prefs.epub_reading_notes_by_path.remove(&key);
    }
    save_preferences(&prefs)
}

pub fn export_epub_reading_notes_file(epub_path: &str) -> Result<String, String> {
    use std::path::Path;
    let book = Path::new(epub_path);
    if !book.is_file() {
        return Err("not a file".into());
    }
    let notes = epub_reading_notes(epub_path);
    let stem = book
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "book".into());
    let out = book.with_file_name(format!("{}.reading-notes.txt", stem));
    std::fs::write(&out, notes.as_bytes()).map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().into_owned())
}

pub fn push_recent_file(prefs: &mut Preferences, path: String, label: String) {
    push_recent_entry(prefs, path, label, None);
}

pub fn push_recent_entry(
    prefs: &mut Preferences,
    path: String,
    label: String,
    kind: Option<String>,
) {
    let pinned = prefs
        .recent_files
        .iter()
        .find(|e| e.path == path)
        .map(|e| e.pinned)
        .unwrap_or(false);
    prefs.recent_files.retain(|e| e.path != path);
    let opened_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    prefs.recent_files.insert(
        0,
        RecentFileEntry {
            path,
            label,
            pinned,
            opened_at_ms,
            kind,
        },
    );
    if prefs.recent_files.len() > RECENT_FILES_MAX {
        prefs.recent_files.truncate(RECENT_FILES_MAX);
    }
}

pub fn toggle_recent_pin(prefs: &mut Preferences, path: &str) -> bool {
    if let Some(entry) = prefs.recent_files.iter_mut().find(|e| e.path == path) {
        entry.pinned = !entry.pinned;
        return entry.pinned;
    }
    false
}

pub fn clear_unpinned_recent(prefs: &mut Preferences) {
    prefs.recent_files.retain(|e| e.pinned);
}

pub fn clear_all_recent(prefs: &mut Preferences) {
    prefs.recent_files.clear();
}

pub fn reset_onboarding_prefs(prefs: &mut Preferences) {
    prefs.onboarding_dismissed = false;
    prefs.onboarding_tour_skipped_forever = false;
    prefs.onboarding_checklist_skip_forever = false;
    prefs.onboarding_checklist_completed.clear();
}

pub fn load_preferences() -> Preferences {
    let path = preferences_file_path();
    if let Some(path) = path {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(p) = serde_json::from_str(&data) {
                return p;
            }
        }
    }
    Preferences::default()
}

pub fn save_preferences(prefs: &Preferences) -> Result<(), String> {
    let path = preferences_file_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            x: None,
            y: None,
        }
    }
}

pub fn load_window_state() -> WindowState {
    let path = crate::config::window_state_file_path();
    if let Some(path) = path {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str(&data) {
                return s;
            }
        }
    }
    WindowState::default()
}

pub fn save_window_state(state: &WindowState) -> Result<(), String> {
    let path = crate::config::window_state_file_path().ok_or("config path unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_window_state(window: &tauri::Window, state: &WindowState) -> Result<(), String> {
    use tauri::LogicalSize;
    window
        .set_size(LogicalSize::new(state.width, state.height))
        .map_err(|e| e.to_string())?;
    if let (Some(x), Some(y)) = (state.x, state.y) {
        use tauri::LogicalPosition;
        window
            .set_position(LogicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn docx_reader_font(path: &str) -> Option<f32> {
    load_preferences()
        .docx_reader_font_pt
        .get(path)
        .copied()
}

pub fn remember_docx_reader_font(path: &str, pt: f32) {
    let mut prefs = load_preferences();
    prefs
        .docx_reader_font_pt
        .insert(path.to_string(), pt);
    let _ = save_preferences(&prefs);
}

pub fn capture_window_state(window: &tauri::Window) -> Result<WindowState, String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok(WindowState {
        width: size.width as f64,
        height: size.height as f64,
        x: Some(pos.x as f64),
        y: Some(pos.y as f64),
    })
}
