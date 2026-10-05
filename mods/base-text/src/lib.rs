use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "base-text";

pub struct TextBackend {
    pub content: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub font_size_pt: f32,
}

impl Default for TextBackend {
    fn default() -> Self {
        Self {
            content: "Hello Unibrowser".into(),
            selection_start: 0,
            selection_end: 5,
            font_size_pt: 12.0,
        }
    }
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
        format!(
            "Text · {} pt · selection {}:{}",
            self.font_size_pt,
            self.selection_start,
            self.selection_end
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
    if !doc.backend.selection().has_text_selection {
        return Enablement::Disabled {
            reason: "Select text first".into(),
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

pub fn open_sample(path: &str) -> Box<dyn unibrowser_engine::document::DocumentBackend> {
    Box::new(TextBackend {
        content: format!("Sample file: {}", path),
        selection_start: 0,
        selection_end: 6,
        font_size_pt: 12.0,
    })
}
