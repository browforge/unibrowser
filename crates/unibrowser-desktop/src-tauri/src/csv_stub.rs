//! CSV / TSV spreadsheet view stub (Apache-2.0; table UI in shell).

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "toy-csv-stub";
pub const VIEWER_KIND: &str = "spreadsheet";

pub struct CsvStubBackend {
    pub path: String,
}

impl unibrowser_engine::document::DocumentBackend for CsvStubBackend {
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

pub fn open_file(path: &std::path::Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(CsvStubBackend {
        path: path.to_string_lossy().into_owned(),
    })
}
