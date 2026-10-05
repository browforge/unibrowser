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
    pub const TEXT_SAVE: &str = "unibrowser.text.save";
    pub const SCRIPT_DEMO_HELLO: &str = "unibrowser.script.demo.hello";
    pub const PAGE_ZOOM_INCREASE: &str = "unibrowser.page.zoom.increase";
    pub const PAGE_ZOOM_DECREASE: &str = "unibrowser.page.zoom.decrease";
    pub const PAGE_ZOOM_RESET_ALL: &str = "unibrowser.page.zoom.resetAll";
    pub const FIND_IN_DOCUMENT: &str = "unibrowser.find.inDocument";
    pub const DOCUMENT_UNDO: &str = "unibrowser.document.undo";
    pub const DOCUMENT_REDO: &str = "unibrowser.document.redo";
    pub const EXPORT_PRINT_PDF: &str = "unibrowser.export.printPdf";
    pub const PRINT_SELECTION: &str = "unibrowser.export.printSelection";
    pub const TOGGLE_WEB_DEVTOOLS: &str = "unibrowser.web.devtools.toggle";
    pub const NEW_WINDOW: &str = "unibrowser.window.new";
    pub const SHOW_KEYBOARD_SHORTCUTS: &str = "unibrowser.keys.showOverlay";
    pub const TOGGLE_READER_MODE: &str = "unibrowser.web.reader.toggle";
    pub const CAPTURE_VISIBLE_TAB: &str = "unibrowser.capture.visibleTab";
    pub const COMPARE_IMAGES: &str = "unibrowser.image.compare";
    pub const COMPARE_TEXT: &str = "unibrowser.text.compare";
    pub const GIT_COMMIT: &str = "unibrowser.git.commit";
    pub const GIT_PUSH: &str = "unibrowser.git.push";
    pub const GIT_SYNC: &str = "unibrowser.git.sync";
    pub const GO_TO_TAB: &str = "unibrowser.tabs.goTo";
    pub const DUPLICATE_TAB: &str = "unibrowser.tabs.duplicate";
    pub const CLOSE_OTHER_TABS: &str = "unibrowser.tabs.closeOthers";
    pub const CLOSE_TABS_TO_RIGHT: &str = "unibrowser.tabs.closeToRight";
    pub const REOPEN_CLOSED_TAB: &str = "unibrowser.tabs.reopenClosed";
    pub const TOGGLE_TAB_MUTE: &str = "unibrowser.tabs.mute.toggle";
    pub const IMAGE_CURVES: &str = "unibrowser.image.curves";
    pub const PICTURE_IN_PICTURE: &str = "unibrowser.media.pictureInPicture";
    pub const CONTEXT_HELP: &str = "unibrowser.help.context";
    pub const WORKSPACE_QUICK_OPEN: &str = "unibrowser.workspace.quickOpen";
    pub const FIND_FILES_DEEP: &str = "unibrowser.find.filesDeep";
    pub const MIDI_RENDER_AUDIO: &str = "unibrowser.midi.renderAudio";
    pub const PDF_ANNOTATE_HIGHLIGHT: &str = "unibrowser.pdf.annotate.highlight";
    pub const PDF_ANNOTATE_STAMP: &str = "unibrowser.pdf.annotate.stamp";
    pub const PDF_ANNOTATE_INK: &str = "unibrowser.pdf.annotate.ink";
    pub const SVG_PATH_BOOLEAN: &str = "unibrowser.svg.path.boolean";
    pub const SVG_PATH_SIMPLIFY: &str = "unibrowser.svg.path.simplify";
}
