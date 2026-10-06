use std::path::Path;

use image::{GenericImageView, ImageReader};
use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    handlers::{
        CAPTURE_VISIBLE_TAB_ID, COMPARE_IMAGES_ID, EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID,
        FONT_DECREASE_ID, FONT_INCREASE_ID, PAGE_ZOOM_DECREASE_ID, PAGE_ZOOM_FIT_ID,
        PAGE_ZOOM_INCREASE_ID,
    },
};

pub const BACKEND_ID: &str = "base-png";
pub const VIEWER_KIND: &str = "image";

pub struct PngBackend {
    pub width: u32,
    pub height: u32,
    /// Simulated text layer (real mod would read PNG metadata / sidecar).
    pub text_layer: Option<TextLayer>,
    pub text_layer_selected: bool,
    pub preview_zoom: f32,
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
            preview_zoom: 1.0,
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
            preview_zoom: 1.0,
        }
    }

    pub fn adjust_preview_zoom(&mut self, factor: f32) {
        self.preview_zoom = (self.preview_zoom * factor).clamp(0.15, 8.0);
    }

    pub fn zoom_to_fit(&mut self) {
        const VW: f32 = 960.0;
        const VH: f32 = 720.0;
        let zw = VW / self.width.max(1) as f32;
        let zh = VH / self.height.max(1) as f32;
        self.preview_zoom = zw.min(zh).clamp(0.15, 8.0);
    }
}

/// Read `unibrowser_text` from PNG `tEXt` / `iTXt` chunks (`font_pt=18;label=Title`).
pub fn parse_png_unibrowser_text_annotation(data: &[u8]) -> Option<TextLayer> {
    if data.len() < 12 {
        return None;
    }
    let mut off = 8usize;
    while off + 12 <= data.len() {
        let len = u32::from_be_bytes(data[off..off + 4].try_into().ok()?) as usize;
        let typ = &data[off + 4..off + 8];
        let chunk_start = off + 8;
        let chunk_end = chunk_start + len;
        if chunk_end > data.len() {
            break;
        }
        let payload = &data[chunk_start..chunk_end];
        if typ == b"tEXt" {
            if let Some(layer) = parse_text_chunk_payload(payload) {
                return Some(layer);
            }
        } else if typ == b"iTXt" {
            if let Some(layer) = parse_itxt_payload(payload) {
                return Some(layer);
            }
        }
        off = chunk_end + 4;
    }
    None
}

fn parse_text_chunk_payload(payload: &[u8]) -> Option<TextLayer> {
    let nul = payload.iter().position(|&b| b == 0)?;
    let key = &payload[..nul];
    if key != b"unibrowser_text" {
        return None;
    }
    let value = std::str::from_utf8(&payload[nul + 1..]).ok()?;
    parse_annotation_value(value)
}

fn parse_itxt_payload(payload: &[u8]) -> Option<TextLayer> {
    let mut parts = payload.split(|&b| b == 0);
    let key = parts.next()?;
    if key != b"unibrowser_text" {
        return None;
    }
    let _compression = parts.next()?;
    let _method = parts.next()?;
    let _lang = parts.next()?;
    let _translated = parts.next()?;
    let value = parts.next()?;
    let value = std::str::from_utf8(value).ok()?;
    parse_annotation_value(value)
}

