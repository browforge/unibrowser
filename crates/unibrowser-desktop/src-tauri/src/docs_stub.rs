//! Internal `unibrowser://docs/{slug}` help viewer.

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "unibrowser-docs";
pub const VIEWER_KIND: &str = "docs";

pub struct DocsStubBackend {
    pub slug: String,
}

impl DocsStubBackend {
    pub fn new(slug: &str) -> Self {
        Self {
            slug: slug.to_string(),
        }
    }
}

impl unibrowser_engine::document::DocumentBackend for DocsStubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/vnd.unibrowser.docs"
    }
    fn display_name(&self) -> &str {
        "Documentation"
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
        format!("Help: {}", self.slug)
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

pub fn location_for_slug(slug: &str) -> String {
    format!("unibrowser://docs/{}", slug)
}

pub fn open(slug: &str) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(DocsStubBackend::new(slug))
}

pub fn parse_slug_from_location(path: &str) -> Option<String> {
    if path.starts_with("unibrowser://docs/") {
        let slug = path.trim_start_matches("unibrowser://docs/").trim();
        if slug.is_empty() {
            return None;
        }
        return Some(slug.split('/').next().unwrap_or(slug).to_string());
    }
    None
}
