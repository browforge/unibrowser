use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_engine::document::DocumentBackend;
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-hex";
pub const VIEWER_KIND: &str = "text";

const BYTES_PER_LINE: usize = 16;

pub struct HexBackend {
    pub path: String,
    pub content: String,
    pub font_size_pt: f32,
}

impl HexBackend {
    pub fn open_path(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("Could not read file: {}", e))?;
        let content = format_hex_dump(&bytes);
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            content,
            font_size_pt: 12.0,
        })
    }

    pub fn open_path_as_text(path: &Path) -> Result<base_text::TextBackend, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read as text: {}", e))?;
        Ok(base_text::TextBackend {
            path: path.to_string_lossy().into_owned(),
            content: raw,
            selection_start: 0,
            selection_end: 0,
            font_size_pt: 12.0,
            dirty: false,
        })
    }
}

fn format_hex_dump(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (offset, chunk) in bytes.chunks(BYTES_PER_LINE).enumerate() {
        let off = offset * BYTES_PER_LINE;
        out.push_str(&format!("{:08x}  ", off));
        for (i, b) in chunk.iter().enumerate() {
            out.push_str(&format!("{:02x} ", b));
            if i == 7 {
                out.push(' ');
            }
        }
        let pad = (BYTES_PER_LINE - chunk.len()) * 3 + if chunk.len() <= 8 { 1 } else { 0 };
        for _ in 0..pad {
            out.push(' ');
        }
        out.push(' ');
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}

impl unibrowser_engine::document::DocumentBackend for HexBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/octet-stream"
    }
    fn display_name(&self) -> &str {
        "Hex dump"
    }
    fn mode(&self) -> Mode {
        Mode::Text
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            font_size: true,
        }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        self.font_size_pt = (self.font_size_pt + 1.0).min(48.0);
        Ok(self.font_size_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.font_size_pt = (self.font_size_pt - 1.0).max(8.0);
        Ok(self.font_size_pt)
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!("Hex view · {}", self.path)
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn try_hex_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut HexBackend> {
    backend.as_any_mut().downcast_mut()
}

pub fn open_file(path: &Path) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(HexBackend::open_path(path).expect("hex open"))
}

fn font_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() == BACKEND_ID {
        Enablement::Enabled
    } else {
        Enablement::Disabled {
            reason: "Font size applies in hex view only".into(),
        }
    }
}

fn font_increase(doc: &mut DocumentSession) -> Result<String, String> {
    let backend = try_hex_backend(&mut *doc.backend).ok_or("Not a hex view")?;
    let pt = backend.increase_font_size()?;
    Ok(format!("Font {} pt", pt))
}

fn font_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let backend = try_hex_backend(&mut *doc.backend).ok_or("Not a hex view")?;
    let pt = backend.decrease_font_size()?;
    Ok(format!("Font {} pt", pt))
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_handler(HandlerRegistration {
        command_id: FONT_INCREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: font_increase,
    });
    registry.register_handler(HandlerRegistration {
        command_id: FONT_DECREASE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: font_decrease,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_has_offset_and_ascii() {
        let s = format_hex_dump(&[0x41, 0x42, 0xff]);
        assert!(s.contains("00000000"));
        assert!(s.contains("41"));
        assert!(s.contains("AB"));
    }
}
