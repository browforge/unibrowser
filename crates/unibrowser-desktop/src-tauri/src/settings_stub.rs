//! Internal `unibrowser://settings` document (chrome prefs surface, not a modal).

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "unibrowser-settings";
pub const VIEWER_KIND: &str = "settings";
pub const LOCATION: &str = "unibrowser://settings";

pub struct SettingsStubBackend;

impl SettingsStubBackend {
    pub fn new() -> Self {
        Self
    }
}

impl unibrowser_engine::document::DocumentBackend for SettingsStubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/vnd.unibrowser.settings"
    }
    fn display_name(&self) -> &str {
        "Settings"
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
        "Application preferences".into()
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
    Box::new(SettingsStubBackend::new())
}
