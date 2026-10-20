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
    pub const PAGE_ZOOM_FIT: &str = "unibrowser.page.zoom.fit";
    pub const FIND_IN_DOCUMENT: &str = "unibrowser.find.inDocument";
    pub const DOCUMENT_UNDO: &str = "unibrowser.document.undo";
    pub const DOCUMENT_REDO: &str = "unibrowser.document.redo";
    pub const HISTORY_TRIM_BEFORE: &str = "unibrowser.history.trimBefore";
    pub const HISTORY_SQUASH_SELECTION: &str = "unibrowser.history.squashSelection";
    pub const FORMAT_DOCUMENT: &str = "unibrowser.editor.formatDocument";
    pub const EXPORT_PRINT_PDF: &str = "unibrowser.export.printPdf";
    pub const PRINT_SELECTION: &str = "unibrowser.export.printSelection";
    pub const TOGGLE_WEB_DEVTOOLS: &str = "unibrowser.web.devtools.toggle";
    pub const TOGGLE_TERMINAL_PANEL: &str = "unibrowser.terminal.toggle";
    pub const COPY_TEXT_FROM_IMAGE: &str = "unibrowser.image.ocr.copy";
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
    pub const PDF_ANNOTATE_UNDERLINE: &str = "unibrowser.pdf.annotate.underline";
    pub const PDF_ANNOTATE_STRIKETHROUGH: &str = "unibrowser.pdf.annotate.strikethrough";
    pub const PDF_ANNOTATE_STAMP: &str = "unibrowser.pdf.annotate.stamp";
    pub const PDF_ANNOTATE_INK: &str = "unibrowser.pdf.annotate.ink";
    pub const PDF_ANNOTATE_TEXT_BOX: &str = "unibrowser.pdf.annotate.textBox";
    pub const PDF_TOOL_SELECT: &str = "unibrowser.pdf.tool.select";
    pub const PDF_TOOL_HAND: &str = "unibrowser.pdf.tool.hand";
    pub const PDF_ANNOTATE_NOTE: &str = "unibrowser.pdf.annotate.note";
    pub const PDF_ANNOTATE_AREA: &str = "unibrowser.pdf.annotate.area";
    pub const PDF_ANNOTATION_PREVIOUS: &str = "unibrowser.pdf.annotation.previous";
    pub const PDF_ANNOTATION_NEXT: &str = "unibrowser.pdf.annotation.next";
    pub const PDF_STROKE_COLOUR: &str = "unibrowser.pdf.appearance.strokeColour";
    pub const PDF_FILL_COLOUR: &str = "unibrowser.pdf.appearance.fillColour";
    pub const PDF_STROKE_WIDTH: &str = "unibrowser.pdf.appearance.strokeWidth";
    pub const PDF_PAGE_NAV: &str = "unibrowser.pdf.page.nav";
    pub const PDF_VIEW_ZOOM: &str = "unibrowser.pdf.view.zoom";
    pub const IMAGE_FIT: &str = "unibrowser.image.fit";
    pub const IMAGE_CROP: &str = "unibrowser.image.crop";
    pub const IMAGE_ROTATE: &str = "unibrowser.image.rotate";
    pub const IMAGE_RESIZE: &str = "unibrowser.image.resize";
    pub const IMAGE_EXPOSURE: &str = "unibrowser.image.exposure";
    pub const IMAGE_CONTRAST: &str = "unibrowser.image.contrast";
    pub const IMAGE_COLOUR: &str = "unibrowser.image.colour";
    pub const IMAGE_ACTUAL_SIZE: &str = "unibrowser.image.actualSize";
    pub const IMAGE_FLIP_HORIZONTAL: &str = "unibrowser.image.flipHorizontal";
    pub const IMAGE_FLIP_VERTICAL: &str = "unibrowser.image.flipVertical";
    pub const IMAGE_HISTOGRAM: &str = "unibrowser.image.histogram";
    pub const IMAGE_EYEDROPPER: &str = "unibrowser.image.eyedropper";
    pub const IMAGE_COPY: &str = "unibrowser.image.copy";
    pub const START_OPEN_FILE: &str = "unibrowser.start.openFile";
    pub const START_OPEN_FOLDER: &str = "unibrowser.start.openFolder";
    pub const START_NEW_PROJECT: &str = "unibrowser.start.newProject";
    pub const START_RECENT: &str = "unibrowser.start.recent";
    pub const SVG_PATH_BOOLEAN: &str = "unibrowser.svg.path.boolean";
    pub const SVG_PATH_SIMPLIFY: &str = "unibrowser.svg.path.simplify";
    pub const PROOFING_TOGGLE: &str = "unibrowser.proofing.toggle";
    pub const PROOFING_NEXT: &str = "unibrowser.proofing.nextProblem";
    pub const PROOFING_PREVIOUS: &str = "unibrowser.proofing.previousProblem";
    pub const BROWSE_NEW_FOLDER: &str = "unibrowser.browse.newFolder";
    pub const BROWSE_NEW_TEXT_FILE: &str = "unibrowser.browse.newTextFile";
    pub const BROWSE_OPEN: &str = "unibrowser.browse.open";
    pub const BROWSE_COPY_PATHS: &str = "unibrowser.browse.copyPaths";
    pub const BROWSE_COPY_TO: &str = "unibrowser.browse.copyTo";
    pub const BROWSE_MOVE_TO: &str = "unibrowser.browse.moveTo";
    pub const BROWSE_RENAME: &str = "unibrowser.browse.rename";
    pub const BROWSE_COMPRESS: &str = "unibrowser.browse.compress";
    pub const BROWSE_PROPERTIES: &str = "unibrowser.browse.properties";
    pub const BROWSE_DELETE: &str = "unibrowser.browse.delete";
    pub const BROWSE_SORT: &str = "unibrowser.browse.sort";
    pub const BROWSE_SORT_DIRECTION: &str = "unibrowser.browse.sortDirection";
    pub const BROWSE_SHOW_HIDDEN: &str = "unibrowser.browse.showHidden";
    pub const BROWSE_SET_WORKSPACE: &str = "unibrowser.browse.setWorkspace";
    pub const BROWSE_CLEAR_WORKSPACE: &str = "unibrowser.browse.clearWorkspace";
    pub const MODS_SEARCH: &str = "unibrowser.mods.search";
    pub const MODS_INSTALL: &str = "unibrowser.mods.install";
    pub const MODS_UPDATES: &str = "unibrowser.mods.updates";
    pub const MODS_PERMISSIONS: &str = "unibrowser.mods.permissions";
    pub const MODS_OPEN_VSX: &str = "unibrowser.mods.openVsx";
    pub const SETTINGS_SEARCH: &str = "unibrowser.settings.search";
    pub const SETTINGS_KEYBINDINGS: &str = "unibrowser.settings.keybindings";
    pub const SETTINGS_DIAGNOSTICS: &str = "unibrowser.settings.diagnostics";
    pub const RESTORE_AUTOSAVE: &str = "unibrowser.autosave.restore";
    pub const TOGGLE_PROBLEMS_PANEL: &str = "unibrowser.problems.toggle";
    pub const TEXT_BOLD: &str = "unibrowser.text.bold";
    pub const TEXT_ITALIC: &str = "unibrowser.text.italic";
    pub const TEXT_UNDERLINE: &str = "unibrowser.text.underline";
    pub const TEXT_STRIKETHROUGH: &str = "unibrowser.text.strikethrough";
    pub const TEXT_SUBSCRIPT: &str = "unibrowser.text.baseline.subscript";
    pub const TEXT_SUPERSCRIPT: &str = "unibrowser.text.baseline.superscript";
    pub const TEXT_COLOUR: &str = "unibrowser.text.colour";
    pub const TEXT_CODE_INLINE: &str = "unibrowser.text.code_inline";
    pub const TEXT_LINK: &str = "unibrowser.text.link";
    pub const TEXT_CLEAR: &str = "unibrowser.text.clear";
    pub const TEXT_LIST_BULLET: &str = "unibrowser.list.bullet";
    pub const TEXT_LIST_NUMBERED: &str = "unibrowser.list.numbered";
    pub const TEXT_INSERT_IMAGE: &str = "unibrowser.insert.image";
    pub const TEXT_COMPACT_OVERFLOW: &str = "unibrowser.text.compact.overflow";
    pub const SEMANTIC_BLOCK_PARAGRAPH: &str = "unibrowser.semantic.block.paragraph";
    pub const SEMANTIC_BLOCK_HEADING_1: &str = "unibrowser.semantic.block.heading_1";
    pub const SEMANTIC_BLOCK_HEADING_2: &str = "unibrowser.semantic.block.heading_2";
    pub const SEMANTIC_BLOCK_HEADING_3: &str = "unibrowser.semantic.block.heading_3";
    pub const SEMANTIC_BLOCK_HEADING_4: &str = "unibrowser.semantic.block.heading_4";
    pub const SEMANTIC_BLOCK_HEADING_5: &str = "unibrowser.semantic.block.heading_5";
    pub const SEMANTIC_BLOCK_HEADING_6: &str = "unibrowser.semantic.block.heading_6";
    pub const SEMANTIC_BLOCK_BLOCKQUOTE: &str = "unibrowser.semantic.block.blockquote";
    pub const SEMANTIC_BLOCK_CODE: &str = "unibrowser.semantic.block.code_block";
    pub const TEXT_FONT_FAMILY: &str = "unibrowser.text.fontFamily";
    pub const TEXT_SIZE_SET: &str = "unibrowser.text.size.set";
    pub const EDITOR_FONT_FAMILY: &str = "unibrowser.editor.fontFamily";
    pub const EDITOR_FONT_SIZE: &str = "unibrowser.editor.fontSize";
}
