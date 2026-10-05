use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::{
        EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID,
        PRINT_SELECTION_ID,
    },
};

pub const BACKEND_ID: &str = "base-docx";
pub const VIEWER_KIND: &str = "docx";

pub struct DocxBackend {
    pub path: String,
    pub byte_len: u64,
    pub reader_font_pt: f32,
    pub font_dirty: bool,
}

impl DocxBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
            reader_font_pt: 12.0,
            font_dirty: false,
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
            font_size_pt: Some(self.reader_font_pt),
            ..Default::default()
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        self.reader_font_pt = (self.reader_font_pt + 1.0).min(96.0);
        self.font_dirty = true;
        Ok(self.reader_font_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.reader_font_pt = (self.reader_font_pt - 1.0).max(6.0);
        self.font_dirty = true;
        Ok(self.reader_font_pt)
    }
    fn is_dirty(&self) -> bool {
        self.font_dirty
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!(
            "DOCX · {} · {:.1} pt preview · {} bytes",
            self.path,
            self.reader_font_pt,
            self.byte_len
        )
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn font_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    if !doc.backend.capabilities().font_size {
        return Enablement::Disabled {
            reason: "Format does not support font size".into(),
        };
    }
    Enablement::Enabled
}

fn exec_increase(doc: &mut DocumentSession) -> Result<String, String> {
    let en = font_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.backend.increase_font_size()?;
    Ok(format!("DOCX preview font: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let en = font_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("DOCX preview font: {:.1} pt", pt))
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_handler(HandlerRegistration {
        command_id: FONT_INCREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: exec_increase,
    });
    registry.register_handler(HandlerRegistration {
        command_id: FONT_DECREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: exec_decrease,
    });
    registry.register_handler(HandlerRegistration {
        command_id: FIND_IN_DOCUMENT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: |_doc| Ok("document-find".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: EXPORT_PRINT_PDF_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: |_doc| Ok("export-print-pdf".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: PRINT_SELECTION_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: |_doc| Ok("export-print-selection".into()),
    });
}

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
