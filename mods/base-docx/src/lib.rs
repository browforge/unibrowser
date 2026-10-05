use std::path::Path;

use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "base-docx";
pub const VIEWER_KIND: &str = "docx";

pub struct DocxBackend {
    pub path: String,
    pub byte_len: u64,
}

impl DocxBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
        })
    }
}

impl unibrowser_engine::document::DocumentBackend for DocxBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    }
    fn display_name(&self) -> &str {
        "Word document"
    }
    fn mode(&self) -> Mode {
        Mode::Document
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            font_size: true,
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot {
            has_text_selection: true,
            text_length: 1,
            font_size_pt: Some(12.0),
            ..Default::default()
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Ok(13.0)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Ok(11.0)
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!(
            "DOCX · {} · {} bytes (mammoth HTML view)",
            self.path,
            self.byte_len
        )
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn register(_registry: &mut CommandRegistry) {}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(DocxBackend::from_path(path)?))
}

pub fn try_docx_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut DocxBackend> {
    backend
        .as_any_mut()
        .downcast_mut::<DocxBackend>()
}