fn parse_annotation_value(value: &str) -> Option<TextLayer> {
    let mut font_size_pt = 18.0f32;
    let mut label = "Annotation".to_string();
    for part in value.split(';') {
        let mut kv = part.splitn(2, '=');
        let k = kv.next()?.trim();
        let v = kv.next()?.trim();
        match k {
            "font_pt" => {
                font_size_pt = v.parse().unwrap_or(font_size_pt);
            }
            "label" => {
                label = v.to_string();
            }
            _ => {}
        }
    }
    Some(TextLayer {
        label,
        font_size_pt,
    })
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
            reason: "This image has no text annotation layer (PNG tEXt unibrowser_text)".into(),
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

fn page_zoom_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

fn exec_page_zoom_in(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_png_backend(&mut *doc.backend).ok_or("not an image")?;
    b.adjust_preview_zoom(1.1);
    Ok(format!("Image zoom {:.0}%", b.preview_zoom * 100.0))
}

fn exec_page_zoom_out(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_png_backend(&mut *doc.backend).ok_or("not an image")?;
    b.adjust_preview_zoom(1.0 / 1.1);
    Ok(format!("Image zoom {:.0}%", b.preview_zoom * 100.0))
}

fn exec_page_zoom_fit(doc: &mut DocumentSession) -> Result<String, String> {
    let b = try_png_backend(&mut *doc.backend).ok_or("not an image")?;
    b.zoom_to_fit();
    Ok(format!("Image zoom {:.0}%", b.preview_zoom * 100.0))
}

fn find_enable(doc: &DocumentSession) -> Enablement {
    font_enable(doc)
}

fn export_pdf_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Export PDF uses printable HTML (text, Markdown, or DOCX preview)".into(),
    }
}

pub fn try_png_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut PngBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<PngBackend>()
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
        enable: find_enable,
        execute: |_doc| Ok("document-find".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: EXPORT_PRINT_PDF_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: export_pdf_disabled,
        execute: |_| Err("disabled".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: COMPARE_IMAGES_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: image_compare_enable,
        execute: image_compare_exec,
    });
    registry.register_handler(HandlerRegistration {
        command_id: CAPTURE_VISIBLE_TAB_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: image_compare_enable,
        execute: image_compare_exec,
    });
}

fn image_compare_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Compare images needs a PNG/JPEG tab".into(),
        };
    }
    Enablement::Enabled
}

fn image_compare_exec(doc: &mut DocumentSession) -> Result<String, String> {
    let _ = doc;
    Err("Use host compare picker (execute_command intercept)".into())
}

pub fn open_file(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    let (width, height) = read_dimensions(path).unwrap_or((640, 480));
    let mut backend = if let Ok(bytes) = std::fs::read(path) {
        if let Some(layer) = parse_png_unibrowser_text_annotation(&bytes) {
            let mut b = PngBackend::without_text_layer();
            b.text_layer = Some(layer);
            b.text_layer_selected = true;
            b
        } else {
            PngBackend::without_text_layer()
        }
    } else {
        PngBackend::without_text_layer()
    };
    backend.width = width;
    backend.height = height;
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

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_engine::document::DocumentSession;
    use unibrowser_mod_api::handlers::FONT_INCREASE_ID;

    #[test]
    fn font_increase_hits_png_text_annotation() {
        let dir = std::env::temp_dir().join("unibrowser_png_font_test");
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let file = dir.join("annot.png");
        write_min_png_with_text(&file, "font_pt=12;label=Title");
        let mut doc = DocumentSession::new(file.to_string_lossy().as_ref(), open_file(&file));
        let mut reg = CommandRegistry::default();
        register(&mut reg);
        reg.execute(FONT_INCREASE_ID, &mut doc).expect("font");
        let png = try_png_backend(&mut *doc.backend).expect("png");
        assert_eq!(png.text_layer.as_ref().map(|t| t.font_size_pt), Some(13.0));
        let _ = std::fs::remove_dir_all(dir);
    }

    fn write_min_png_with_text(path: &Path, annotation: &str) {
        let mut out = Vec::new();
        out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
        append_chunk(&mut out, b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]);
        let text = format!("unibrowser_text\0{}", annotation);
        append_chunk(&mut out, b"tEXt", text.as_bytes());
        append_chunk(&mut out, b"IEND", &[]);
        std::fs::write(path, out).expect("png");
    }

    fn append_chunk(out: &mut Vec<u8>, typ: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(typ);
        out.extend_from_slice(data);
        let crc = crc32_chunk(typ, data);
        out.extend_from_slice(&crc.to_be_bytes());
    }

    fn crc32_chunk(typ: &[u8; 4], data: &[u8]) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(typ);
        hasher.update(data);
        hasher.finalize()
    }
}
