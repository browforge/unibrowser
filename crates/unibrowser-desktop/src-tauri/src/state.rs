use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};

use crate::listing_control::ListingControl;

use base_files::list_directory;
use base_web::{WebBackend, handles as web_handles};
use unibrowser_engine::{
    CommandManagerState, CommandRegistry, DocumentSession, KeySequenceMatcher, KeybindingMap,
    LoadedModContribution, ModRuntimeRegistry, SurfaceProviderRegistry, TabManager, infer_context,
    load_command_manager_file, merge_toolbar_layouts, navigation::parse_user_input,
    build_surface_provider_registry, ContributionMergeDiagnostics,
};

use crate::config::{keybindings_file_path, toolbars_file_path};
use crate::window_session::{SharedWindowSession, WindowSession};
use unibrowser_mod_api::BrowserLocation;
use unibrowser_mod_host::{ModHost, ModRegisterFn, default_mods_dir};

#[derive(Default)]
pub struct SplitViewState {
    pub enabled: bool,
    pub secondary_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FileCompareState {
    pub left_path: String,
    pub right_path: String,
}

#[derive(Debug, Clone)]
pub struct ImageCompareState {
    pub left_path: String,
    pub right_path: String,
}

pub struct AppState {
    pub registry: Mutex<CommandRegistry>,
    pub sessions: Mutex<HashMap<String, SharedWindowSession>>,
    pub cwd: PathBuf,
    mods_loaded: RwLock<String>,
    #[cfg(feature = "dynamic-mods")]
    dynamic_cdylibs: Mutex<Vec<unibrowser_mod_host::dynamic::LoadedCdylibMod>>,
    pub keybindings: Mutex<KeybindingMap>,
    pub command_manager: Mutex<CommandManagerState>,
    pub vsix_host_commands: Mutex<HashSet<String>>,
    pub key_sequences: Mutex<KeySequenceMatcher>,
    pub listing_control: ListingControl,
    pub filename_search_cancel: AtomicBool,
    pub _mod_host: ModHost,
    pub mod_runtime: Mutex<ModRuntimeRegistry>,
    pub surface_providers: RwLock<SurfaceProviderRegistry>,
}

impl AppState {
    pub fn bundled_mod_count(&self) -> usize {
        self._mod_host.mods.len()
    }

    pub fn registry_command_count(&self) -> usize {
        self.registry
            .lock()
            .map(|r| r.sorted_command_ids().len())
            .unwrap_or(0)
    }

    pub fn session(&self, label: &str) -> Result<SharedWindowSession, String> {
        let mut map = self.sessions.lock().map_err(|e| e.to_string())?;
        Ok(map
            .entry(label.to_string())
            .or_insert_with(|| SharedWindowSession::new(WindowSession::new_with_home(&self.cwd)))
            .clone())
    }
}

fn bundled_mod_registers() -> HashMap<&'static str, ModRegisterFn> {
    [
        ("base-files", base_files::register as ModRegisterFn),
        ("base-web", base_web::register as ModRegisterFn),
        ("base-text", base_text::register as ModRegisterFn),
        ("base-png", base_png::register as ModRegisterFn),
        ("base-svg", base_svg::register as ModRegisterFn),
        ("base-docx", base_docx::register as ModRegisterFn),
        ("base-pdf", base_pdf::register as ModRegisterFn),
        ("base-dxf", base_dxf::register as ModRegisterFn),
        ("base-js", base_js::register as ModRegisterFn),
        ("base-video", base_video::register as ModRegisterFn),
        ("base-audio", base_audio::register as ModRegisterFn),
        ("toy-markdown", toy_markdown::register as ModRegisterFn),
        ("base-hex", base_hex::register as ModRegisterFn),
        ("base-epub", base_epub::register as ModRegisterFn),
        ("base-csv", base_csv::register as ModRegisterFn),
        ("spell-en-nz", spell_en_nz::register as ModRegisterFn),
        ("grammar-harper", grammar_harper::register as ModRegisterFn),
        ("dxf-checker", dxf_checker::register as ModRegisterFn),
        ("unibrow-checker", unibrow_checker::register as ModRegisterFn),
        ("live-proving", live_proving::register as ModRegisterFn),
        ("ai-assistant", ai_assistant::register as ModRegisterFn),
        ("cmd-proving", cmd_proving::register as ModRegisterFn),
        (
            "text-toolbar-sample",
            text_toolbar_sample::register as ModRegisterFn,
        ),
        (
            "image-grayscale-preview",
            image_grayscale_preview::register as ModRegisterFn,
        ),
    ]
    .into_iter()
    .collect()
}

