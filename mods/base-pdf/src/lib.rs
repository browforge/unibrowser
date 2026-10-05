//! PDF proving mod — pdf.js in the shell for rasterised view.
//!
//! Multi-engine composition (future, [ADR 0011](https://github.com/unibrowser/unibrowser/blob/main/docs/adr/0011-lazy-conversion-native-until-encode.md)):
//! - **Vector** — path/text objects in project graph
//! - **Raster** — page tiles / scrub proxies in `derived/`
//! - **Document** — text selection and office-like commands

use std::path::Path;

use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "base-pdf";
pub const VIEWER_KIND: &str = "pdf";

pub struct PdfBackend {
    pub path: String,
    pub byte_len: u64,
}

impl PdfBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
        })
    }
}

impl unibrowser_engine::document::DocumentBackend for PdfBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/pdf"
    }
    fn display_name(&self) -> &str {
        "PDF"
    }
    fn mode(&self) -> Mode {
        Mode::Document
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            font_size: false,
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("PDF font size uses document engine when text layer is selected".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("PDF font size uses document engine when text layer is selected".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!("PDF · {} · {} bytes (pdf.js view)", self.path, self.byte_len)
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn register(_registry: &mut CommandRegistry) {}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(PdfBackend::from_path(path)?))
}

pub fn try_pdf_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut PdfBackend> {
    backend.as_any_mut().downcast_mut::<PdfBackend>()
}
