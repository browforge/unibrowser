//! Internal `unibrowser://about` document.

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "unibrowser-about";
pub const VIEWER_KIND: &str = "about";
pub const LOCATION: &str = "unibrowser://about";

pub struct AboutStubBackend;

impl AboutStubBackend {
    pub fn new() -> Self {
        Self
    }
}

impl unibrowser_engine::document::DocumentBackend for AboutStubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/vnd.unibrowser.about"
    }
    fn display_name(&self) -> &str {
        "About Unibrowser"
    }
    fn mode(&self) -> Mode {
        Mode::Browse
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn status_line(&self) -> String {
        "About".into()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn open() -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(AboutStubBackend::new())
}