impl AppState {
    pub fn mod_contributions(&self) -> Vec<LoadedModContribution> {
        permitted_mod_contributions(&self._mod_host)
    }

    fn merge_mod_floating_toolbars(cm: &mut CommandManagerState, contributions: &[LoadedModContribution]) {
        let mut diag = ContributionMergeDiagnostics::default();
        let bars = merge_toolbar_layouts(contributions, &mut diag);
        for bar in bars {
            if cm.floating_bars.iter().any(|existing| existing.id == bar.id) {
                continue;
            }
            cm.floating_bars.push(bar);
        }
    }
}

fn permitted_mod_contributions(host: &ModHost) -> Vec<LoadedModContribution> {
    host.mods
        .iter()
            .filter_map(|loaded| {
                loaded.manifest.contributes.clone().map(|mut contributes| {
                    let grants = &loaded.manifest.capabilities;
                    if !grants.iter().any(|g| g == "ui.command_manager.contribute") {
                        contributes.controls.clear();
                        contributes.groups.clear();
                        contributes.families.clear();
                        contributes.tabs.clear();
                    }
                    if !grants.iter().any(|g| g == "commands.capability.provide") {
                        contributes.capability_providers.clear();
                    }
                    if !grants.iter().any(|g| g == "ui.toolbar.contribute") {
                        contributes.toolbars.clear();
                    } else if !grants.iter().any(|g| g == "ui.toolbar.detach") {
                        for toolbar in &mut contributes.toolbars {
                            toolbar.detachable = false;
                        }
                    }
                    if !grants.iter().any(|g| g == "ui.panel.contribute") {
                        contributes.panels.clear();
                        contributes.views.clear();
                    }
                    if !grants.iter().any(|g| g == "ui.status.contribute") {
                        contributes.status_items.clear();
                    }
                    if !grants.iter().any(|g| g == "service.register") {
                        contributes.services.clear();
                    }
                    LoadedModContribution {
                        mod_name: loaded.manifest.name.clone(),
                        contributes,
                    }
                })
            })
        .collect()
}

pub fn vsix_keybinding_pairs() -> Vec<(String, String)> {
    let root = crate::config::vsix_extensions_dir();
    let root = match root {
        Some(r) => r,
        None => return Vec::new(),
    };
    let installed = unibrowser_vsix::list_installed(&root).unwrap_or_default();
    unibrowser_vsix::enabled_keybindings(&installed)
}

pub fn load_merged_keybindings() -> KeybindingMap {
    let vsix = vsix_keybinding_pairs();
    KeybindingMap::load_merged(keybindings_file_path().as_deref(), &vsix)
}

fn append_mod_name(line: &mut String, name: &str) {
    if !line.is_empty() {
        line.push_str(", ");
    }
    line.push_str(name);
}

struct BuiltRegistry {
    registry: CommandRegistry,
    mod_host: ModHost,
    mod_runtime: ModRuntimeRegistry,
    mods_line: String,
    #[cfg(feature = "dynamic-mods")]
    dynamic_libs: Vec<unibrowser_mod_host::dynamic::LoadedCdylibMod>,
}

