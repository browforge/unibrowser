use unibrowser_mod_api::{
    BrowserLocation, Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::{
        FONT_DECREASE_ID, FONT_INCREASE_ID, PAGE_ZOOM_DECREASE_ID, PAGE_ZOOM_INCREASE_ID,
    },
};
use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};

pub const BACKEND_ID: &str = "base-web";

pub struct WebBackend {
    pub url: String,
}

impl WebBackend {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl unibrowser_engine::document::DocumentBackend for WebBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "text/html"
    }
    fn display_name(&self) -> &str {
        "Web page"
    }
    fn mode(&self) -> Mode {
        Mode::Web
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities { font_size: true }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("Not implemented".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("Not implemented".into())
    }
    fn status_line(&self) -> String {
        format!("Web · {}", self.url)
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn handles(location: &BrowserLocation) -> bool {
    match location {
        BrowserLocation::Http { url } => {
            url.starts_with("http://") || url.starts_with("https://")
        }
        _ => false,
    }
}

pub fn http_url(location: &BrowserLocation) -> Option<&str> {
    match location {
        BrowserLocation::Http { url } => Some(url.as_str()),
        _ => None,
    }
}

fn web_font_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

fn web_font_exec(doc: &mut DocumentSession) -> Result<String, String> {
    let _ = doc;
    Err("Use host user-style injection (execute_command intercept)".into())
}

fn web_page_zoom_enable(doc: &DocumentSession) -> Enablement {
    web_font_enable(doc)
}

fn web_page_zoom_exec(doc: &mut DocumentSession) -> Result<String, String> {
    let _ = doc;
    Err("Use host page zoom injection (execute_command intercept)".into())
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_handler(HandlerRegistration {
        command_id: FONT_INCREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: web_font_enable,
        execute: web_font_exec,
    });
    registry.register_handler(HandlerRegistration {
        command_id: FONT_DECREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: web_font_enable,
        execute: web_font_exec,
    });
    registry.register_handler(HandlerRegistration {
        command_id: PAGE_ZOOM_INCREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: web_page_zoom_enable,
        execute: web_page_zoom_exec,
    });
    registry.register_handler(HandlerRegistration {
        command_id: PAGE_ZOOM_DECREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: web_page_zoom_enable,
        execute: web_page_zoom_exec,
    });
}
