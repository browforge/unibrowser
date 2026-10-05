//! Internal `unibrowser://keys` document (keyboard shortcut reference).

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "unibrowser-keys";
pub const VIEWER_KIND: &str = "keys";
pub const LOCATION: &str = "unibrowser://keys";

pub struct KeysStubBackend;

impl KeysStubBackend {
    pub fn new() -> Self {
        Self
    }
}

impl unibrowser_engine::document::DocumentBackend for KeysStubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/vnd.unibrowser.keys"
    }
    fn display_name(&self) -> &str {
        "Keyboard shortcuts"
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
        "Shortcut reference".into()
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
    Box::new(KeysStubBackend::new())
}
