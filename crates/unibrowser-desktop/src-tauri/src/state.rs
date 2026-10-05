use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base_files::list_directory;
use base_web::{WebBackend, handles as web_handles};
use unibrowser_engine::{
    CommandManagerState, CommandRegistry, DocumentSession, KeybindingMap, TabManager,
    infer_context, load_command_manager_file,
    navigation::parse_user_input,
};

use crate::config::{keybindings_file_path, toolbars_file_path};
use unibrowser_mod_api::BrowserLocation;
use unibrowser_mod_host::{ModHost, ModRegisterFn, default_mods_dir};

pub struct AppState {
    pub registry: CommandRegistry,
    pub tabs: Mutex<TabManager>,
    pub cwd: PathBuf,
    pub mods_loaded: String,
    pub keybindings: KeybindingMap,
    pub command_manager: Mutex<CommandManagerState>,
    pub _mod_host: ModHost,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = CommandRegistry::with_builtins();
        let bundled: HashMap<&str, ModRegisterFn> = [
            ("base-files", base_files::register as ModRegisterFn),
            ("base-web", base_web::register as ModRegisterFn),
            ("base-text", base_text::register as ModRegisterFn),
            ("base-png", base_png::register as ModRegisterFn),
            ("base-svg", base_svg::register as ModRegisterFn),
            ("base-docx", base_docx::register as ModRegisterFn),
            ("base-pdf", base_pdf::register as ModRegisterFn),
            ("base-dxf", base_dxf::register as ModRegisterFn),
            ("base-js", base_js::register as ModRegisterFn),
            ("toy-markdown", toy_markdown::register as ModRegisterFn),
        ]
        .into_iter()
        .collect();
        let mod_host =
            ModHost::discover_and_load(&[default_mods_dir()], &mut registry, &bundled).expect("mods");
        let mods_loaded = mod_host.load_order.join(", ");
        let cwd = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
        let home = BrowserLocation::File {
            path: cwd.clone(),
        };
        let mut tab_mgr = TabManager::new_with_home(home);
        if let Some(tab) = tab_mgr.active_tab_mut() {
            tab.is_directory = cwd.is_dir();
        }
        let keybindings = KeybindingMap::load_merged(keybindings_file_path().as_deref());
        let cm_file = load_command_manager_file(toolbars_file_path().as_deref());
        let command_manager = Mutex::new(
            CommandManagerState::from_file(&cm_file, unibrowser_mod_api::CommandContext::Browse),
        );

        Self {
            registry,
            tabs: Mutex::new(tab_mgr),
            cwd,
            mods_loaded,
            keybindings,
            command_manager,
            _mod_host: mod_host,
        }
    }

    pub fn refresh_command_manager_context(&self) {
        let tabs = self.tabs.lock().expect("tabs lock");
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
        }
        (location, false, None)
    }

    pub fn navigate_input(&self, input: &str) -> Result<BrowserLocation, String> {
        let loc = parse_user_input(input, &self.cwd);
        let (loc, is_dir, doc) = self.resolve_location(loc);
        let mut tabs = self.tabs.lock().map_err(|e| e.to_string())?;
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
    if unibrowser_project::is_unibrow_path(path) {
        let package = unibrowser_project::open_package(path).map_err(|e| e.to_string());
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
    let backend: Option<Box<dyn unibrowser_engine::document::DocumentBackend>> = match ext.as_str()
    {
        "png" => Some(base_png::open_file(path)),
        "svg" => Some(base_svg::open_file(path)),
        "docx" => base_docx::open_file(path).ok(),
        "pdf" => base_pdf::open_file(path).ok(),
        "dxf" => base_dxf::open_file(path).ok(),
        "js" | "rhai" => base_js::open_file(path).ok(),
        "txt" | "md" | "json" | "rs" | "toml" | "html" | "css" => base_text::open_file(path).ok(),
        "mp4" | "webm" | "wav" | "mid" | "midi" => {
            // Media opens as status + optional ffprobe until video/audio engine lands.
            Some(media_placeholder_backend(path))
        }
        _ => base_text::open_file(path).ok(),
    };
    backend.map(|b| DocumentSession::new(path_str, b))
}

pub fn list_dir(path: &Path) -> Result<Vec<base_files::DirEntry>, String> {
    list_directory(path)
}

struct MediaPlaceholderBackend {
    path: String,
    status: String,
}

impl unibrowser_engine::document::DocumentBackend for MediaPlaceholderBackend {
    fn backend_id(&self) -> &str {
        "media-placeholder"
    }
    fn doc_type(&self) -> &str {
        "application/octet-stream"
    }
    fn display_name(&self) -> &str {
        "Media"
    }
    fn mode(&self) -> unibrowser_mod_api::Mode {
        unibrowser_mod_api::Mode::Browse
    }
    fn capabilities(&self) -> unibrowser_mod_api::Capabilities {
        unibrowser_mod_api::Capabilities::default()
    }
    fn selection(&self) -> unibrowser_mod_api::SelectionSnapshot {
        unibrowser_mod_api::SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        self.status.clone()
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn media_placeholder_backend(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    let path_str = path.to_string_lossy().to_string();
    let status = match unibrowser_media::probe_media_file(path) {
        Ok(p) => format!(
            "Media · {} · ffprobe: {}",
            path_str,
            p.ffprobe_summary.unwrap_or_else(|| "ok".into())
        ),
        Err(e) => format!("Media · {} · ffmpeg T3: {}", path_str, e),
    };
    Box::new(MediaPlaceholderBackend {
        path: path_str,
        status,
    })
}
