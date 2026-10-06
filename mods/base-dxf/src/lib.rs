use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, CommandMeta, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    handlers::{
        EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID,
        PAGE_ZOOM_FIT_ID,
    },
};

pub const BACKEND_ID: &str = "base-dxf";
pub const VIEWER_KIND: &str = "dxf";

pub const CMD_MEASURE: &str = "dxf.measure";
pub const CMD_EXPORT_STEP: &str = "dxf.export.step";

const DEFAULT_TEXT_HEIGHT: f32 = 2.5;

pub struct DxfBackend {
    pub path: String,
    pub byte_len: u64,
    pub text_entity_count: usize,
    pub text_height_scale: f32,
    pub preview_zoom: f32,
}

impl DxfBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let text_entity_count = count_dxf_text_entities(&content);
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            byte_len: meta.len(),
            text_entity_count,
            text_height_scale: 1.0,
            preview_zoom: 1.0,
        })
    }

    pub fn zoom_to_fit(&mut self) {
        self.preview_zoom = 1.0;
    }

    pub fn effective_text_height(&self) -> f32 {
        DEFAULT_TEXT_HEIGHT * self.text_height_scale
    }
}

pub fn count_dxf_text_entities(data: &str) -> usize {
    data.lines()
        .map(str::trim)
        .filter(|line| *line == "TEXT" || *line == "MTEXT")
        .count()
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
        Capabilities {
            font_size: self.text_entity_count > 0,
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        let has = self.text_entity_count > 0;
        SelectionSnapshot {
            domains: if has {
                vec![SelectionDomain::Text, SelectionDomain::Path]
            } else {
                vec![SelectionDomain::Path]
            },
            has_text_selection: has,
            text_length: if has { 1 } else { 0 },
            font_size_pt: Some(self.effective_text_height()),
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        if self.text_entity_count == 0 {
            return Err("DXF has no TEXT/MTEXT entities".into());
        }
        self.text_height_scale = (self.text_height_scale * 1.1).min(8.0);
        Ok(self.effective_text_height())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        if self.text_entity_count == 0 {
            return Err("DXF has no TEXT/MTEXT entities".into());
        }
        self.text_height_scale = (self.text_height_scale / 1.1).max(0.25);
        Ok(self.effective_text_height())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!(
            "DXF · {} · {} bytes · {} text · height {:.2} · zoom {:.0}%",
            self.path,
            self.byte_len,
            self.text_entity_count,
            self.effective_text_height(),
            self.preview_zoom * 100.0
        )
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn try_dxf_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut DxfBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<DxfBackend>()
}

fn font_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    if !doc.backend.capabilities().font_size {
        return Enablement::Disabled {
            reason: "DXF has no TEXT or MTEXT entities".into(),
        };
    }
    Enablement::Enabled
}

fn exec_increase(doc: &mut DocumentSession) -> Result<String, String> {
    let h = doc.backend.increase_font_size()?;
    Ok(format!("DXF text height: {:.2}", h))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let h = doc.backend.decrease_font_size()?;
    Ok(format!("DXF text height: {:.2}", h))
}

fn find_enable(doc: &DocumentSession) -> Enablement {
    font_enable(doc)
}

fn zoom_fit_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

fn exec_zoom_fit(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_dxf_backend(&mut *doc.backend).ok_or("not DXF")?;
    b.zoom_to_fit();
    Ok(format!("DXF zoom {:.0}%", b.preview_zoom * 100.0))
}

fn export_pdf_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Export PDF uses printable HTML (text, Markdown, or DOCX preview)".into(),
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
        enable: find_enable,
        execute: |_doc| Ok("document-find".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: PAGE_ZOOM_FIT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: zoom_fit_enable,
        execute: exec_zoom_fit,
    });
    registry.register_handler(HandlerRegistration {
        command_id: EXPORT_PRINT_PDF_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: export_pdf_disabled,
        execute: |_| Err("disabled".into()),
    });
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(DxfBackend::from_path(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use unibrowser_engine::document::DocumentSession;
    use unibrowser_mod_api::handlers::FONT_INCREASE_ID;

    #[test]
    fn font_increase_hits_dxf_text_entities() {
        let dir = std::env::temp_dir().join("unibrowser_dxf_font_test");
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let file = dir.join("note.dxf");
        std::fs::File::create(&file)
            .and_then(|mut f| f.write_all(b"0\nTEXT\n40\n2.5\n"))
            .expect("write");
        let mut doc = DocumentSession::new(file.to_string_lossy().as_ref(), open_file(&file).expect("open"));
        let mut reg = CommandRegistry::default();
        register(&mut reg);
        reg.execute(FONT_INCREASE_ID, &mut doc).expect("font");
        let dxf = try_dxf_backend(&mut *doc.backend).expect("dxf");
        assert!(dxf.text_height_scale > 1.0);
        let _ = std::fs::remove_dir_all(dir);
    }
}
