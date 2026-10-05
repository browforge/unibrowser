use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

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

pub struct AppState {
    pub registry: Mutex<CommandRegistry>,
    pub sessions: Mutex<HashMap<String, SharedWindowSession>>,
    pub cwd: PathBuf,
    pub mods_loaded: String,
    #[cfg(feature = "dynamic-mods")]
    _dynamic_cdylib: Option<unibrowser_mod_host::dynamic::LoadedCdylibMod>,
    pub keybindings: Mutex<KeybindingMap>,
    pub command_manager: Mutex<CommandManagerState>,
    pub vsix_host_commands: Mutex<HashSet<String>>,
    pub key_sequences: Mutex<KeySequenceMatcher>,
    pub listing_control: ListingControl,
    pub filename_search_cancel: AtomicBool,
    pub _mod_host: ModHost,
}

impl AppState {
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

impl AppState {
    pub fn new() -> Self {
        let mut registry = CommandRegistry::with_builtins();
        let bundled = bundled_mod_registers();
        let mod_host =
            ModHost::discover_and_load(&[default_mods_dir()], &mut registry, &bundled).expect("mods");
        let vsix_host_commands = Mutex::new(HashSet::new());
        if let Some(vsix_root) = crate::config::vsix_extensions_dir() {
            unibrowser_vsix::register_enabled_contributions(&mut registry, &vsix_root, &vsix_host_commands);
        }
        let mut mods_loaded = mod_host.load_order.join(", ");
        #[cfg(all(feature = "dynamic-mods", target_os = "linux"))]
        let dynamic_cdylib = {
            let mut loaded = None;
            if std::env::var("UNIBROWSER_LOAD_CDYLIB_MOD").as_deref() == Ok("1") {
                use unibrowser_mod_host::dynamic::{default_cdylib_search_paths, load_cdylib_mod};
                for path in default_cdylib_search_paths() {
                    match load_cdylib_mod(&path, &mut registry) {
                        Ok(mod_lib) => {
                            if !mods_loaded.is_empty() {
                                mods_loaded.push_str(", ");
                            }
                            mods_loaded.push_str(&mod_lib.name);
                            loaded = Some(mod_lib);
                            break;
                        }
                        Err(_) => {}
                    }
                }
            }
            loaded
        };
        #[cfg(not(all(feature = "dynamic-mods", target_os = "linux")))]
        let dynamic_cdylib = None;
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
        let command_manager = Mutex::new(
            CommandManagerState::from_file(&cm_file, unibrowser_mod_api::CommandContext::Browse),
        );

        Self {
            registry: Mutex::new(registry),
            sessions,
            cwd,
            mods_loaded,
            keybindings,
            command_manager,
            vsix_host_commands,
            key_sequences: Mutex::new(KeySequenceMatcher::default()),
            listing_control: ListingControl::new(),
            filename_search_cancel: AtomicBool::new(false),
            _mod_host: mod_host,
            #[cfg(feature = "dynamic-mods")]
            _dynamic_cdylib: dynamic_cdylib,
        }
    }

    pub fn refresh_vsix_catalog(&self) {
        let mut registry = CommandRegistry::with_builtins();
        let bundled = bundled_mod_registers();
        for register in bundled.values() {
            register(&mut registry);
        }
        if let Some(vsix_root) = crate::config::vsix_extensions_dir() {
            unibrowser_vsix::register_enabled_contributions(
                &mut registry,
                &vsix_root,
                &self.vsix_host_commands,
            );
        }
        if let Ok(mut guard) = self.registry.lock() {
            *guard = registry;
        }
        if let Ok(mut kb) = self.keybindings.lock() {
            *kb = load_merged_keybindings();
        }
    }

    pub fn refresh_command_manager_context(&self, window_label: &str) {
        let sess = self.session(window_label).expect("session");
        let tabs = sess.tabs.lock().expect("tabs lock");
        let tab = tabs.active_tab();
        let ctx = if let Some(tab) = tab {
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
        } else {
            unibrowser_mod_api::CommandContext::Browse
        };
        let mut cm = self.command_manager.lock().expect("command manager lock");
        cm.select_tab_for_context(ctx);
    }

    pub fn resolve_location(
        &self,
        location: BrowserLocation,
    ) -> (BrowserLocation, bool, Option<DocumentSession>) {
        if let BrowserLocation::Http { url } = &location {
            if web_handles(&location) {
                let doc = DocumentSession::new(
                    url.clone(),
                    Box::new(WebBackend::new(url.clone())),
                );
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
        }
        (location, false, None)
    }

    pub fn navigate_input(&self, input: &str, window_label: &str) -> Result<BrowserLocation, String> {
        let loc = parse_user_input(input, &self.cwd);
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

pub fn open_file_document(path: &Path) -> Option<DocumentSession> {
    let path_str = path.to_string_lossy().to_string();
    let sniffed = crate::content_sniff::sniff_format(path);
    if unibrowser_project::is_unibrow_path(path) {
        let root = crate::config::unibrow_zip_cache_dir()
            .ok_or_else(|| "config path unavailable".to_string());
        let package = root
            .and_then(|cache| {
                unibrowser_project::resolve_package_root(path, &cache).map_err(|e| e.to_string())
            })
            .and_then(|root| unibrowser_project::open_package(&root).map_err(|e| e.to_string()));
        if let Ok(pkg) = package {
            use crate::project_backend::UnibrowBackend;
            return Some(DocumentSession::new(
                path_str,
                Box::new(UnibrowBackend::open(pkg)),
            ));
        }
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
        "docx" => base_docx::open_file(path).ok(),
        "pdf" => base_pdf::open_file(path).ok(),
        "dxf" => base_dxf::open_file(path).ok(),
        "js" | "rhai" => base_js::open_file(path).ok(),
        "csv" | "tsv" => Some(crate::csv_stub::open_file(path)),
        "txt" | "md" | "json" | "rs" | "toml" | "html" | "css" => base_text::open_file(path).ok(),
        "mp4" | "webm" | "avi" => base_video::open_file(path).ok(),
        "wav" | "mp3" | "ogg" | "flac" => base_audio::open_file(path).ok(),
        "mid" | "midi" => crate::midi_stub::open_file(path).ok(),
        "epub" => Some(crate::epub_stub::open_file(path)),
        _ => base_text::open_file(path).ok(),
    };
    backend.map(|b| DocumentSession::new(path_str, b))
}

pub fn list_dir(path: &Path) -> Result<Vec<base_files::DirEntry>, String> {
    let prefs = crate::preferences::load_preferences();
    let entries = list_directory(path)?;
    Ok(crate::files_listing::apply_files_view(entries, &prefs.files_view))
}

