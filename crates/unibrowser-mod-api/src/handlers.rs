use crate::{Enablement, commands};

/// Shared enablement for font-size commands (mods with `DocumentBackend` font support).
pub fn font_size_enable(doc: &dyn FontSizeHost) -> Enablement {
    if !doc.capabilities_font_size() {
        return Enablement::Disabled {
            reason: "This format does not support font size".into(),
        };
    }
    if !doc.selection_has_text() {
        return Enablement::Disabled {
            reason: "Select text first".into(),
        };
    }
    Enablement::Enabled
}

pub trait FontSizeHost {
    fn capabilities_font_size(&self) -> bool;
    fn selection_has_text(&self) -> bool;
    fn increase_font_size(&mut self) -> Result<f32, String>;
    fn decrease_font_size(&mut self) -> Result<f32, String>;
}

pub fn font_size_increase(doc: &mut dyn FontSizeHost) -> Result<String, String> {
    let en = font_size_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.increase_font_size()?;
    Ok(format!("Font size: {:.1} pt", pt))
}

pub fn font_size_decrease(doc: &mut dyn FontSizeHost) -> Result<String, String> {
    let en = font_size_enable(doc);
    if !en.is_enabled() {
        return Err(en.disabled_reason().unwrap_or("disabled").into());
    }
    let pt = doc.decrease_font_size()?;
    Ok(format!("Font size: {:.1} pt", pt))
}

pub const FONT_INCREASE_ID: &str = commands::FONT_SIZE_INCREASE;
pub const FONT_DECREASE_ID: &str = commands::FONT_SIZE_DECREASE;
pub const PAGE_ZOOM_INCREASE_ID: &str = commands::PAGE_ZOOM_INCREASE;
pub const PAGE_ZOOM_DECREASE_ID: &str = commands::PAGE_ZOOM_DECREASE;
pub const FIND_IN_DOCUMENT_ID: &str = commands::FIND_IN_DOCUMENT;
pub const DOCUMENT_UNDO_ID: &str = commands::DOCUMENT_UNDO;
pub const DOCUMENT_REDO_ID: &str = commands::DOCUMENT_REDO;
pub const EXPORT_PRINT_PDF_ID: &str = commands::EXPORT_PRINT_PDF;
pub const PRINT_SELECTION_ID: &str = commands::PRINT_SELECTION;
pub const TOGGLE_WEB_DEVTOOLS_ID: &str = commands::TOGGLE_WEB_DEVTOOLS;
pub const SHOW_KEYBOARD_SHORTCUTS_ID: &str = commands::SHOW_KEYBOARD_SHORTCUTS;
pub const TOGGLE_READER_MODE_ID: &str = commands::TOGGLE_READER_MODE;
pub const CAPTURE_VISIBLE_TAB_ID: &str = commands::CAPTURE_VISIBLE_TAB;
pub const COMPARE_IMAGES_ID: &str = commands::COMPARE_IMAGES;
pub const COMPARE_TEXT_ID: &str = commands::COMPARE_TEXT;
pub const PICTURE_IN_PICTURE_ID: &str = commands::PICTURE_IN_PICTURE;
pub const CONTEXT_HELP_ID: &str = commands::CONTEXT_HELP;
pub const PDF_ANNOTATE_HIGHLIGHT_ID: &str = commands::PDF_ANNOTATE_HIGHLIGHT;
pub const PDF_ANNOTATE_STAMP_ID: &str = commands::PDF_ANNOTATE_STAMP;
pub const PDF_ANNOTATE_INK_ID: &str = commands::PDF_ANNOTATE_INK;
