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