fn build_command_registry(
    vsix_host_commands: &Mutex<HashSet<String>>,
) -> Result<BuiltRegistry, String> {
    let mut registry = CommandRegistry::with_builtins();
    let bundled = bundled_mod_registers();
    let mod_host = ModHost::discover_and_load(&[default_mods_dir()], &mut registry, &bundled)
        .map_err(|e| e.to_string())?;
    if let Some(vsix_root) = crate::config::vsix_extensions_dir() {
        unibrowser_vsix::register_enabled_contributions(
            &mut registry,
            &vsix_root,
            vsix_host_commands,
        );
    }
    crate::midi_stub::register_handlers(&mut registry);
    crate::project_undo_handlers::register_handlers(&mut registry);
    let mut mods_line = mod_host.load_order.join(", ");
    if let Some(name) = unibrowser_mod_host::wasm::register_proving_hello_stub(&mut registry) {
        append_mod_name(&mut mods_line, &name);
    }
    #[cfg(feature = "dynamic-mods")]
    let mut dynamic_libs = Vec::new();
    if !crate::preferences::user_mods_disabled() {
        if let Some(root) = crate::config::user_mods_dir() {
            if let Ok(wasm_names) =
                unibrowser_mod_host::wasm::load_user_wasm_mods(&root, &mut registry)
            {
                for name in wasm_names {
                    append_mod_name(&mut mods_line, &name);
                }
            }
        }
    }
    #[cfg(all(feature = "dynamic-mods", target_os = "linux"))]
    {
        if !crate::preferences::user_mods_disabled() {
            if let Some(root) = crate::config::user_mods_dir() {
                if let Ok(user_libs) =
                    unibrowser_mod_host::dynamic::load_user_native_mods(&root, &mut registry)
                {
                    for m in user_libs {
                        append_mod_name(&mut mods_line, &m.name);
                        dynamic_libs.push(m);
                    }
                }
            }
        }
        if !crate::preferences::user_mods_disabled()
            && std::env::var("UNIBROWSER_LOAD_CDYLIB_MOD").as_deref() == Ok("1")
        {
            use unibrowser_mod_host::dynamic::{default_cdylib_search_paths, load_cdylib_mod};
            for path in default_cdylib_search_paths() {
                if let Ok(m) = load_cdylib_mod(None, &path, &mut registry) {
                    append_mod_name(&mut mods_line, &m.name);
                    dynamic_libs.push(m);
                    break;
                }
            }
        }
    }
    let mut mod_runtime = ModRuntimeRegistry::default();
    cmd_proving::register_runtime(&mut mod_runtime);
    image_grayscale_preview::register_runtime(&mut mod_runtime);

    Ok(BuiltRegistry {
        registry,
        mod_host,
        mod_runtime,
        mods_line,
        #[cfg(feature = "dynamic-mods")]
        dynamic_libs,
    })
}

