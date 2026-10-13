use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const SCHEMA_ID: &str = "unibrowser.document.model";
pub const SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentModel {
    pub schema_id: String,
    pub schema_version: String,
    pub metadata: DocumentMetadata,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DocumentMetadata {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub front_matter: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pub host_format: String,
    #[serde(default)]
    pub serializer_plugin_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Paragraph {
        id: String,
        inlines: Vec<InlineSpan>,
    },
    Heading {
        id: String,
        level: u8,
        inlines: Vec<InlineSpan>,
    },
    Blockquote {
        id: String,
        inlines: Vec<InlineSpan>,
    },
    CodeBlock {
        id: String,
        language: Option<String>,
        text: String,
        fence: Option<String>,
    },
    BulletList {
        id: String,
        items: Vec<ListItem>,
    },
    OrderedList {
        id: String,
        start: u32,
        items: Vec<ListItem>,
    },
    HorizontalRule {
        id: String,
    },
    Image {
        id: String,
        src: String,
        alt: String,
        title: Option<String>,
    },
    Opaque(OpaqueBlock),
    Table {
        id: String,
        rows: Vec<Vec<TableCell>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpaqueBlock {
    pub id: String,
    pub raw: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListItem {
    pub id: String,
    pub checked: Option<bool>,
    pub inlines: Vec<InlineSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableCell {
    pub inlines: Vec<InlineSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InlineSpan {
    pub text: String,
    #[serde(default)]
    pub marks: Vec<InlineMark>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InlineMark {
    pub kind: MarkKind,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub emulation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Bold,
    Italic,
    Strikethrough,
    Code,
    Link,
    Underline,
    Subscript,
    Superscript,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SourceMap {
    pub entries: Vec<SourceMapEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceMapEntry {
    pub block_id: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub digest: String,
}

#[derive(Debug, Error)]
pub enum ModelValidationError {
    #[error("unsupported schema_id {0}")]
    SchemaId(String),
    #[error("block {block_id}: {message}")]
    Block { block_id: String, message: String },
    #[error("empty document")]
    Empty,
}

pub fn digest_bytes(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hex::encode(hash)
}

pub fn validate_document_model(model: &DocumentModel) -> Result<(), ModelValidationError> {
    if model.schema_id != SCHEMA_ID {
        return Err(ModelValidationError::SchemaId(model.schema_id.clone()));
    }
    if model.blocks.is_empty() {
        return Err(ModelValidationError::Empty);
    }
    for block in &model.blocks {
        let id = block_id(block);
        if let Block::Heading { level, .. } = block {
            if *level < 1 || *level > 6 {
                return Err(ModelValidationError::Block {
                    block_id: id,
                    message: "heading level must be 1–6".into(),
                });
            }
        }
    }
    Ok(())
}

pub fn block_id(block: &Block) -> String {
    match block {
        Block::Paragraph { id, .. } => id.clone(),
        Block::Heading { id, .. } => id.clone(),
        Block::Blockquote { id, .. } => id.clone(),
        Block::CodeBlock { id, .. } => id.clone(),
        Block::BulletList { id, .. } => id.clone(),
        Block::OrderedList { id, .. } => id.clone(),
        Block::HorizontalRule { id } => id.clone(),
        Block::Image { id, .. } => id.clone(),
        Block::Opaque(o) => o.id.clone(),
        Block::Table { id, .. } => id.clone(),
    }
}

pub fn content_tree_nodes(model: &DocumentModel) -> Vec<unibrowser_mod_api::ContentKindNode> {
    vec![unibrowser_mod_api::ContentKindNode {
        node_id: "rich_text_root".into(),
        kind_id: "unibrowser.content.rich_text".into(),
        parent_id: None,
        host_format_id: model.metadata.host_format.clone(),
        blob_ref: None,
        label: model.metadata.title.clone(),
    }]
}
