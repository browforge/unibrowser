use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{Capabilities, CommandMeta, Enablement, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "base-dxf";
pub const VIEWER_KIND: &str = "dxf";

pub const CMD_MEASURE: &str = "dxf.measure";
pub const CMD_EXPORT_STEP: &str = "dxf.export.step";

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
        format!(
            "DXF proving mod · {} · {} bytes (wireframe preview)",
            self.path,
            self.byte_len
        )
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn cad_tool_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Proving mod: requires full CAD engine (not bundled)".into(),
    }
}

fn cad_tool_stub(_doc: &mut DocumentSession) -> Result<String, String> {
    Err("CAD tool not available in proving mod".into())
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: CMD_MEASURE,
        title: "Measure distance (CAD)",
        icon: "ruler",
        default_keybinding: None,
        aliases: &["measure", "cad measure"],
    });
    registry.register_command(CommandMeta {
        id: CMD_EXPORT_STEP,
        title: "Export STEP (CAD)",
        icon: "export",
        default_keybinding: None,
        aliases: &["step export"],
    });
    for cmd in [CMD_MEASURE, CMD_EXPORT_STEP] {
        registry.register_handler(HandlerRegistration {
            command_id: cmd.into(),
            backend_id: BACKEND_ID.into(),
            enable: cad_tool_disabled,
            execute: cad_tool_stub,
        });
    }
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(DxfBackend::from_path(path)?))
}
