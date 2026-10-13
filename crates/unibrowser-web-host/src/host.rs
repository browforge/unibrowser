use base_text::TextBackend;
use serde::Serialize;
use serde_json::Value;
use unibrowser_engine::{
    CommandManagerState, CommandRegistry, DocumentSession, KeybindingMap, LoadedModContribution,
    ModRuntimeRegistry, TabManager, infer_context, palette::search_commands,
    ui_command::CommandManagerFile,
};
use unibrowser_mod_api::{BrowserLocation, CommandContext, HostContext};

use crate::ai_prefs::AiAssistantPrefsWeb;
use crate::prefs::WebPrefs;
use crate::snapshot::{
    build_snapshot, is_text_path, navigate_tabs, open_text_backend, TabContentDto,
};

const WEB_UNAVAILABLE: &str = "Available in the desktop app";

pub struct RecentFile {
    pub path: String,
    pub label: String,
    pub pinned: bool,
    pub opened_at_ms: u64,
}

pub struct WebHost {
    pub registry: CommandRegistry,
    pub tabs: TabManager,
    pub command_manager: CommandManagerState,
    pub keybindings: KeybindingMap,
    pub prefs: WebPrefs,
    pub mods_line: String,
    pub recent_files: Vec<RecentFile>,
    pub closed_tabs: Vec<BrowserLocation>,
    pub chrome_notice: Option<String>,
    pub last_unknown_banner: Option<Value>,
    pub mod_runtime: ModRuntimeRegistry,
    pub pending_save: Option<(String, String)>,
    pub ai_prefs: AiAssistantPrefsWeb,
}

impl WebHost {
    pub fn new() -> Self {
        let mut registry = CommandRegistry::with_builtins();
        base_text::register(&mut registry);
        toy_markdown::register(&mut registry);
        let cm_file = CommandManagerFile::factory_default();
        let cm = CommandManagerState::with_layout_path(
            &cm_file,
            CommandContext::Start,
            None,
        );
        let home = BrowserLocation::Internal {
            page: "home".into(),
        };
        Self {
            registry,
            tabs: TabManager::new_with_home(home),
            command_manager: cm,
            keybindings: KeybindingMap::default(),
            prefs: WebPrefs::default(),
            mods_line: "base-text, toy-markdown (web)".into(),
            recent_files: vec![],
            closed_tabs: vec![],
            chrome_notice: None,
            last_unknown_banner: None,
            mod_runtime: ModRuntimeRegistry::default(),
            pending_save: None,
            ai_prefs: AiAssistantPrefsWeb::default(),
        }
    }

    pub fn snapshot(&mut self) -> TabContentDto {
        self.refresh_cm_context();
        let prefs = self.prefs.clone();
        build_snapshot(self, &prefs)
    }

    fn refresh_cm_context(&mut self) {
        let tab = self.tabs.active_tab();
        let host = tab
            .and_then(|t| t.document.as_ref())
            .map(|d| d.host_context())
            .unwrap_or_else(unibrowser_engine::ui_command::host_context_for_browse);
        let ctx = if let Some(t) = tab {
            infer_context(
                t.is_directory,
                matches!(&t.location, BrowserLocation::Http { .. }),
                t.document.as_ref().map(|d| d.path.as_str()),
                Some(host.mode),
            )
        } else {
            CommandContext::Start
        };
        self.command_manager.select_tab_for_context(ctx);
    }

