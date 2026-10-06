use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::{
        EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID,
        PAGE_ZOOM_FIT_ID,
    },
};

pub const BACKEND_ID: &str = "base-epub";
pub const VIEWER_KIND: &str = "epub";

pub struct EpubBackend {
    pub path: String,
    pub reader_font_pt: f32,
    pub reader_scale: f32,
}

impl unibrowser_engine::document::DocumentBackend for EpubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/epub+zip"
    }
    fn display_name(&self) -> &str {
        "EPUB"
    }
    fn mode(&self) -> Mode {
        Mode::Browse
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
    fn status_line(&self) -> String {
        format!(
            "EPUB · {} · {:.1} pt reader · scale {:.0}%",
            self.path,
            self.reader_font_pt,
            self.reader_scale * 100.0
        )
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        self.reader_font_pt = (self.reader_font_pt + 1.0).min(96.0);
        Ok(self.reader_font_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.reader_font_pt = (self.reader_font_pt - 1.0).max(6.0);
        Ok(self.reader_font_pt)
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl EpubBackend {
    pub fn zoom_to_fit(&mut self) {
        self.reader_scale = 1.0;
    }
}

pub fn try_epub_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut EpubBackend> {
    backend.as_any_mut().downcast_mut::<EpubBackend>()
}

fn font_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

fn exec_increase(doc: &mut DocumentSession) -> Result<String, String> {
    let pt = doc.backend.increase_font_size()?;
    Ok(format!("EPUB reader font: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("EPUB reader font: {:.1} pt", pt))
}

fn exec_zoom_fit(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_epub_backend(&mut *doc.backend).ok_or("not EPUB")?;
    b.zoom_to_fit();
    Ok(format!("EPUB reader scale {:.0}%", b.reader_scale * 100.0))
}

fn export_pdf_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Export PDF uses printable HTML (text, Markdown, or DOCX preview)".into(),
    }
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
        command_id: PAGE_ZOOM_FIT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: exec_zoom_fit,
    });
    registry.register_handler(HandlerRegistration {
        command_id: EXPORT_PRINT_PDF_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: export_pdf_disabled,
        execute: |_| Err("disabled".into()),
    });
}

pub fn open_file(path: &std::path::Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(EpubBackend {
        path: path.to_string_lossy().into_owned(),
        reader_font_pt: 12.0,
        reader_scale: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_engine::document::DocumentSession;
    use unibrowser_mod_api::handlers::FONT_INCREASE_ID;

    #[test]
    fn font_increase_hits_epub_handler() {
        let mut doc = DocumentSession::new(
            "/books/sample.epub",
            open_file(std::path::Path::new("/books/sample.epub")),
        );
        let mut reg = CommandRegistry::default();
        register(&mut reg);
        reg.execute(FONT_INCREASE_ID, &mut doc).expect("font");
        let epub = try_epub_backend(&mut *doc.backend).expect("epub");
        assert_eq!(epub.reader_font_pt, 13.0);
    }
}
