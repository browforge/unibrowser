use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};

use crate::listing_control::ListingControl;

use base_files::list_directory;
use base_web::{WebBackend, handles as web_handles};
use unibrowser_engine::{
    CommandManagerState, CommandRegistry, DocumentSession, KeySequenceMatcher, KeybindingMap,
    TabManager, infer_context, load_command_manager_file, navigation::parse_user_input,
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
    ]
    .into_iter()
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
    Ok(BuiltRegistry {
        registry,
        mod_host,
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
        let cwd = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
        let home = BrowserLocation::File {
            path: cwd.clone(),
        };
        let main_session = SharedWindowSession::new(WindowSession::new_with_home(&cwd));
        let sessions = Mutex::new(HashMap::from([("main".to_string(), main_session)]));
        let keybindings = Mutex::new(load_merged_keybindings());
        let cm_file = load_command_manager_file(toolbars_file_path().as_deref());
        let command_manager = Mutex::new(CommandManagerState::with_layout_path(
            &cm_file,
            unibrowser_mod_api::CommandContext::Start,
            toolbars_file_path().as_deref(),
        ));

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
            #[cfg(feature = "dynamic-mods")]
            dynamic_cdylibs: Mutex::new(built.dynamic_libs),
        }
    }

    pub fn refresh_extension_catalog(&self) -> Result<(), String> {
        let built = build_command_registry(&self.vsix_host_commands)?;
        if let Ok(mut guard) = self.registry.lock() {
            *guard = built.registry;
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
                let doc = open_file_document(path);
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

pub fn open_file_document_with_mode(path: &Path, mode: FileOpenMode) -> Option<DocumentSession> {
    let path_str = path.to_string_lossy().to_string();
    match mode {
        FileOpenMode::PlainText => {
            let backend = base_hex::HexBackend::open_path_as_text(path).ok()?;
            return Some(DocumentSession::new(path_str, Box::new(backend)));
        }
        FileOpenMode::Hex => {
            let backend = base_hex::HexBackend::open_path(path).ok()?;
            return Some(DocumentSession::new(path_str, Box::new(backend)));
        }
        FileOpenMode::Auto => {}
    }
    open_file_document(path)
}

pub fn open_file_document(path: &Path) -> Option<DocumentSession> {
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
    let effective = match sniffed {
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
        Some(crate::content_sniff::SniffedFormat::Zip) => ext.as_str(),
        None => ext.as_str(),
    };
    let backend: Option<Box<dyn unibrowser_engine::document::DocumentBackend>> = match effective
    {
        "png" | "jpg" | "jpeg" | "gif" | "webp" => Some(base_png::open_file(path)),
        "svg" => Some(base_svg::open_file(path)),
        "docx" => {
            let mut backend = base_docx::open_file(path).ok()?;
            let path_key = path.to_string_lossy();
            if let Some(pt) = crate::preferences::docx_reader_font(path_key.as_ref()) {
                if let Some(b) = base_docx::try_docx_backend(&mut *backend) {
                    b.reader_font_pt = pt;
                }
            }
            Some(backend)
        }
        "pdf" => base_pdf::open_file(path).ok(),
        "dxf" => base_dxf::open_file(path).ok(),
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" => base_js::open_file(path).ok(),
        "rhai" => base_text::open_file(path).ok(),
        "csv" | "tsv" => Some(base_csv::open_file(path)),
        "txt" | "md" | "json" | "rs" | "toml" | "html" | "css" => base_text::open_file(path).ok(),
        "mp4" | "webm" | "avi" => base_video::open_file(path).ok(),
        "wav" | "mp3" | "ogg" | "flac" => base_audio::open_file(path).ok(),
        "mid" | "midi" => crate::midi_stub::open_file(path).ok(),
        "epub" => Some(base_epub::open_file(path)),
        "zip" | "7z" | "rar" | "tar" | "gz" => None,
        _ if is_known_text_extension(effective) => base_text::open_file(path).ok(),
        _ => unibrowser_mod_host::wasm::open_wasm_extension_backend(path),
    };
    backend.map(|b| DocumentSession::new(path_str, b))
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

