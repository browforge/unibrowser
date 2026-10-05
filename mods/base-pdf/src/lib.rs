//! PDF proving mod — pdf.js in the shell for rasterised view.
//!
//! Multi-engine composition (future, [ADR 0011](https://github.com/unibrowser/unibrowser/blob/main/docs/adr/0011-lazy-conversion-native-until-encode.md)):
//! - **Vector** — path/text objects in project graph
//! - **Raster** — page tiles / scrub proxies in `derived/`
//! - **Document** — text selection and office-like commands

use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-pdf";
pub const VIEWER_KIND: &str = "pdf";

pub struct PdfBackend {
    pub path: String,
    pub byte_len: u64,
    pub render_scale: f32,
    pub text_selected: bool,
}

impl PdfBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
            render_scale: 1.25,
            text_selected: false,
        })
    }

    pub fn set_text_selected(&mut self, selected: bool) {
        self.text_selected = selected;
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
            font_size: true,
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot {
            domains: if self.text_selected {
                vec![SelectionDomain::Text]
            } else {
                vec![]
            },
            has_text_selection: self.text_selected,
            text_length: if self.text_selected { 1 } else { 0 },
            font_size_pt: Some(self.render_scale * 12.0),
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        self.render_scale = (self.render_scale + 0.15).min(4.0);
        Ok(self.render_scale * 12.0)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.render_scale = (self.render_scale - 0.15).max(0.5);
        Ok(self.render_scale * 12.0)
    }
    fn set_demo_text_selection(&mut self, selected: bool) {
        self.text_selected = selected;
    }
    fn status_line(&self) -> String {
        format!(
            "PDF · {} · {} bytes · scale {:.2}{}",
            self.path,
            self.byte_len,
            self.render_scale,
            if self.text_selected { " · text selected" } else { "" }
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
    if !doc.backend.selection().has_text_selection {
        return Enablement::Disabled {
            reason: "Select text in the PDF text layer".into(),
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
    Ok(format!("PDF view scale: {:.1} pt eq", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let en = font_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("PDF view scale: {:.1} pt eq", pt))
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
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(PdfBackend::from_path(path)?))
}

pub fn try_pdf_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut PdfBackend> {
    backend.as_any_mut().downcast_mut::<PdfBackend>()
}
