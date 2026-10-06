use std::fs;
use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, CommandMeta, Enablement, Mode, SelectionDomain, SelectionSnapshot, commands,
    handlers::{
        DOCUMENT_REDO_ID, DOCUMENT_UNDO_ID, EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID,
        FONT_DECREASE_ID, FONT_INCREASE_ID, PAGE_ZOOM_FIT_ID,
    },
};

pub const BACKEND_ID: &str = "base-svg";
pub const VIEWER_KIND: &str = "svg";

pub struct SvgBackend {
    pub path: String,
    pub content: String,
    pub width: u32,
    pub height: u32,
    pub element_count: usize,
    pub text_element_count: usize,
    pub text_selected: bool,
    pub font_size_pt: f32,
    pub preview_zoom: f32,
    pub dirty: bool,
}

impl SvgBackend {
    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let content = fs::read_to_string(path).unwrap_or_default();
        let (width, height, element_count) = parse_svg_metadata(&content);
        let text_element_count = count_svg_text_elements(&content);
        Self {
            path: path_str,
            content,
            width,
            height,
            element_count,
            text_element_count,
            text_selected: text_element_count > 0,
            font_size_pt: 14.0,
            preview_zoom: 1.0,
            dirty: false,
        }
    }

    pub fn set_content(&mut self, content: String) {
        if self.content != content {
            self.content = content;
            self.dirty = true;
            let (width, height, element_count) = parse_svg_metadata(&self.content);
            self.width = width;
            self.height = height;
            self.element_count = element_count;
            self.text_element_count = count_svg_text_elements(&self.content);
            self.text_selected = self.text_element_count > 0;
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("No file path for save".into());
        }
        fs::write(&self.path, &self.content).map_err(|e| format!("Save failed: {}", e))?;
        self.dirty = false;
        Ok(())
    }

    pub fn zoom_to_fit(&mut self) {
        const VW: f32 = 960.0;
        const VH: f32 = 720.0;
        let zw = VW / self.width.max(1) as f32;
        let zh = VH / self.height.max(1) as f32;
        self.preview_zoom = zw.min(zh).clamp(0.15, 8.0);
    }
}

pub fn count_svg_text_elements(content: &str) -> usize {
    let lower = content.to_lowercase();
    lower.matches("<text").count() + lower.matches("<tspan").count()
}

