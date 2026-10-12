//! Versioned content-kind registry (ADR 0028, CMD-12).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CONTENT_KIND_SCHEMA_VERSION: &str = "1.0.0";

pub mod ids {
    pub const TEXT_PLAIN: &str = "unibrowser.content.text_plain";
    pub const TEXT_MARKDOWN: &str = "unibrowser.content.text_markdown";
    pub const TEXT_RICH: &str = "unibrowser.content.text_rich";
    pub const IMAGE: &str = "unibrowser.content.image";
    pub const PDF_PAGE: &str = "unibrowser.content.pdf_page";
    pub const VECTOR_CAD: &str = "unibrowser.content.vector_cad";
    pub const AUDIO_CLIP: &str = "unibrowser.content.audio_clip";
    pub const VIDEO_CLIP: &str = "unibrowser.content.video_clip";
    pub const PROJECT_GRAPH: &str = "unibrowser.content.project_graph";
    pub const TABULAR: &str = "unibrowser.content.tabular";
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentKindRecord {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ContentKindRegistry {
    kinds: BTreeMap<String, ContentKindRecord>,
}

impl ContentKindRegistry {
    pub fn with_core_kinds() -> Self {
        let mut reg = Self::default();
        reg.register_core(ids::TEXT_PLAIN, "Plain text", None);
        reg.register_core(ids::TEXT_MARKDOWN, "Markdown", Some(ids::TEXT_PLAIN));
        reg.register_core(ids::TEXT_RICH, "Rich text", Some(ids::TEXT_PLAIN));
        reg.register_core(ids::IMAGE, "Image", None);
        reg.register_core(ids::PDF_PAGE, "PDF page", None);
        reg.register_core(ids::VECTOR_CAD, "CAD vector", None);
        reg.register_core(ids::AUDIO_CLIP, "Audio", None);
        reg.register_core(ids::VIDEO_CLIP, "Video", None);
        reg.register_core(ids::PROJECT_GRAPH, "Project graph", None);
        reg.register_core(ids::TABULAR, "Tabular data", None);
        reg
    }

    fn register_core(&mut self, id: &str, label: &str, parent: Option<&str>) {
        self.kinds.insert(
            id.to_string(),
            ContentKindRecord {
                id: id.to_string(),
                label: label.to_string(),
                parent: parent.map(|p| p.to_string()),
            },
        );
    }

    pub fn get(&self, id: &str) -> Option<&ContentKindRecord> {
        self.kinds.get(id)
    }

    pub fn register_extension(
        &mut self,
        mod_name: &str,
        id: &str,
        label: &str,
        parent: Option<String>,
    ) -> Result<(), String> {
        validate_content_kind_id(id, Some(mod_name))?;
        if self.kinds.contains_key(id) {
            return Err(format!("content kind {} is already registered", id));
        }
        self.kinds.insert(
            id.to_string(),
            ContentKindRecord {
                id: id.to_string(),
                label: label.to_string(),
                parent,
            },
        );
        Ok(())
    }

    pub fn validate_reference(&self, id: &str) -> Result<(), String> {
        if self.kinds.contains_key(id) {
            Ok(())
        } else {
            Err(format!("unknown content kind {}", id))
        }
    }
}

pub fn is_core_content_kind(id: &str) -> bool {
    id.starts_with("unibrowser.content.")
}

pub fn validate_content_kind_id(id: &str, registering_mod: Option<&str>) -> Result<(), String> {
    if id.is_empty() || !id.contains('.') {
        return Err(format!("invalid content kind id {}", id));
    }
    if is_core_content_kind(id) {
        if registering_mod.is_some() {
            return Err(format!("mod cannot register core content kind {}", id));
        }
        return Ok(());
    }
    if let Some(mod_name) = registering_mod {
        let prefix = format!("{}.content.", mod_name);
        if !id.starts_with(&prefix) {
            return Err(format!(
                "extension content kind {} must use namespace {}",
                id,
                prefix
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_mod_claiming_core_kind() {
        let err = validate_content_kind_id(ids::TEXT_PLAIN, Some("acme")).unwrap_err();
        assert!(err.contains("core"));
    }

    #[test]
    fn accepts_extension_kind_under_mod_namespace() {
        validate_content_kind_id("acme.content.widget", Some("acme")).expect("ok");
    }
}
