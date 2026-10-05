use std::path::Path;

use image::{GenericImageView, ImageReader};
use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-png";
pub const VIEWER_KIND: &str = "image";

pub struct PngBackend {
    pub width: u32,
    pub height: u32,
    /// Simulated text layer (real mod would read PNG metadata / sidecar).
    pub text_layer: Option<TextLayer>,
    pub text_layer_selected: bool,
}

pub struct TextLayer {
    pub label: String,
    pub font_size_pt: f32,
}

impl PngBackend {
    pub fn without_text_layer() -> Self {
        Self {
            width: 640,
            height: 480,
            text_layer: None,
            text_layer_selected: false,
        }
    }

    pub fn with_text_layer() -> Self {
        Self {
            width: 640,
            height: 480,
            text_layer: Some(TextLayer {
                label: "Title".into(),
                font_size_pt: 18.0,
            }),
            text_layer_selected: true,
        }
    }
}

impl unibrowser_engine::document::DocumentBackend for PngBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "image/png"
    }
    fn display_name(&self) -> &str {
        "PNG image"
    }
    fn mode(&self) -> Mode {
        Mode::Image
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            font_size: self.text_layer.is_some(),
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        let text_sel = self.text_layer.is_some() && self.text_layer_selected;
        SelectionSnapshot {
            domains: if text_sel {
                vec![SelectionDomain::Text, SelectionDomain::Bitmap]
            } else if self.text_layer.is_some() {
                vec![SelectionDomain::Bitmap]
            } else {
                vec![SelectionDomain::Bitmap]
            },
            text_length: if text_sel { 5 } else { 0 },
            has_text_selection: text_sel,
            font_size_pt: self.text_layer.as_ref().map(|t| t.font_size_pt),
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        let layer = self
            .text_layer
            .as_mut()
            .ok_or_else(|| "No text layer".to_string())?;
        layer.font_size_pt = (layer.font_size_pt + 1.0).min(96.0);
        Ok(layer.font_size_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        let layer = self
            .text_layer
            .as_mut()
            .ok_or_else(|| "No text layer".to_string())?;
        layer.font_size_pt = (layer.font_size_pt - 1.0).max(6.0);
        Ok(layer.font_size_pt)
    }
    fn status_line(&self) -> String {
        match &self.text_layer {
            Some(t) => format!(
                "PNG {}×{} · text layer '{}' {} pt",
                self.width,
                self.height,
                t.label,
                t.font_size_pt
            ),
            None => format!("PNG {}×{} · no text layer", self.width, self.height),
        }
    }
    fn set_demo_text_selection(&mut self, selected: bool) {
        self.text_layer_selected = selected && self.text_layer.is_some();
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
    if !doc.backend.capabilities().font_size {
        return Enablement::Disabled {
            reason: "PNG has no editable text layer".into(),
        };
    }
    if !doc.backend.selection().has_text_selection {
        return Enablement::Disabled {
            reason: "Select the text layer".into(),
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
    Ok(format!("Text layer font: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let en = font_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("Text layer font: {:.1} pt", pt))
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

pub fn open_file(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    let (width, height) = read_dimensions(path).unwrap_or((640, 480));
    let mut backend = PngBackend::without_text_layer();
    backend.width = width;
    backend.height = height;
    if path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.contains("text"))
        .unwrap_or(false)
    {
        backend = PngBackend::with_text_layer();
        backend.width = width;
        backend.height = height;
    }
    Box::new(backend)
}

fn read_dimensions(path: &Path) -> Option<(u32, u32)> {
    let reader = ImageReader::open(path).ok()?;
    let img = reader.decode().ok()?;
    Some(img.dimensions())
}

pub fn open_sample(path: &str) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    open_file(Path::new(path))
}
