use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot, commands,
    handlers::{FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-text";

pub struct TextBackend {
    pub path: String,
    pub content: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub font_size_pt: f32,
    pub dirty: bool,
}

impl Default for TextBackend {
    fn default() -> Self {
        Self {
            path: String::new(),
            content: "Hello Unibrowser".into(),
            selection_start: 0,
            selection_end: 5,
            font_size_pt: 12.0,
            dirty: false,
        }
    }
}

impl TextBackend {
    pub fn open_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read file: {}", e))?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            content,
            selection_start: 0,
            selection_end: 0,
            font_size_pt: 12.0,
            dirty: false,
        })
    }

    pub fn set_content(&mut self, content: String) {
        if self.content != content {
            self.content = content;
            self.dirty = true;
        }
    }

    pub fn set_selection_range(&mut self, start: usize, end: usize) {
        let len = self.content.len();
        self.selection_start = start.min(len);
        self.selection_end = end.min(len);
        if self.selection_end < self.selection_start {
            self.selection_end = self.selection_start;
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        let path = Path::new(&self.path);
        if self.path.is_empty() {
            return Err("No file path for save".into());
        }
        std::fs::write(path, &self.content).map_err(|e| format!("Save failed: {}", e))?;
        self.dirty = false;
        Ok(())
    }

    pub fn language_id(path: &str) -> &'static str {
        match Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
        {
            Some(ext) if ext == "md" || ext == "markdown" => "markdown",
            Some(ext) if ext == "rs" => "rust",
            Some(ext) if ext == "json" => "json",
            Some(ext) if ext == "ts" || ext == "tsx" => "typescript",
            Some(ext) if ext == "js" || ext == "jsx" => "javascript",
            Some(ext) if ext == "html" || ext == "htm" => "html",
            Some(ext) if ext == "css" => "css",
            Some(ext) if ext == "toml" => "toml",
            Some(ext) if ext == "yaml" || ext == "yml" => "yaml",
            Some(ext) if ext == "xml" => "xml",
            _ => "plaintext",
        }
    }
}

pub fn try_text_backend(backend: &mut dyn unibrowser_engine::document::DocumentBackend) -> Option<&mut TextBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<TextBackend>()
}

impl unibrowser_engine::document::DocumentBackend for TextBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "text/plain"
    }
    fn display_name(&self) -> &str {
        "Plain text"
    }
    fn mode(&self) -> Mode {
        Mode::Text
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities { font_size: true }
    }
    fn selection(&self) -> SelectionSnapshot {
        let has = self.selection_end > self.selection_start;
        SelectionSnapshot {
            domains: if has {
                vec![SelectionDomain::Text]
            } else {
                vec![]
            },
            text_length: if has {
                self.selection_end - self.selection_start
            } else {
                0
            },
            has_text_selection: has,
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
        let dirty = if self.dirty { " · modified" } else { "" };
        format!(
            "Text · {} pt · selection {}:{}{}",
            self.font_size_pt,
            self.selection_start,
            self.selection_end,
            dirty
        )
    }
    fn set_demo_text_selection(&mut self, selected: bool) {
        if selected {
            self.selection_start = 0;
            self.selection_end = self.content.len().min(6);
        } else {
            self.selection_start = 0;
            self.selection_end = 0;
        }
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
            reason: "Format does not support font size".into(),
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
    Ok(format!("Font size: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let en = font_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("Font size: {:.1} pt", pt))
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
    let backend = try_text_backend(&mut *doc.backend).ok_or("Not a text document")?;
    backend.save()?;
    Ok("Saved".into())
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(unibrowser_mod_api::CommandMeta {
        id: commands::TEXT_SAVE,
        title: "Save",
        icon: "save",
        default_keybinding: Some("Ctrl+S"),
        aliases: &["write file", "save file"],
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
        command_id: commands::TEXT_SAVE.into(),
        backend_id: BACKEND_ID.into(),
        enable: save_enable,
        execute: exec_save,
    });
    registry.register_handler(HandlerRegistration {
        command_id: FIND_IN_DOCUMENT_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: font_enable,
        execute: |_doc| Ok("document-find".into()),
    });
}

pub fn open_sample(path: &str) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    let path_buf = Path::new(path);
    if path_buf.is_file() {
        if let Ok(backend) = TextBackend::open_path(path_buf) {
            return Box::new(backend);
        }
    }
    Box::new(TextBackend {
        path: path.to_string(),
        content: format!("Sample file: {}", path),
        selection_start: 0,
        selection_end: 6,
        font_size_pt: 12.0,
        dirty: false,
    })
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(TextBackend::open_path(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn round_trip_save() {
        let dir = std::env::temp_dir().join("unibrowser_base_text_test");
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let file = dir.join("note.txt");
        std::fs::File::create(&file)
            .and_then(|mut f| f.write_all(b"alpha"))
            .expect("write");
        let mut backend = TextBackend::open_path(&file).expect("open");
        backend.set_content("beta".into());
        backend.save().expect("save");
        let again = std::fs::read_to_string(&file).expect("read");
        assert_eq!(again, "beta");
        let _ = std::fs::remove_dir_all(dir);
    }
}
