//! Command system shared types (ADR 0027).

use serde::{Deserialize, Serialize};

/// Capability availability for a control (orthogonal to selection state).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CapabilityState {
    Supported,
    Emulated { provider_id: String, note: String },
    Unsupported { reason: String },
    HiddenByUser,
    Locked { reason: String },
}

impl CapabilityState {
    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            CapabilityState::Supported | CapabilityState::Emulated { .. }
        )
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match self {
            CapabilityState::Unsupported { reason } | CapabilityState::Locked { reason } => {
                Some(reason.as_str())
            }
            CapabilityState::Emulated { note, .. } => Some(note.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionState {
    Off,
    On,
    Mixed,
    Indeterminate,
    Value,
    MixedValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlType {
    Button,
    Toggle,
    SplitButton,
    Dropdown,
    ValueStepper,
    Colour,
    Gallery,
    Segmented,
    MenuButton,
    IndicatorAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityVariant {
    Compact,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverflowPriority {
    Critical,
    Primary,
    Normal,
    Low,
    Custom,
}

/// Stable control placement identity.
pub mod control_ids {
    pub const TEXT_FONT: &str = "unibrowser.control.text.font";
    pub const TEXT_SIZE: &str = "unibrowser.control.text.size";
    pub const TEXT_BOLD: &str = "unibrowser.control.text.bold";
    pub const TEXT_ITALIC: &str = "unibrowser.control.text.italic";
    pub const TEXT_UNDERLINE: &str = "unibrowser.control.text.underline";
    pub const TEXT_STRIKETHROUGH: &str = "unibrowser.control.text.strikethrough";
    pub const TEXT_SUBSCRIPT: &str = "unibrowser.control.text.subscript";
    pub const TEXT_SUPERSCRIPT: &str = "unibrowser.control.text.superscript";
    pub const TEXT_COLOUR: &str = "unibrowser.control.text.colour";
    pub const EDITOR_FONT: &str = "unibrowser.control.editor.font";
    pub const EDITOR_SIZE: &str = "unibrowser.control.editor.size";
    pub const START_OPEN_FILE: &str = "unibrowser.control.start.openFile";
    pub const START_OPEN_FOLDER: &str = "unibrowser.control.start.openFolder";
    pub const START_NEW_PROJECT: &str = "unibrowser.control.start.newProject";
    pub const START_RECENT: &str = "unibrowser.control.start.recent";
    pub const PDF_SELECT: &str = "unibrowser.control.pdf.select";
    pub const PDF_NOTE: &str = "unibrowser.control.pdf.note";
    pub const PDF_AREA: &str = "unibrowser.control.pdf.area";
    pub const PDF_PAGE: &str = "unibrowser.control.pdf.page";
    pub const PDF_VIEW_ZOOM: &str = "unibrowser.control.pdf.viewZoom";
    pub const PDF_EXPORT: &str = "unibrowser.control.pdf.export";
    pub const BROWSE_NEW_FOLDER: &str = "unibrowser.control.browse.newFolder";
    pub const BROWSE_NEW_TEXT_FILE: &str = "unibrowser.control.browse.newTextFile";
    pub const BROWSE_OPEN: &str = "unibrowser.control.browse.open";
    pub const BROWSE_COPY_PATHS: &str = "unibrowser.control.browse.copyPaths";
    pub const BROWSE_COPY_TO: &str = "unibrowser.control.browse.copyTo";
    pub const BROWSE_MOVE_TO: &str = "unibrowser.control.browse.moveTo";
    pub const BROWSE_RENAME: &str = "unibrowser.control.browse.rename";
    pub const BROWSE_COMPRESS: &str = "unibrowser.control.browse.compress";
    pub const BROWSE_PROPERTIES: &str = "unibrowser.control.browse.properties";
    pub const BROWSE_DELETE: &str = "unibrowser.control.browse.delete";
    pub const BROWSE_SORT: &str = "unibrowser.control.browse.sort";
    pub const BROWSE_SORT_DIRECTION: &str = "unibrowser.control.browse.sortDirection";
    pub const BROWSE_SHOW_HIDDEN: &str = "unibrowser.control.browse.showHidden";
    pub const BROWSE_SET_WORKSPACE: &str = "unibrowser.control.browse.setWorkspace";
    pub const BROWSE_CLEAR_WORKSPACE: &str = "unibrowser.control.browse.clearWorkspace";
    pub const SETTINGS_SEARCH: &str = "unibrowser.control.settings.search";
    pub const SETTINGS_KEYBINDINGS: &str = "unibrowser.control.settings.keybindings";
    pub const SETTINGS_DIAGNOSTICS: &str = "unibrowser.control.settings.diagnostics";
}

pub mod family_ids {
    pub const TEXT_FORMATTING: &str = "unibrowser.family.text-formatting";
    pub const EDITOR_APPEARANCE: &str = "unibrowser.family.editor-appearance";
    pub const HISTORY: &str = "unibrowser.family.history";
    pub const FIND: &str = "unibrowser.family.find";
    pub const PROOFING: &str = "unibrowser.family.proofing";
    pub const EXPORT: &str = "unibrowser.family.export";
    pub const PDF_ANNOTATION: &str = "unibrowser.family.pdf-annotation";
    pub const IMAGE_ADJUST: &str = "unibrowser.family.image-adjust";
    pub const START: &str = "unibrowser.family.start";
    pub const BROWSE: &str = "unibrowser.family.browse";
    pub const SETTINGS: &str = "unibrowser.family.settings";
    pub const PROJECT: &str = "unibrowser.family.project";
}

pub mod group_ids {
    pub const TEXT_TYPE: &str = "unibrowser.group.text.type";
    pub const TEXT_EMPHASIS: &str = "unibrowser.group.text.emphasis";
    pub const TEXT_BASELINE: &str = "unibrowser.group.text.baseline";
}

/// Document/work context for capability defaults (finer than Command Manager tab).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkContext {
    Start,
    PlainText,
    Markdown,
    RichDocument,
    CodeEditor,
    Pdf,
    Image,
    Cad3d,
    Project,
    Browse,
    Settings,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedControl {
    pub control_id: String,
    pub control_type: ControlType,
    pub command_ids: Vec<String>,
    pub capability: CapabilityState,
    pub selection: SelectionState,
    pub value: Option<String>,
    pub label: String,
    pub short_label: Option<String>,
    pub icon: String,
    pub shortcut: Option<String>,
    pub density_variant: DensityVariant,
    pub provider_id: Option<String>,
    pub context_revision: u64,
    pub group_id: String,
    pub family_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedGroup {
    pub group_id: String,
    pub label: String,
    pub overflow_priority: OverflowPriority,
    pub controls: Vec<ResolvedControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFamily {
    pub family_id: String,
    pub label: String,
    pub overflow_priority: OverflowPriority,
    pub groups: Vec<ResolvedGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCommandManagerTab {
    pub tab_id: String,
    pub label: String,
    pub families: Vec<ResolvedFamily>,
}

/// Provider candidate considered during resolution (CMD-02 diagnostics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCandidate {
    pub provider_id: String,
    pub source: ProviderSource,
    pub priority: i32,
    pub dependency_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSource {
    UserSelected,
    Backend,
    Mod,
    Emulation,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResolution {
    pub winner: Option<String>,
    pub reason: String,
    pub candidates: Vec<ProviderCandidate>,
}

/// Map legacy enablement to capability (CMD-01 adapter).
pub fn capability_from_enablement(en: &crate::Enablement) -> CapabilityState {
    match en {
        crate::Enablement::Enabled => CapabilityState::Supported,
        crate::Enablement::Disabled { reason } => CapabilityState::Unsupported {
            reason: reason.clone(),
        },
    }
}

pub fn enablement_from_capability(cap: &CapabilityState) -> crate::Enablement {
    match cap {
        CapabilityState::Supported | CapabilityState::Emulated { .. } => crate::Enablement::Enabled,
        CapabilityState::Unsupported { reason } | CapabilityState::Locked { reason } => {
            crate::Enablement::Disabled {
                reason: reason.clone(),
            }
        }
        CapabilityState::HiddenByUser => crate::Enablement::Disabled {
            reason: "Hidden by customisation".into(),
        },
    }
}
