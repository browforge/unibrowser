use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPrefs {
    pub onboarding_dismissed: bool,
    pub onboarding_tour_skipped_forever: bool,
    pub ui_locale: String,
    pub theme: String,
    pub editor_theme_id: String,
    pub offline_mode: bool,
}

impl Default for WebPrefs {
    fn default() -> Self {
        Self {
            onboarding_dismissed: false,
            onboarding_tour_skipped_forever: false,
            ui_locale: "en-US".into(),
            theme: "system".into(),
            editor_theme_id: "auto".into(),
            offline_mode: true,
        }
    }
}

#[derive(Serialize)]
struct PreferencesDto {
    onboarding_dismissed: bool,
    onboarding_tour_skipped_forever: bool,
    ui_locale: String,
    onboarding_checklist_skip_forever: bool,
    onboarding_checklist_completed: Vec<String>,
    theme: String,
    editor_theme_id: String,
    command_manager_dense: bool,
    cm_pin_tip_seen: bool,
    high_contrast: bool,
    ui_scale_percent: u16,
    reduce_motion: String,
    restore_session_on_launch: bool,
    auto_save_enabled: bool,
    auto_save_interval_secs: u32,
    auto_save_slot_count: u32,
    download_preference: String,
    board_snap_to_grid: bool,
    midi_scrub_audio_enabled: bool,
    fluidsynth_soundfont_path: Option<String>,
    project_autosave_enabled: bool,
    project_autosave_interval_secs: u32,
    crash_recovery_enabled: bool,
    ntp_sections: NtpSectionsDto,
    advanced_webview_devtools: bool,
    advanced_experimental_spellcheck: bool,
    proofing_squiggles_enabled: bool,
    proofing_grammar_enabled: bool,
    advanced_node_extension_host: bool,
    open_downloads_in_system_browser: bool,
    files_view: FilesViewDto,
    bookmark_bar_enabled: bool,
    bookmark_bar_pins: Vec<String>,
    bookmark_quick_links: Vec<String>,
    workspace_root: Option<String>,
    disable_user_mods: bool,
    safe_mode_env: bool,
    chrome_focus_mode: bool,
    larger_hit_targets: bool,
    web_proxy: WebProxyPrefsDto,
    warn_insecure_http: bool,
    offline_mode: bool,
    vsix_trusted_extensions: Vec<String>,
}

#[derive(Serialize)]
struct NtpSectionsDto {
    recent_files: bool,
    pinned_places: bool,
    demo_pack: bool,
    tips: bool,
}

#[derive(Serialize)]
struct FilesViewDto {
    sort: String,
    sort_ascending: bool,
    show_hidden: bool,
}

#[derive(Serialize)]
struct WebProxyPrefsDto {
    mode: String,
    host: String,
    port: u16,
}

impl WebPrefs {
    pub fn to_preferences_dto(&self) -> serde_json::Value {
        let dto = PreferencesDto {
            onboarding_dismissed: self.onboarding_dismissed,
            onboarding_tour_skipped_forever: self.onboarding_tour_skipped_forever,
            ui_locale: self.ui_locale.clone(),
            onboarding_checklist_skip_forever: false,
            onboarding_checklist_completed: vec![],
            theme: self.theme.clone(),
            editor_theme_id: self.editor_theme_id.clone(),
            command_manager_dense: false,
            cm_pin_tip_seen: false,
            high_contrast: false,
            ui_scale_percent: 100,
            reduce_motion: "system".into(),
            restore_session_on_launch: false,
            auto_save_enabled: false,
            auto_save_interval_secs: 300,
            auto_save_slot_count: 3,
            download_preference: "open_in_new_tab".into(),
            board_snap_to_grid: true,
            midi_scrub_audio_enabled: false,
            fluidsynth_soundfont_path: None,
            project_autosave_enabled: false,
            project_autosave_interval_secs: 60,
            crash_recovery_enabled: false,
            ntp_sections: NtpSectionsDto {
                recent_files: true,
                pinned_places: true,
                demo_pack: true,
                tips: true,
            },
            advanced_webview_devtools: false,
            advanced_experimental_spellcheck: false,
            proofing_squiggles_enabled: false,
            proofing_grammar_enabled: false,
            advanced_node_extension_host: false,
            open_downloads_in_system_browser: false,
            files_view: FilesViewDto {
                sort: "name".into(),
                sort_ascending: true,
                show_hidden: false,
            },
            bookmark_bar_enabled: false,
            bookmark_bar_pins: vec![],
            bookmark_quick_links: vec![],
            workspace_root: None,
            disable_user_mods: true,
            safe_mode_env: false,
            chrome_focus_mode: false,
            larger_hit_targets: false,
            web_proxy: WebProxyPrefsDto {
                mode: "system".into(),
                host: String::new(),
                port: 0,
            },
            warn_insecure_http: true,
            offline_mode: self.offline_mode,
            vsix_trusted_extensions: vec![],
        };
        serde_json::to_value(dto).unwrap_or(serde_json::Value::Null)
    }
}
