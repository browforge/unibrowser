//! PDF proving mod — pdf.js in the shell for rasterised view.
//!
//! Multi-engine composition (future, [ADR 0011](https://github.com/unibrowser/unibrowser/blob/main/docs/adr/0011-lazy-conversion-native-until-encode.md)):
//! - **Vector** — path/text objects in project graph
//! - **Raster** — page tiles / scrub proxies in `derived/`
//! - **Document** — text selection and office-like commands

use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, CommandMeta, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    commands::{PDF_ANNOTATE_HIGHLIGHT, PDF_ANNOTATE_INK, PDF_ANNOTATE_STAMP},
    handlers::{
        EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID,
        PAGE_ZOOM_DECREASE_ID, PAGE_ZOOM_FIT_ID, PAGE_ZOOM_INCREASE_ID,
    },
};

pub const BACKEND_ID: &str = "base-pdf";
pub const VIEWER_KIND: &str = "pdf";

pub struct PdfBackend {
    pub path: String,
    pub byte_len: u64,
    pub render_scale: f32,
    pub text_selected: bool,
    pub page_index: u32,
}

impl PdfBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
            render_scale: 1.25,
            text_selected: false,
            page_index: 1,
        })
    }

    pub fn set_text_selected(&mut self, selected: bool) {
        self.text_selected = selected;
    }

    pub fn set_page_index(&mut self, page: u32) {
        self.page_index = page.max(1);
    }

    pub fn set_render_scale(&mut self, scale: f32) {
        self.render_scale = scale.clamp(0.5, 4.0);
    }

    pub fn zoom_to_fit(&mut self) {
        self.render_scale = 1.0;
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

fn page_zoom_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

fn exec_page_zoom_in(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    b.set_render_scale(b.render_scale * 1.1);
    Ok(format!("PDF zoom {:.0}%", b.render_scale * 100.0))
}

fn exec_page_zoom_out(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    b.set_render_scale(b.render_scale / 1.1);
    Ok(format!("PDF zoom {:.0}%", b.render_scale * 100.0))
}

fn exec_page_zoom_fit(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_pdf_backend(&mut *doc.backend).ok_or("not a PDF")?;
    b.zoom_to_fit();
    Ok(format!("PDF zoom {:.0}%", b.render_scale * 100.0))
}

fn find_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "PDF find uses the page search bar (not wired in proving mod)".into(),
    }
}

fn export_pdf_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Export PDF uses printable HTML (text, Markdown, or DOCX preview)".into(),
    }
}

fn pdf_only_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Open a PDF tab".into(),
        };
    }
    Enablement::Enabled
}

fn stamp_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "PDF stamp tool not implemented (sidecar highlights only)".into(),
    }
}

fn ink_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "PDF ink tool not implemented".into(),
    }
}

fn exec_highlight_arm(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("pdf-highlight-armed".into())
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: PDF_ANNOTATE_HIGHLIGHT,
        title: "Add PDF highlight (drag on page)",
        icon: "highlight",
        default_keybinding: None,
        aliases: &["pdf highlight"],
    });
    registry.register_command(CommandMeta {
        id: PDF_ANNOTATE_STAMP,
        title: "PDF stamp (not available)",
        icon: "stamp",
        default_keybinding: None,
        aliases: &[],
    });
    registry.register_command(CommandMeta {
        id: PDF_ANNOTATE_INK,
        title: "PDF ink (not available)",
        icon: "edit",
        default_keybinding: None,
        aliases: &[],
    });
    registry.register_handler(HandlerRegistration {
        command_id: PDF_ANNOTATE_HIGHLIGHT.into(),
        backend_id: BACKEND_ID.into(),
        enable: pdf_only_enable,
        execute: exec_highlight_arm,
    });
    registry.register_handler(HandlerRegistration {
        command_id: PDF_ANNOTATE_STAMP.into(),
        backend_id: BACKEND_ID.into(),
        enable: stamp_disabled,
        execute: |_| Err("disabled".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: PDF_ANNOTATE_INK.into(),
        backend_id: BACKEND_ID.into(),
        enable: ink_disabled,
        execute: |_| Err("disabled".into()),
    });
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
        command_id: PAGE_ZOOM_INCREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: page_zoom_enable,
        execute: exec_page_zoom_in,
    });
    registry.register_handler(HandlerRegistration {
        command_id: PAGE_ZOOM_DECREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: page_zoom_enable,
        execute: exec_page_zoom_out,
    });
    registry.register_handler(HandlerRegistration {
        command_id: PAGE_ZOOM_FIT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: page_zoom_enable,
        execute: exec_page_zoom_fit,
    });
    registry.register_handler(HandlerRegistration {
        command_id: FIND_IN_DOCUMENT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: find_disabled,
        execute: |_| Err("disabled".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: EXPORT_PRINT_PDF_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: export_pdf_disabled,
        execute: |_| Err("disabled".into()),
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
