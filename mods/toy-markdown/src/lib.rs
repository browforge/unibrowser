use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionDomain, SelectionSnapshot,
    handlers::{FONT_DECREASE_ID, FONT_INCREASE_ID},
};

pub const BACKEND_ID: &str = "toy-markdown";

pub struct MarkdownBackend {
    pub source: String,
    pub heading_font_pt: f32,
    pub heading_selected: bool,
}

impl Default for MarkdownBackend {
    fn default() -> Self {
        Self {
            source: "# Unibrowser\n\nSame command, different mod.".into(),
            heading_font_pt: 16.0,
            heading_selected: true,
        }
    }
}

impl unibrowser_engine::document::DocumentBackend for MarkdownBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "text/markdown"
    }
    fn display_name(&self) -> &str {
        "Markdown"
    }
    fn mode(&self) -> Mode {
        Mode::Text
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities { font_size: true }
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot {
            domains: if self.heading_selected {
                vec![SelectionDomain::Text]
            } else {
                vec![]
            },
            text_length: if self.heading_selected { 11 } else { 0 },
            has_text_selection: self.heading_selected,
            font_size_pt: Some(self.heading_font_pt),
        }
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        self.heading_font_pt = (self.heading_font_pt + 1.0).min(96.0);
        Ok(self.heading_font_pt)
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        self.heading_font_pt = (self.heading_font_pt - 1.0).max(6.0);
        Ok(self.heading_font_pt)
    }
    fn status_line(&self) -> String {
        format!("Markdown heading · {} pt", self.heading_font_pt)
    }
    fn set_demo_text_selection(&mut self, selected: bool) {
        self.heading_selected = selected;
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
    if !doc.backend.selection().has_text_selection {
        return Enablement::Disabled {
            reason: "Select heading text".into(),
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
    Ok(format!("Heading font: {:.1} pt", pt))
}

fn exec_decrease(doc: &mut DocumentSession) -> Result<String, String> {
    let en = font_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.backend.decrease_font_size()?;
    Ok(format!("Heading font: {:.1} pt", pt))
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
    let _ = path;
    Box::new(MarkdownBackend::default())
}