/// Scale quoted `font-size` values in SVG markup (used by font-size commands).
pub fn adjust_svg_text_font_sizes(content: &str, delta_pt: f32) -> String {
    let needle = b"font-size";
    let bytes = content.as_bytes();
    let mut out = String::with_capacity(content.len() + 32);
    let mut i = 0usize;
    while i < bytes.len() {
        if i + needle.len() <= bytes.len() && &bytes[i..i + needle.len()] == needle {
            out.push_str("font-size");
            i += needle.len();
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                out.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'=' {
                out.push('=');
                i += 1;
                while i < bytes.len() && bytes[i] == b' ' {
                    out.push(' ');
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'"' {
                    out.push('"');
                    i += 1;
                    let start = i;
                    while i < bytes.len() && bytes[i] != b'"' {
                        i += 1;
                    }
                    let val_str = &content[start..i];
                    out.push_str(&scale_font_size_value(val_str, delta_pt));
                    if i < bytes.len() && bytes[i] == b'"' {
                        out.push('"');
                        i += 1;
                    }
                    continue;
                }
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn scale_font_size_value(raw: &str, delta_pt: f32) -> String {
    let trimmed = raw.trim();
    let (num, suffix) = if let Some(stripped) = trimmed.strip_suffix("px") {
        (stripped, "px")
    } else if let Some(stripped) = trimmed.strip_suffix("pt") {
        (stripped, "pt")
    } else {
        (trimmed, "")
    };
    let base = num.parse::<f32>().unwrap_or(12.0);
    let next = (base + delta_pt).clamp(4.0, 256.0);
    if suffix.is_empty() {
        format!("{:.2}", next)
    } else {
        format!("{:.2}{}", next, suffix)
    }
}

fn parse_svg_metadata(text: &str) -> (u32, u32, usize) {
    let width = extract_attr(text, "width").unwrap_or(800.0) as u32;
    let height = extract_attr(text, "height").unwrap_or(600.0) as u32;
    let element_count = text.matches('<').count().saturating_sub(1);
    (width.max(1), height.max(1), element_count)
}

fn extract_attr(svg: &str, name: &str) -> Option<f32> {
    let needle = format!("{}=\"", name);
    let start = svg.find(&needle)?;
    let rest = &svg[start + needle.len()..];
    let end = rest.find('"')?;
    let raw = &rest[..end];
    raw.trim_end_matches("px")
        .trim_end_matches("pt")
        .parse()
        .ok()
}

pub fn try_svg_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut SvgBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<SvgBackend>()
}

impl unibrowser_engine::document::DocumentBackend for SvgBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "image/svg+xml"
    }
    fn display_name(&self) -> &str {
        "SVG vector"
    }
    fn mode(&self) -> Mode {
        Mode::Vector
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            font_size: self.text_element_count > 0,
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot {
            domains: if self.text_selected {
                vec![SelectionDomain::Text, SelectionDomain::Path]
            } else {
                vec![SelectionDomain::Path]
            },
            text_length: if self.text_selected { 4 } else { 0 },
            has_text_selection: self.text_selected,
            font_size_pt: Some(self.font_size_pt),
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        if self.text_element_count == 0 {
            return Err("SVG has no <text> elements".into());
        }
        self.content = adjust_svg_text_font_sizes(&self.content, 1.0);
        self.font_size_pt = (self.font_size_pt + 1.0).min(96.0);
        self.dirty = true;
        Ok(self.font_size_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        if self.text_element_count == 0 {
            return Err("SVG has no <text> elements".into());
        }
        self.content = adjust_svg_text_font_sizes(&self.content, -1.0);
        self.font_size_pt = (self.font_size_pt - 1.0).max(6.0);
        self.dirty = true;
        Ok(self.font_size_pt)
    }
    fn status_line(&self) -> String {
        let mark = if self.dirty { " · unsaved" } else { "" };
        format!(
            "SVG {}×{} · ~{} elements · {} text · font {:.1} pt · zoom {:.0}%{}",
            self.width,
            self.height,
            self.element_count,
            self.text_element_count,
            self.font_size_pt,
            self.preview_zoom * 100.0,
            mark
        )
    }
    fn set_demo_text_selection(&mut self, selected: bool) {
        self.text_selected = selected && self.text_element_count > 0;
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn is_dirty(&self) -> bool {
        self.dirty
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
            reason: "SVG has no <text> or <tspan> elements".into(),
        };
    }
    Enablement::Enabled
}

fn exec_increase(doc: &mut DocumentSession) -> Result<String, String> {
    let pt = doc.backend.increase_font_size()?;
    Ok(format!("SVG text font: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("SVG text font: {:.1} pt", pt))
}

fn find_enable(doc: &DocumentSession) -> Enablement {
    font_enable(doc)
}

fn export_pdf_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Export PDF uses printable HTML (text, Markdown, or DOCX preview)".into(),
    }
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
    let b = try_svg_backend(&mut *doc.backend).ok_or("Not an SVG document")?;
    b.zoom_to_fit();
    Ok(format!("SVG zoom {:.0}%", b.preview_zoom * 100.0))
}

fn save_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

fn exec_save(doc: &mut DocumentSession) -> Result<String, String> {
    let backend = try_svg_backend(&mut *doc.backend).ok_or("Not an SVG document")?;
    backend.save()?;
    Ok("Saved SVG".into())
}

fn advanced_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Advanced path tools not implemented (use fill/stroke panel)".into(),
    }
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: commands::SVG_PATH_BOOLEAN,
        title: "Path boolean (not available)",
        icon: "path",
        default_keybinding: None,
        aliases: &["union", "subtract paths"],
    });
    registry.register_command(CommandMeta {
        id: commands::SVG_PATH_SIMPLIFY,
        title: "Simplify path (not available)",
        icon: "path",
        default_keybinding: None,
        aliases: &["reduce nodes"],
    });
    registry.register_handler(HandlerRegistration {
        command_id: commands::SVG_PATH_BOOLEAN.into(),
        backend_id: BACKEND_ID.into(),
        enable: advanced_disabled,
        execute: |_| Err("disabled".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: commands::SVG_PATH_SIMPLIFY.into(),
        backend_id: BACKEND_ID.into(),
        enable: advanced_disabled,
        execute: |_| Err("disabled".into()),
    });
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
        command_id: EXPORT_PRINT_PDF_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: export_pdf_disabled,
        execute: |_| Err("disabled".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: PAGE_ZOOM_FIT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: zoom_fit_enable,
        execute: exec_zoom_fit,
    });
    registry.register_handler(HandlerRegistration {
        command_id: commands::TEXT_SAVE.into(),
        backend_id: BACKEND_ID.into(),
        enable: save_enable,
        execute: exec_save,
    });
    registry.register_handler(HandlerRegistration {
        command_id: DOCUMENT_UNDO_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: |_doc| Ok("document-undo".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: DOCUMENT_REDO_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: |_doc| Ok("document-redo".into()),
    });
}

pub fn open_file(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(SvgBackend::from_path(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use unibrowser_engine::document::DocumentSession;
    use unibrowser_mod_api::handlers::FONT_INCREASE_ID;

    #[test]
    fn font_increase_scales_text_font_size_attribute() {
        let dir = std::env::temp_dir().join("unibrowser_svg_font_test");
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let file = dir.join("label.svg");
        std::fs::File::create(&file)
            .and_then(|mut f| {
                f.write_all(
                    br#"<svg width="100" height="50"><text font-size="12">Hi</text></svg>"#,
                )
            })
            .expect("write");
        let mut doc = DocumentSession::new(
            file.to_string_lossy().as_ref(),
            open_file(&file),
        );
        let mut reg = CommandRegistry::default();
        register(&mut reg);
        reg.execute(FONT_INCREASE_ID, &mut doc).expect("font");
        let svg = try_svg_backend(&mut *doc.backend).expect("svg");
        assert!(svg.content.contains("font-size=\"13"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
