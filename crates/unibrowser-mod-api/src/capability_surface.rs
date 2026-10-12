//! Document surface capability ids and manifest entries (ADR 0030, SKEL-01).

use serde::{Deserialize, Serialize};

use crate::content_kind::validate_content_kind_id;
use crate::ContentKindRegistry;

pub const CAPABILITY_SCHEMA_VERSION: &str = "1.0.0";

pub mod ids {
    pub const OPEN_TEXT: &str = "unibrowser.capability.open_text";
    pub const RENDER_PDF: &str = "unibrowser.capability.render_pdf";
    pub const RENDER_IMAGE: &str = "unibrowser.capability.render_image";
    pub const RENDER_SVG: &str = "unibrowser.capability.render_svg";
    pub const OPEN_DOCX: &str = "unibrowser.capability.open_docx";
    pub const OPEN_DXF: &str = "unibrowser.capability.open_dxf";
    pub const OPEN_VIDEO: &str = "unibrowser.capability.open_video";
    pub const OPEN_AUDIO: &str = "unibrowser.capability.open_audio";
    pub const OPEN_EPUB: &str = "unibrowser.capability.open_epub";
    pub const OPEN_CSV: &str = "unibrowser.capability.open_csv";
    pub const OPEN_SCRIPT: &str = "unibrowser.capability.open_script";
    pub const PROJECT_SURFACE: &str = "unibrowser.capability.project_surface";
    pub const BROWSE_FILES: &str = "unibrowser.capability.browse_files";
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityProvideEntry {
    /// Stable capability contract id (`unibrowser.capability.*`).
    pub capability: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub content_kinds: Vec<String>,
}

pub fn validate_capability_id(id: &str) -> Result<(), String> {
    if !id.starts_with("unibrowser.capability.") {
        return Err(format!("capability id {} must use unibrowser.capability.*", id));
    }
    if id.len() <= "unibrowser.capability.".len() {
        return Err(format!("capability id {} is incomplete", id));
    }
    Ok(())
}

pub fn validate_capability_provide(
    mod_name: &str,
    entry: &CapabilityProvideEntry,
    kinds: &ContentKindRegistry,
) -> Result<(), String> {
    validate_capability_id(&entry.capability)?;
    let _ = mod_name;
    for kind in &entry.content_kinds {
        kinds.validate_reference(kind)?;
        validate_content_kind_id(kind, None)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_core_capability_namespace() {
        assert!(validate_capability_id("acme.open_text").is_err());
    }
}