impl AppState {
    pub fn mods_loaded_line(&self) -> String {
        self.mods_loaded
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn new() -> Self {
        let vsix_host_commands = Mutex::new(HashSet::new());
        let built = build_command_registry(&vsix_host_commands).expect("mods");
        let cwd = std::env::var("UNIBROWSER_REPO_ROOT")
            .map(|r| PathBuf::from(r.trim_end_matches('/')))
            .or_else(|_| std::env::current_dir())
            .or_else(|_| std::env::var("HOME").map(PathBuf::from))
            .unwrap_or_else(|_| PathBuf::from("/"));
        let home = BrowserLocation::File {
            path: cwd.clone(),
        };
        let main_session = SharedWindowSession::new(WindowSession::new_with_home(&cwd));
        let sessions = Mutex::new(HashMap::from([("main".to_string(), main_session)]));
        let keybindings = Mutex::new(load_merged_keybindings());
        let cm_file = load_command_manager_file(toolbars_file_path().as_deref());
        let mut cm_state = CommandManagerState::with_layout_path(
            &cm_file,
            unibrowser_mod_api::CommandContext::Start,
            toolbars_file_path().as_deref(),
        );
        AppState::merge_mod_floating_toolbars(
            &mut cm_state,
            &permitted_mod_contributions(&built.mod_host),
        );
        if std::env::var("UNIBROWSER_DEMO_STATE")
            .unwrap_or_default()
            .split([',', ';'])
            .any(|token| token.trim().eq_ignore_ascii_case("cmd-proving-detached"))
        {
            if let Some(bar) = cm_state
                .floating_bars
                .iter_mut()
                .find(|bar| bar.id == "cmd-proving.detached-markdown")
            {
                bar.presentation = unibrowser_engine::ToolbarPresentation::DetachedWindow;
                bar.anchor = unibrowser_engine::ToolbarAnchor::Floating;
                bar.coord_space = unibrowser_engine::ToolbarCoordSpace::Screen;
                bar.x = 1050.0;
                bar.y = 120.0;
            }
        }
        let command_manager = Mutex::new(cm_state);

        let surface_providers = RwLock::new(surface_registry_for_host(&built.mod_host));

        Self {
            registry: Mutex::new(built.registry),
            sessions,
            cwd,
            mods_loaded: RwLock::new(built.mods_line),
            keybindings,
            command_manager,
            vsix_host_commands,
            key_sequences: Mutex::new(KeySequenceMatcher::default()),
            listing_control: ListingControl::new(),
            filename_search_cancel: AtomicBool::new(false),
            _mod_host: built.mod_host,
            mod_runtime: Mutex::new(built.mod_runtime),
            surface_providers,
            #[cfg(feature = "dynamic-mods")]
            dynamic_cdylibs: Mutex::new(built.dynamic_libs),
        }
    }

    pub fn surface_registry(&self) -> SurfaceProviderRegistry {
        self.surface_providers
            .read()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    pub fn refresh_extension_catalog(&self) -> Result<(), String> {
        let built = build_command_registry(&self.vsix_host_commands)?;
        if let Ok(mut guard) = self.registry.lock() {
            *guard = built.registry;
        }
        if let Ok(mut runtime) = self.mod_runtime.lock() {
            *runtime = built.mod_runtime;
        }
        if let Ok(mut line) = self.mods_loaded.write() {
            *line = built.mods_line;
        }
        #[cfg(feature = "dynamic-mods")]
        if let Ok(mut libs) = self.dynamic_cdylibs.lock() {
            *libs = built.dynamic_libs;
        }
        if let Ok(mut kb) = self.keybindings.lock() {
            *kb = load_merged_keybindings();
        }
        if let Ok(mut reg) = self.surface_providers.write() {
            *reg = surface_registry_for_host(&built.mod_host);
        }
        Ok(())
    }

    pub fn refresh_vsix_catalog(&self) {
        let _ = self.refresh_extension_catalog();
    }

    pub fn refresh_command_manager_context(&self, window_label: &str) {
        let sess = self.session(window_label).expect("session");
        let tabs = sess.tabs.lock().expect("tabs lock");
        let tab = tabs.active_tab();
        let ctx = if let Some(tab) = tab {
            if matches!(
                &tab.location,
                BrowserLocation::Internal { page } if page == "home"
            ) {
                unibrowser_mod_api::CommandContext::Start
            } else {
                let path = tab
                    .document
                    .as_ref()
                    .map(|d| d.path.clone())
                    .or_else(|| {
                        if let BrowserLocation::File { path } = &tab.location {
                            Some(path.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    });
                let mode = tab.document.as_ref().map(|d| d.backend.mode());
                infer_context(
                    tab.is_directory,
                    matches!(tab.location, BrowserLocation::Http { .. }),
                    path.as_deref(),
                    mode,
                )
            }
        } else {
            unibrowser_mod_api::CommandContext::Start
        };
        let mut cm = self.command_manager.lock().expect("command manager lock");
        cm.bump_context_revision();
        cm.select_tab_for_context(ctx);
    }

    pub fn resolve_location(
        &self,
        location: BrowserLocation,
    ) -> (BrowserLocation, bool, Option<DocumentSession>) {
        if let BrowserLocation::Http { url } = &location {
            if web_handles(&location) {
                let mut backend = WebBackend::new(url.clone());
                backend.reader_mode = crate::web_reader_prefs::get_origin_reader_mode(url);
                let doc = DocumentSession::new(url.clone(), Box::new(backend));
                return (location, false, Some(doc));
            }
        }
        if let BrowserLocation::File { path } = &location {
            if path.is_dir() {
                return (location, true, None);
            }
            if path.is_file() {
                let doc = self.open_file_document(path);
                return (location, false, doc);
            }
        }
        if let BrowserLocation::Internal { page } = &location {
            if page == "home" {
                return (location, false, None);
            }
            if page.starts_with("files") {
                let path = internal_files_path(page, &self.cwd);
                return (
                    BrowserLocation::File { path },
                    true,
                    None,
                );
            }
            if page.starts_with("mods") {
                return (location, false, None);
            }
            if page == "settings" {
                let doc = DocumentSession::new(
                    crate::settings_stub::LOCATION.to_string(),
                    crate::settings_stub::open(),
                );
                return (location, false, Some(doc));
            }
            if page == "about" {
                let doc = DocumentSession::new(
                    crate::about_stub::LOCATION.to_string(),
                    crate::about_stub::open(),
                );
                return (location, false, Some(doc));
            }
            if page == "keys" {
                let doc = DocumentSession::new(
                    crate::keys_stub::LOCATION.to_string(),
                    crate::keys_stub::open(),
                );
                return (location, false, Some(doc));
            }
            if page == "downloads" {
                let doc = DocumentSession::new(
                    crate::downloads_stub::LOCATION.to_string(),
                    crate::downloads_stub::open(),
                );
                return (location, false, Some(doc));
            }
            if page == "diagnostics" {
                let doc = DocumentSession::new(
                    crate::diagnostics_stub::LOCATION.to_string(),
                    crate::diagnostics_stub::open(),
                );
                return (location, false, Some(doc));
            }
            if page.starts_with("docs/") {
                let slug = page
                    .strip_prefix("docs/")
                    .unwrap_or("getting-started")
                    .split('/')
                    .next()
                    .unwrap_or("getting-started");
                let loc = crate::docs_stub::location_for_slug(slug);
                let doc = DocumentSession::new(loc, crate::docs_stub::open(slug));
                return (location, false, Some(doc));
            }
        }
        (location, false, None)
    }

    pub fn navigate_input(&self, input: &str, window_label: &str) -> Result<BrowserLocation, String> {
        let loc = if input.trim().starts_with("file://") {
            let path = crate::path_security::resolve_file_url_input(input.trim(), &self.cwd)?;
            BrowserLocation::File { path }
        } else {
            parse_user_input(input, &self.cwd)
        };
        let (loc, is_dir, doc) = self.resolve_location(loc);
        let loc = match loc {
            BrowserLocation::File { path } => {
                let resolved = crate::path_security::resolve_open_path(
                    &path.to_string_lossy(),
                    &self.cwd,
                )?;
                BrowserLocation::File { path: resolved }
            }
            other => other,
        };
        let sess = self.session(window_label)?;
        let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
        tabs.navigate_active(loc.clone(), is_dir, doc);
        Ok(loc)
    }
}

fn internal_files_path(page: &str, cwd: &Path) -> PathBuf {
    if let Some(rest) = page.strip_prefix("files?path=") {
        return PathBuf::from(rest);
    }
    if page == "files" {
        return cwd.to_path_buf();
    }
    cwd.to_path_buf()
}

pub fn try_open_unibrow_document(path: &Path) -> Result<DocumentSession, String> {
    use crate::project_backend::UnibrowBackend;
    let path_str = path.to_string_lossy().to_string();
    let cache = crate::config::unibrow_zip_cache_dir().ok_or("config path unavailable")?;
    let root =
        unibrowser_project::resolve_package_root(path, &cache).map_err(|e| {
            unibrowser_project::friendly_open_error(&e)
        })?;
    let pkg = unibrowser_project::open_package(&root).map_err(|e| {
        unibrowser_project::friendly_open_error(&e)
    })?;
    Ok(DocumentSession::new(
        path_str,
        Box::new(UnibrowBackend::open(pkg)),
    ))
}

pub enum FileOpenMode {
    Auto,
    PlainText,
    Hex,
}

fn surface_registry_for_host(host: &ModHost) -> SurfaceProviderRegistry {
    let manifests: Vec<_> = host.mods.iter().map(|m| m.manifest.clone()).collect();
    build_surface_provider_registry(&manifests)
}

impl AppState {
    pub fn open_file_document_with_mode(
        &self,
        path: &Path,
        mode: FileOpenMode,
    ) -> Option<DocumentSession> {
        let path_str = path.to_string_lossy().to_string();
        let sniffed = crate::content_sniff::sniff_format(path);
        if unibrowser_project::is_unibrow_path(path) {
            if let Ok(doc) = try_open_unibrow_document(path) {
                return Some(doc);
            }
            return None;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let effective = effective_extension(sniffed, &ext);
        let prefs = crate::preferences::load_preferences();
        let registry = self.surface_registry();
        let backend = crate::capability_router::open_backend_for_path(
            &registry,
            &prefs,
            path,
            &effective,
            mode,
        )?;
        apply_docx_font_pref(path, backend)
    }

    pub fn open_file_document(&self, path: &Path) -> Option<DocumentSession> {
        self.open_file_document_with_mode(path, FileOpenMode::Auto)
    }
}

pub fn open_file_document_with_mode(path: &Path, mode: FileOpenMode) -> Option<DocumentSession> {
    // Legacy entry for unit tests: uses bundled registry from a throwaway AppState.
    AppState::new().open_file_document_with_mode(path, mode)
}

pub fn open_file_document(path: &Path) -> Option<DocumentSession> {
    AppState::new().open_file_document(path)
}

pub fn effective_extension_for_path(path: &Path, ext: &str) -> String {
    let sniffed = crate::content_sniff::sniff_format(path);
    effective_extension(sniffed, ext).to_string()
}

fn effective_extension(
    sniffed: Option<crate::content_sniff::SniffedFormat>,
    ext: &str,
) -> &str {
    match sniffed {
        Some(crate::content_sniff::SniffedFormat::Pdf) => "pdf",
        Some(crate::content_sniff::SniffedFormat::Png) => "png",
        Some(crate::content_sniff::SniffedFormat::Jpeg) => "jpeg",
        Some(crate::content_sniff::SniffedFormat::Gif) => "gif",
        Some(crate::content_sniff::SniffedFormat::Webp) => "webp",
        Some(crate::content_sniff::SniffedFormat::Wav) => "wav",
        Some(crate::content_sniff::SniffedFormat::Mp3) => "mp3",
        Some(crate::content_sniff::SniffedFormat::RiffAvi) => "avi",
        Some(crate::content_sniff::SniffedFormat::Ogg) => "ogg",
        Some(crate::content_sniff::SniffedFormat::Flac) => "flac",
        Some(crate::content_sniff::SniffedFormat::Epub) => "epub",
        Some(crate::content_sniff::SniffedFormat::Zip) => ext,
        None => ext,
    }
}

fn apply_docx_font_pref(
    path: &Path,
    backend: Box<dyn unibrowser_engine::document::DocumentBackend>,
) -> Option<DocumentSession> {
    let path_str = path.to_string_lossy().to_string();
    let mut backend = backend;
    if backend.backend_id() == base_docx::BACKEND_ID {
        let path_key = path.to_string_lossy();
        if let Some(pt) = crate::preferences::docx_reader_font(path_key.as_ref()) {
            if let Some(b) = base_docx::try_docx_backend(&mut *backend) {
                b.reader_font_pt = pt;
            }
        }
    }
    Some(DocumentSession::new(path_str, backend))
}

/// Extensions opened as plain text when sniffing does not override.
pub fn is_known_text_extension(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "markdown"
            | "json"
            | "rs"
            | "toml"
            | "yaml"
            | "yml"
            | "html"
            | "htm"
            | "css"
            | "scss"
            | "xml"
            | "sh"
            | "bash"
            | "zsh"
            | "log"
            | "ini"
            | "cfg"
            | "conf"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "sql"
            | "lua"
            | "rhai"
    )
}

pub fn list_dir(path: &Path) -> Result<Vec<base_files::DirEntry>, String> {
    let prefs = crate::preferences::load_preferences();
    let entries = list_directory(path)?;
    Ok(crate::files_listing::apply_files_view(entries, &prefs.files_view))
}

#[cfg(test)]
mod cmd_proving_tests {
    use super::*;

    #[test]
    fn png_image_tab_resolves_grayscale_control() {
        let built = build_command_registry(&Mutex::new(HashSet::new())).expect("registry");
        let contributions = permitted_mod_contributions(&built.mod_host);
        let cm = CommandManagerState::with_layout_path(
            &unibrowser_engine::load_command_manager_file(None),
            unibrowser_mod_api::CommandContext::Image,
            None,
        );
        let host = unibrowser_mod_api::HostContext {
            mode: unibrowser_mod_api::Mode::Image,
            doc_type: "image".into(),
            backend_id: "base-png".into(),
            selection: Default::default(),
            capabilities: Default::default(),
        };
        let families = cm.resolve_families_for_tab(
            &built.registry,
            &host,
            None,
            "image",
            Some("/workspace/examples/demo-pack/sample.png"),
            Some(&contributions),
            Some(&built.mod_runtime),
        );
        let ids: Vec<_> = families
            .iter()
            .flat_map(|f| f.groups.iter())
            .flat_map(|g| g.controls.iter())
            .map(|c| c.control_id.as_str())
            .collect();
        assert!(
            ids.contains(&"image-grayscale-preview.control.grayscale"),
            "expected grayscale control in resolved families, got {:?}",
            ids
        );
    }

    #[test]
    fn grayscale_mod_contributes_image_adjust_control() {
        let built = build_command_registry(&Mutex::new(HashSet::new())).expect("registry");
        let contributions = permitted_mod_contributions(&built.mod_host);
        let grayscale = contributions
            .iter()
            .find(|loaded| loaded.mod_name == "image-grayscale-preview")
            .expect("image-grayscale-preview contribution");
        let family = grayscale
            .contributes
            .families
            .iter()
            .find(|f| f.id == "unibrowser.family.image-adjust")
            .expect("image-adjust family extension");
        assert!(
            family
                .controls
                .iter()
                .any(|c| c.id == "image-grayscale-preview.control.grayscale")
        );
    }

    #[test]
    fn proving_toolbar_is_loaded_with_detach_permission() {
        let built = build_command_registry(&Mutex::new(HashSet::new())).expect("registry");
        let contributions = permitted_mod_contributions(&built.mod_host);
        let proving = contributions
            .iter()
            .find(|loaded| loaded.mod_name == "cmd-proving")
            .expect("cmd-proving contribution");
        assert!(proving.contributes.toolbars[0].detachable);
        assert!(
            built
                .registry
                .command_meta(cmd_proving::SUBSCRIPT_APPLY)
                .is_some()
        );
    }
}

