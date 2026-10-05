use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base_files::list_directory;
use base_web::{WebBackend, handles as web_handles};
use unibrowser_engine::{
    CommandRegistry, DocumentSession, TabManager,
    navigation::parse_user_input,
};
use unibrowser_mod_api::BrowserLocation;
use unibrowser_mod_host::{ModHost, ModRegisterFn, default_mods_dir};

pub struct AppState {
    pub registry: CommandRegistry,
    pub tabs: Mutex<TabManager>,
    pub cwd: PathBuf,
    pub mods_loaded: String,
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
        Self {
            registry,
            tabs: Mutex::new(tab_mgr),
            cwd,
            mods_loaded,
            _mod_host: mod_host,
        }
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
                return (location, false, Some(doc));
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

fn open_file_document(path: &Path) -> DocumentSession {
    let path_str = path.to_string_lossy();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let backend: Box<dyn unibrowser_engine::document::DocumentBackend> = match ext {
        "png" => base_png::open_sample(&path_str),
        "md" => toy_markdown::open_sample(&path_str),
        _ => base_text::open_sample(&path_str),
    };
    DocumentSession::new(path_str.to_string(), backend)
}

pub fn list_dir(path: &Path) -> Result<Vec<base_files::DirEntry>, String> {
    list_directory(path)
}
