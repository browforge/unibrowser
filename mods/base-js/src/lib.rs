//! Monaco proving lane for JavaScript / TypeScript (syntax highlighting; Rhai run stays in palette).

use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot, commands,
    handlers::{
        DOCUMENT_REDO_ID, DOCUMENT_UNDO_ID, FIND_IN_DOCUMENT_ID, FONT_DECREASE_ID,
        FONT_INCREASE_ID, PRINT_SELECTION_ID,
    },
};

pub const BACKEND_ID: &str = "base-js";
/// Not used for a separate viewer — Monaco uses the text editor surface.
pub const VIEWER_KIND: &str = "script";

pub struct JsMonacoBackend {
    pub path: String,
    pub content: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub font_size_pt: f32,
    pub dirty: bool,
}

impl JsMonacoBackend {
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
        if self.path.is_empty() {
            return Err("No file path for save".into());
        }
        std::fs::write(Path::new(&self.path), &self.content)
            .map_err(|e| format!("Save failed: {}", e))?;
        self.dirty = false;
        Ok(())
    }

    pub fn language_id(path: &str) -> &'static str {
        match Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
        {
            Some(ext) if ext == "ts" || ext == "tsx" => "typescript",
            _ => "javascript",
        }
    }
}

pub fn try_js_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut JsMonacoBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<JsMonacoBackend>()
}

impl unibrowser_engine::document::DocumentBackend for JsMonacoBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "text/javascript"
    }
    fn display_name(&self) -> &str {
        "JavaScript / TypeScript"
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
        self.font_size_pt = (self.font_size_pt + 1.0).min(48.0);
        Ok(self.font_size_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.font_size_pt = (self.font_size_pt - 1.0).max(8.0);
        Ok(self.font_size_pt)
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        let lang = Self::language_id(&self.path);
        if self.dirty {
            format!("{} · {} (unsaved)", lang, self.path)
        } else {
            format!("{} · {}", lang, self.path)
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn font_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() == BACKEND_ID {
        Enablement::Enabled
    } else {
        Enablement::Disabled {
            reason: "Wrong backend".into(),
        }
    }
}

fn exec_increase(doc: &mut DocumentSession) -> Result<String, String> {
    let pt = doc.backend.increase_font_size()?;
    Ok(format!("Font size: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("Font size: {:.1} pt", pt))
}

fn save_enable(doc: &DocumentSession) -> Enablement {
    font_enable(doc)
}

fn exec_save(doc: &mut DocumentSession) -> Result<String, String> {
    let backend = try_js_backend(&mut *doc.backend).ok_or("Not a script document")?;
    backend.save()?;
    Ok("Saved".into())
}

fn undo_redo_enable(doc: &DocumentSession) -> Enablement {
    font_enable(doc)
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
    registry.register_handler(HandlerRegistration {
        command_id: DOCUMENT_UNDO_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: undo_redo_enable,
        execute: |_doc| Ok("document-undo".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: DOCUMENT_REDO_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: undo_redo_enable,
        execute: |_doc| Ok("document-redo".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: PRINT_SELECTION_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: print_selection_enable,
        execute: |_doc| Ok("export-print-selection".into()),
    });
}

fn print_selection_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    let sel = doc.backend.selection();
    if sel.has_text_selection && sel.text_length > 0 {
        Enablement::Enabled
    } else {
        Enablement::Disabled {
            reason: "Select text first".into(),
        }
    }
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(JsMonacoBackend::open_path(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_ids() {
        assert_eq!(JsMonacoBackend::language_id("a.ts"), "typescript");
        assert_eq!(JsMonacoBackend::language_id("a.tsx"), "typescript");
        assert_eq!(JsMonacoBackend::language_id("a.js"), "javascript");
    }
}
