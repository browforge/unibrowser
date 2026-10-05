//! EPUB viewer stub — proves catalog slot before a real `base-epub` mod.

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "toy-epub-stub";
pub const VIEWER_KIND: &str = "epub";

pub struct EpubStubBackend {
    pub path: String,
}

impl unibrowser_engine::document::DocumentBackend for EpubStubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/epub+zip"
    }
    fn display_name(&self) -> &str {
        "EPUB (preview stub)"
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
        format!(
            "EPUB stub for {} — spine rendering planned in base-epub mod",
            self.path
        )
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("EPUB stub does not support font commands".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("EPUB stub does not support font commands".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn open_file(path: &std::path::Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(EpubStubBackend {
        path: path.to_string_lossy().into_owned(),
    })
}