    pub fn dispatch(&mut self, cmd: &str, args: &Value) -> Result<Value, String> {
        match cmd {
            "get_browser_state" => Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?),
            "new_tab" => {
                self.tabs.open_tab(BrowserLocation::Internal {
                    page: "home".into(),
                });
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "select_tab" => {
                let index = args
                    .get("index")
                    .and_then(|v| v.as_u64())
                    .ok_or("index required")? as usize;
                if index >= self.tabs.tab_count() {
                    return Err("invalid tab index".into());
                }
                self.tabs.set_active(index);
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "close_tab" => {
                let index = args
                    .get("index")
                    .and_then(|v| v.as_u64())
                    .ok_or("index required")? as usize;
                let _force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
                if self.tabs.tab_count() <= 1 {
                    return Err("Cannot close the last tab".into());
                }
                if let Some(tab) = self.tabs.tabs().get(index) {
                    self.closed_tabs.push(tab.location.clone());
                }
                if !self.tabs.close_tab(index) {
                    return Err("cannot close tab".into());
                }
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "undo_close_tab" => {
                let loc = self
                    .closed_tabs
                    .pop()
                    .ok_or("No recently closed tabs")?;
                self.tabs.open_tab(loc);
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "reorder_tabs" => {
                let from = args.get("fromIndex").and_then(|v| v.as_u64()).ok_or("fromIndex")? as usize;
                let to = args.get("toIndex").and_then(|v| v.as_u64()).ok_or("toIndex")? as usize;
                if !self.tabs.move_tab(from, to) {
                    return Err("reorder failed".into());
                }
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "toggle_tab_pin" => {
                let index = args.get("index").and_then(|v| v.as_u64()).ok_or("index")? as usize;
                let pinned = self.tabs.is_tab_pinned(index);
                if !self.tabs.set_tab_pinned(index, !pinned) {
                    return Err("invalid tab".into());
                }
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "open_path" => self.cmd_open_path(args),
            "open_text_with_content" => self.cmd_open_text_with_content(args),
            "navigate_omnibox" => {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .ok_or("input required")?;
                self.cmd_navigate(input)
            }
            "sync_text_document" => self.cmd_sync_text(args),
            "save_active_document" | "write_document_bytes" => self.cmd_save(args),
            "get_command_manager" => Ok(serde_json::to_value(self.build_cm_dto()).map_err(|e| e.to_string())?),
            "select_command_manager_tab" => {
                let tab_id = args
                    .get("tabId")
                    .and_then(|v| v.as_str())
                    .ok_or("tabId required")?;
                if self.command_manager.tabs.iter().any(|t| t.id == tab_id) {
                    self.command_manager.selected_tab = tab_id.to_string();
                }
                Ok(Value::Null)
            }
            "pin_command_manager_context" => {
                self.command_manager.pin_selected_tab();
                Ok(Value::Null)
            }
            "clear_command_manager_context_pin" => {
                self.command_manager.clear_context_pin();
                Ok(Value::Null)
            }
            "save_command_manager_layout" => Ok(Value::Null),
            "dismiss_cm_pin_tip" => Ok(self.prefs.to_preferences_dto()),
            "execute_command" => {
                let command_id = args
                    .get("commandId")
                    .or_else(|| args.get("command_id"))
                    .and_then(|v| v.as_str())
                    .ok_or("commandId required")?;
                self.cmd_execute(command_id)
            }
            "get_keybindings" => {
                let list: Vec<KeybindingDto> = self
                    .keybindings
                    .bindings_for_display()
                    .into_iter()
                    .map(|(command_id, chord)| KeybindingDto { command_id, chord })
                    .collect();
                Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
            }
            "palette_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                self.refresh_cm_context();
                let host = self.active_host_context();
                let scored = search_commands(&self.registry, &host, query);
                let mapped: Vec<PaletteEntryDto> = scored
                    .into_iter()
                    .map(|e| PaletteEntryDto {
                        command_id: e.command_id,
                        title: e.title,
                        score: e.score,
                        enabled: e.enablement.is_enabled(),
                        reason: e.enablement.disabled_reason().map(|s| s.to_string()),
                    })
                    .collect();
                Ok(serde_json::to_value(mapped).map_err(|e| e.to_string())?)
            }
            "get_preferences" => Ok(self.prefs.to_preferences_dto()),
            "set_ui_theme" => {
                let theme = args.get("theme").and_then(|v| v.as_str()).unwrap_or("system");
                self.prefs.theme = theme.into();
                Ok(self.prefs.to_preferences_dto())
            }
            "set_editor_theme" => {
                let theme_id = args
                    .get("themeId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auto");
                self.prefs.editor_theme_id = theme_id.into();
                Ok(self.prefs.to_preferences_dto())
            }
            "set_download_preference"
            | "set_restore_session_on_launch"
            | "set_board_snap_to_grid"
            | "set_midi_scrub_audio_enabled"
            | "set_fluidsynth_soundfont_path"
            | "set_crash_recovery_enabled"
            | "set_offline_mode"
            | "set_warn_insecure_http"
            | "set_bookmark_bar_enabled"
            | "set_ui_locale" => Ok(self.prefs.to_preferences_dto()),
            "dismiss_onboarding" => {
                self.prefs.onboarding_dismissed = true;
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "log_shell_ui_event" | "record_diagnostic_error" | "save_window_geometry"
            | "set_content_bounds" => Ok(Value::Null),
            "get_home_dir" => Ok(Value::String(String::new())),
            "go_back" | "go_forward" | "reload_tab" => {
                Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
            }
            "set_files_view_prefs" => Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?),
            "search_open_tabs" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let hits = self.search_tabs(query);
                Ok(serde_json::to_value(hits).map_err(|e| e.to_string())?)
            }
            "reload_keybindings" => Ok(Value::Null),
            "get_autosave_status" => {
                Ok(serde_json::json!({ "last_autosave_at_ms": 0 }))
            }
            "get_self_test_plan" | "take_doc_capture_path" | "get_demo_state" => Ok(Value::Null),
            "take_launch_open_paths" => Ok(serde_json::json!([])),
            "resolve_capture_path" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or("path required")?;
                Ok(Value::String(path.to_string()))
            }
            "get_ai_assistant_prefs" => {
                Ok(serde_json::to_value(&self.ai_prefs).map_err(|e| e.to_string())?)
            }
            "set_ai_assistant_prefs" => {
                self.ai_prefs.merge_from_args(args);
                Ok(Value::Null)
            }
            "ai_assistant_status" => {
                Ok(serde_json::to_value(self.ai_prefs.status()).map_err(|e| e.to_string())?)
            }
            "list_editor_themes_cmd" => Ok(serde_json::json!([
                {"id": "vs", "label": "Light", "source": "builtin"},
                {"id": "vs-dark", "label": "Dark", "source": "builtin"},
                {"id": "auto", "label": "Match app theme", "source": "builtin"}
            ])),
            "list_monaco_grammars_cmd" => Ok(serde_json::json!([])),
            "get_editor_monaco_theme" => Ok(Value::Null),
            "releases_page_url" => Ok(Value::String(
                "https://github.com/browforge/unibrowser/releases".into(),
            )),
            "desktop_download_page_url" => Ok(Value::String(
                "https://github.com/browforge/unibrowser/releases/latest".into(),
            )),
            other => Ok(serde_json::json!({
                "web_stub": true,
                "command": other,
                "reason": WEB_UNAVAILABLE
            })),
        }
    }

    fn cmd_open_path(&mut self, args: &Value) -> Result<Value, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("path required")?;
        let content = args.get("content").and_then(|v| v.as_str());
        if let Some(text) = content {
            return self.open_text_path(path, text);
        }
        if path.starts_with("demo-pack/") {
            return Err(
                "Demo pack files are opened from bundled assets in the web demo (use Open file or the demo pack links)"
                    .into(),
            );
        }
        if path.starts_with("unibrowser://") {
            return self.cmd_navigate(path);
        }
        Err("Open local files via the file picker in the web demo".into())
    }

    fn cmd_open_text_with_content(&mut self, args: &Value) -> Result<Value, String> {
        let path = args.get("path").and_then(|v| v.as_str()).ok_or("path")?;
        let content = args.get("content").and_then(|v| v.as_str()).ok_or("content")?;
        self.open_text_path(path, content)
    }

    fn open_text_path(&mut self, path: &str, content: &str) -> Result<Value, String> {
        if !is_text_path(path) {
            self.last_unknown_banner = Some(serde_json::json!({
                "path": path,
                "message": "This format needs the desktop app in v1 web demo"
            }));
            let loc = BrowserLocation::File {
                path: path.into(),
            };
            navigate_tabs(&mut self.tabs, loc, None);
            return Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?);
        }
        self.last_unknown_banner = None;
        let doc = open_text_backend(path.into(), content.into());
        let loc = BrowserLocation::File {
            path: path.into(),
        };
        navigate_tabs(&mut self.tabs, loc, Some(doc));
        self.touch_recent(path);
        Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
    }

