use serde::{Deserialize, Serialize};

/// High-level UI mode (SOLIDWORKS-style context).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Browse,
    Text,
    Image,
    Vector,
    Document,
    Web,
}

impl Mode {
    pub fn tab_id(self) -> &'static str {
        match self {
            Mode::Browse => "browse",
            Mode::Text => "text",
            Mode::Image => "image",
            Mode::Vector => "vector",
            Mode::Document => "document",
            Mode::Web => "web",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Browse => "Browse",
            Mode::Text => "Text",
            Mode::Image => "Image",
            Mode::Vector => "Vector",
            Mode::Document => "Document",
            Mode::Web => "Web",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionDomain {
    Text,
    Path,
    Bitmap,
    Timecode,
    Cell,
    Node,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionSnapshot {
    pub domains: Vec<SelectionDomain>,
    pub text_length: usize,
    pub has_text_selection: bool,
    pub font_size_pt: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    pub font_size: bool,
}

#[derive(Debug, Clone)]
pub struct CommandMeta {
    pub id: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
    pub default_keybinding: Option<&'static str>,
    pub aliases: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Enablement {
    Enabled,
    Disabled { reason: String },
}

impl Enablement {
    pub fn is_enabled(&self) -> bool {
        matches!(self, Enablement::Enabled)
    }

    pub fn disabled_reason(&self) -> Option<&str> {
        match self {
            Enablement::Enabled => None,
            Enablement::Disabled { reason } => Some(reason.as_str()),
        }
    }
}

/// Read-only host view for enablement predicates.
#[derive(Debug, Clone)]
pub struct HostContext {
    pub mode: Mode,
    pub doc_type: String,
    pub backend_id: String,
    pub selection: SelectionSnapshot,
    pub capabilities: Capabilities,
}

/// Well-known command IDs.
pub mod commands {
    pub const FONT_SIZE_INCREASE: &str = "unibrowser.font.size.increase";
    pub const FONT_SIZE_DECREASE: &str = "unibrowser.font.size.decrease";
}
