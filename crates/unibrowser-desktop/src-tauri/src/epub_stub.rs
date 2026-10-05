//! EPUB viewer stub — proves catalog slot before a real `base-epub` mod.

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::{FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "toy-epub-stub";
pub const VIEWER_KIND: &str = "epub";

pub struct EpubStubBackend {
    pub path: String,
    pub reader_font_pt: f32,
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
            "EPUB · {} · {:.1} pt reader · spine preview",
            self.path,
            self.reader_font_pt
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

pub fn try_epub_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut EpubStubBackend> {
    backend.as_any_mut().downcast_mut::<EpubStubBackend>()
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

pub fn register_handlers(registry: &mut CommandRegistry) {
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
}

pub fn open_file(path: &std::path::Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(EpubStubBackend {
        path: path.to_string_lossy().into_owned(),
        reader_font_pt: 12.0,
    })
}