    fn cmd_navigate(&mut self, input: &str) -> Result<Value, String> {
        let trimmed = input.trim();
        if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
            self.chrome_notice = Some(
                "Live web tabs need the desktop app. Try the demo pack or open a local file.".into(),
            );
            return Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?);
        }
        if trimmed == "unibrowser://settings" || trimmed == "settings" {
            let loc = BrowserLocation::Internal {
                page: "settings".into(),
            };
            navigate_tabs(&mut self.tabs, loc, None);
            return Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?);
        }
        if trimmed == "unibrowser://home" || trimmed.is_empty() {
            let loc = BrowserLocation::Internal {
                page: "home".into(),
            };
            navigate_tabs(&mut self.tabs, loc, None);
            return Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?);
        }
        if trimmed.starts_with("demo-pack/") {
            self.chrome_notice = Some(
                "Loading demo files from the bundled pack. Pick a file if fetch fails.".into(),
            );
            let loc = BrowserLocation::File {
                path: trimmed.into(),
            };
            navigate_tabs(&mut self.tabs, loc, None);
            return Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?);
        }
        if is_text_path(trimmed) {
            return Err(
                "Text file content is not bundled at this path; use Open file or a demo-pack link"
                    .into(),
            );
        }
        self.chrome_notice = Some(format!("Cannot open `{}` in the web demo yet.", trimmed));
        Ok(serde_json::to_value(self.snapshot()).map_err(|e| e.to_string())?)
    }

    fn cmd_sync_text(&mut self, args: &Value) -> Result<Value, String> {
        let content = args.get("content").and_then(|v| v.as_str()).ok_or("content")?;
        let tab = self.tabs.active_tab_mut().ok_or("no active tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| base_text::try_text_backend(&mut *doc.backend).map(|b| b.path.clone()))
            .ok_or("path")?;
        if doc.path != path {
            return Err("path mismatch".into());
        }
        let font = args
            .get("fontSizePt")
            .and_then(|v| v.as_f64())
            .unwrap_or(14.0) as f32;
        if let Some(backend) = base_text::try_text_backend(&mut *doc.backend) {
            backend.set_content(content.into());
            backend.font_size_pt = font;
            let sel_start = args
                .get("selectionStart")
                .or_else(|| args.get("selection_start"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            let sel_end = args
                .get("selectionEnd")
                .or_else(|| args.get("selection_end"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            if let (Some(start), Some(end)) = (sel_start, sel_end) {
                backend.set_selection_range(start, end);
            }
        }
        Ok(Value::Null)
    }

    fn cmd_save(&mut self, args: &Value) -> Result<Value, String> {
        let tab = self.tabs.active_tab_mut().ok_or("no active tab")?;
        let doc = tab.document.as_mut().ok_or("no document")?;
        if let Some(backend) = base_text::try_text_backend(&mut *doc.backend) {
            let path = backend.path.clone();
            let content = backend.content.clone();
            backend.dirty = false;
            self.pending_save = Some((path.clone(), content.clone()));
            if args.get("path").is_some() {
                return Ok(Value::Null);
            }
            return Ok(serde_json::json!({ "path": path, "content": content }));
        }
        Err("Nothing to save".into())
    }

    fn cmd_execute(&mut self, command_id: &str) -> Result<Value, String> {
        if let Some(reason) = web_disable_reason(command_id) {
            return Err(reason.into());
        }
        let tab = self.tabs.active_tab_mut().ok_or("no active tab")?;
        let doc = tab.document.as_mut().ok_or("no document open")?;
        let msg = self
            .registry
            .execute(command_id, doc)
            .map_err(|e| e.to_string())?;
        Ok(Value::String(msg))
    }

    fn touch_recent(&mut self, path: &str) {
        let label = path.rsplit('/').next().unwrap_or(path).into();
        self.recent_files.retain(|r| r.path != path);
        self.recent_files.insert(
            0,
            RecentFile {
                path: path.into(),
                label,
                pinned: false,
                opened_at_ms: 0,
            },
        );
        if self.recent_files.len() > 12 {
            self.recent_files.truncate(12);
        }
    }

    fn active_host_context(&self) -> HostContext {
        self.tabs
            .active_tab()
            .and_then(|t| t.document.as_ref())
            .map(|d| d.host_context())
            .unwrap_or_else(unibrowser_engine::ui_command::host_context_for_browse)
    }

    fn search_tabs(&self, query: &str) -> Vec<TabSearchHit> {
        let q = query.to_ascii_lowercase();
        self.tabs
            .tabs()
            .iter()
            .enumerate()
            .filter_map(|(index, tab)| {
                let label = crate::snapshot::tab_label(tab);
                let omnibox = crate::snapshot::location_to_omnibox(&tab.location);
                if q.is_empty() || label.to_ascii_lowercase().contains(&q) || omnibox.contains(&q) {
                    Some(TabSearchHit {
                        window_label: "main".into(),
                        tab_index: index,
                        label,
                        omnibox,
                        score: 100,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn build_cm_dto(&mut self) -> CommandManagerDto {
        self.refresh_cm_context();
        let host = self.active_host_context();
        let doc_ref = self.tabs.active_tab().and_then(|t| t.document.as_ref());
        let contributions: Vec<LoadedModContribution> = vec![];
        let selected = self.command_manager.selected_tab.clone();
        let auto_context = self.command_manager.auto_context.tab_id().to_string();
        let tabs: Vec<ManagerTabDto> = self
            .command_manager
            .tabs
            .iter()
            .map(|tab| {
                let families = self.command_manager.resolve_families_for_tab(
                    &self.registry,
                    &host,
                    doc_ref,
                    &tab.id,
                    doc_ref.map(|d| d.path.as_str()),
                    Some(&contributions),
                    Some(&self.mod_runtime),
                );
                let compact_toolbar_families = self.command_manager.resolve_compact_toolbar_families(
                    &self.registry,
                    &host,
                    doc_ref,
                    &tab.id,
                    doc_ref.map(|d| d.path.as_str()),
                    Some(&contributions),
                    Some(&self.mod_runtime),
                );
                let resolved_family_count = families.len();
                let resolved_control_count = families
                    .iter()
                    .flat_map(|f| &f.groups)
                    .map(|g| g.controls.len())
                    .sum();
                let resolved_cmds = if families.is_empty() {
                    self.command_manager
                        .resolve_tab_commands(&self.registry, &host, doc_ref, &tab.id)
                } else {
                    CommandManagerState::flatten_resolved_controls(&families)
                };
                let commands: Vec<UiCommandDto> = resolved_cmds
                    .into_iter()
                    .map(|c| map_ui_command(&c.command_id, c.title, c.icon, c.enabled, c.reason))
                    .collect();
                ManagerTabDto {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                    phase: format!("{:?}", tab.phase).to_lowercase(),
                    selected: tab.id == selected,
                    commands,
                    families,
                    compact_toolbar_families,
                    resolved_family_count,
                    resolved_control_count,
                }
            })
            .collect();
        let path = doc_ref.map(|d| d.path.as_str());
        CommandManagerDto {
            auto_context,
            selected_tab: selected,
            context_display_label: unibrowser_engine::command_context_display_label(
                self.command_manager.auto_context,
                host.mode,
                path,
            )
            .into(),
            context_pinned: self.command_manager.context_pin.is_some(),
            tabs,
            floating_bars: vec![],
            diagnostics: vec!["Web demo: native mods and sidecars are unavailable.".into()],
            shell_contributions: ShellContributionsDto::default(),
        }
    }
}

fn map_ui_command(
    command_id: &str,
    title: String,
    icon: String,
    enabled: bool,
    reason: Option<String>,
) -> UiCommandDto {
    if let Some(web_reason) = web_disable_reason(command_id) {
        return UiCommandDto {
            command_id: command_id.into(),
            title,
            icon,
            enabled: false,
            reason: Some(web_reason.into()),
        };
    }
    UiCommandDto {
        command_id: command_id.into(),
        title,
        icon,
        enabled,
        reason,
    }
}

fn web_disable_reason(command_id: &str) -> Option<&'static str> {
    const PREFIXES: &[(&str, &str)] = &[
        ("unibrowser.terminal.", "Terminal requires the desktop app"),
        ("unibrowser.lsp.", "Language server requires the desktop app"),
        ("vsix.", "VSIX extensions require the desktop app"),
        ("unibrowser.capture.", "Tab capture requires the desktop app"),
        ("unibrowser.download.", "Download manager requires the desktop app"),
    ];
    for (prefix, reason) in PREFIXES {
        if command_id.starts_with(prefix) {
            return Some(reason);
        }
    }
    match command_id {
        "unibrowser.openModPortal" | "unibrowser.mods.install" => {
            Some("Mod portal installs require the desktop app")
        }
        "unibrowser.newWindow" => Some("Multiple windows require the desktop app"),
        _ => None,
    }
}

#[derive(Serialize)]
struct PaletteEntryDto {
    command_id: String,
    title: String,
    score: i32,
    enabled: bool,
    reason: Option<String>,
}

#[derive(Serialize)]
struct KeybindingDto {
    command_id: String,
    chord: String,
}

#[derive(Serialize)]
struct TabSearchHit {
    window_label: String,
    tab_index: usize,
    label: String,
    omnibox: String,
    score: i32,
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
    pub compact_toolbar_families: Vec<unibrowser_mod_api::ResolvedFamily>,
    pub resolved_family_count: usize,
    pub resolved_control_count: usize,
}

#[derive(Serialize, Default)]
pub struct ShellContributionsDto {
    pub panels: Vec<Value>,
    pub views: Vec<Value>,
    pub status_items: Vec<Value>,
    pub services: Vec<Value>,
}

#[derive(Serialize)]
pub struct CommandManagerDto {
    pub auto_context: String,
    pub selected_tab: String,
    pub context_display_label: String,
    pub context_pinned: bool,
    pub tabs: Vec<ManagerTabDto>,
    pub floating_bars: Vec<Value>,
    pub diagnostics: Vec<String>,
    pub shell_contributions: ShellContributionsDto,
}
