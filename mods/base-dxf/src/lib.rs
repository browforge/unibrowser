use std::path::Path;

use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "base-dxf";
pub const VIEWER_KIND: &str = "dxf";

pub struct DxfBackend {
    pub path: String,
    pub byte_len: u64,
}

impl DxfBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
        })
    }
}

impl unibrowser_engine::document::DocumentBackend for DxfBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "image/vnd.dxf"
    }
    fn display_name(&self) -> &str {
        "DXF drawing"
    }
    fn mode(&self) -> Mode {
        Mode::Vector
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("Not applicable".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("Not applicable".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!("DXF · {} · {} bytes", self.path, self.byte_len)
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn register(_registry: &mut CommandRegistry) {}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(DxfBackend::from_path(path)?))
}
