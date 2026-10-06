//! Minimal document backend for WASM-registered file extensions.

use std::path::Path;

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};
use unibrowser_engine::document::DocumentBackend;

pub const WASM_STUB_BACKEND_ID: &str = "wasm-mod-stub";

pub struct WasmStubBackend {
    pub path: String,
    pub content: String,
    pub mod_name: String,
}

impl WasmStubBackend {
    pub fn open_path(path: &Path, mod_name: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read file: {}", e))?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            content,
            mod_name: mod_name.to_string(),
        })
    }
}

impl DocumentBackend for WasmStubBackend {
    fn backend_id(&self) -> &str {
        WASM_STUB_BACKEND_ID
    }

    fn doc_type(&self) -> &str {
        "wasm-mod"
    }

    fn display_name(&self) -> &str {
        "WASM mod file"
    }

    fn mode(&self) -> Mode {
        Mode::Text
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }

    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("Font size not supported for WASM stub viewer".into())
    }

    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("Font size not supported for WASM stub viewer".into())
    }

    fn status_line(&self) -> String {
        format!(
            "Opened via WASM mod `{}` (stub text viewer)",
            self.mod_name
        )
    }

    fn set_demo_text_selection(&mut self, _selected: bool) {}

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
