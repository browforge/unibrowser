use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::{FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-csv";
pub const VIEWER_KIND: &str = "spreadsheet";

pub struct CsvBackend {
    pub path: String,
}

impl unibrowser_engine::document::DocumentBackend for CsvBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "text/csv"
    }
    fn display_name(&self) -> &str {
        "Spreadsheet (CSV)"
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
        format!("CSV table preview for {}", self.path)
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("CSV preview is read-only".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("CSV preview is read-only".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn font_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Spreadsheet view uses table zoom, not reader font size".into(),
    }
}

fn find_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Use column filter in the spreadsheet view (find bar is for documents)".into(),
    }
}

pub fn register(registry: &mut CommandRegistry) {
    for cmd in [FONT_INCREASE_ID, FONT_DECREASE_ID] {
        registry.register_handler(HandlerRegistration {
            command_id: cmd.into(),
            backend_id: BACKEND_ID.into(),
            enable: font_disabled,
            execute: |_| Err("disabled".into()),
        });
    }
    registry.register_handler(HandlerRegistration {
        command_id: FIND_IN_DOCUMENT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: find_disabled,
        execute: |_| Err("disabled".into()),
    });
}

pub fn open_file(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(CsvBackend {
        path: path.to_string_lossy().into_owned(),
    })
}
