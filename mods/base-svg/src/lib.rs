use std::fs;
use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot, commands,
    handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-svg";
pub const VIEWER_KIND: &str = "svg";

pub struct SvgBackend {
    pub path: String,
    pub content: String,
    pub width: u32,
    pub height: u32,
    pub element_count: usize,
    pub text_selected: bool,
    pub font_size_pt: f32,
    pub dirty: bool,
}

impl SvgBackend {
    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let content = fs::read_to_string(path).unwrap_or_default();
        let (width, height, element_count) = parse_svg_metadata(&content);
        Self {
            path: path_str,
            content,
            width,
            height,
            element_count,
            text_selected: element_count > 0,
            font_size_pt: 14.0,
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
            font_size: self.element_count > 0,
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
        self.font_size_pt = (self.font_size_pt + 1.0).min(96.0);
        Ok(self.font_size_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.font_size_pt = (self.font_size_pt - 1.0).max(6.0);
        Ok(self.font_size_pt)
    }
    fn status_line(&self) -> String {
        let mark = if self.dirty { " · unsaved" } else { "" };
        format!(
            "SVG {}×{} · ~{} elements · font {:.1} pt{}",
            self.width,
            self.height,
            self.element_count,
            self.font_size_pt,
            mark
        )
    }
    fn set_demo_text_selection(&mut self, selected: bool) {
        self.text_selected = selected;
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
    if !doc.backend.selection().has_text_selection {
        return Enablement::Disabled {
            reason: "Select vector text".into(),
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
        command_id: commands::TEXT_SAVE.into(),
        backend_id: BACKEND_ID.into(),
        enable: save_enable,
        execute: exec_save,
    });
}

pub fn open_file(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(SvgBackend::from_path(path))
}
