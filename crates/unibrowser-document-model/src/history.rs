use crate::model::{Block, DocumentModel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op_type", rename_all = "snake_case")]
pub enum DocumentHistoryOp {
    #[serde(rename = "base-text.block.set_style")]
    BlockSetStyle {
        block_id: String,
        style: String,
        emulation_note: Option<String>,
    },
    #[serde(rename = "base-text.text.replace")]
    TextReplace {
        block_id: String,
        from: usize,
        to: usize,
        text: String,
        marks: Vec<String>,
    },
    #[serde(rename = "base-text.mark.toggle")]
    MarkToggle {
        block_id: String,
        from: usize,
        to: usize,
        mark: String,
    },
    #[serde(rename = "base-text.opaque.replace")]
    OpaqueReplace { block_id: String, raw: String },
    #[serde(rename = "unibrowser.history.snapshot")]
    Snapshot {
        model: DocumentModel,
        source_map_blob: String,
    },
}

pub fn apply_history_op(model: &mut DocumentModel, op: &DocumentHistoryOp) -> Result<(), String> {
    match op {
        DocumentHistoryOp::BlockSetStyle { block_id, style, .. } => {
            let index = model
                .blocks
                .iter()
                .position(|b| block_id_of(b) == block_id.as_str())
                .ok_or_else(|| "block not found".to_string())?;
            let block = model.blocks[index].clone();
            let inlines = inlines_of(&block);
            let new_block = if style == "paragraph" {
                Block::Paragraph {
                    id: block_id.clone(),
                    inlines,
                }
            } else if let Some(level) = style.strip_prefix("heading_") {
                let level = level
                    .parse::<u8>()
                    .map_err(|_| "invalid heading level".to_string())?;
                Block::Heading {
                    id: block_id.clone(),
                    level,
                    inlines,
                }
            } else if style == "blockquote" {
                Block::Blockquote {
                    id: block_id.clone(),
                    inlines,
                }
            } else if style == "code_block" {
                Block::CodeBlock {
                    id: block_id.clone(),
                    language: None,
                    text: inline_plain(&inlines),
                    fence: Some("```".into()),
                }
            } else {
                return Err("unsupported style".into());
            };
            model.blocks[index] = new_block;
            Ok(())
        }
        DocumentHistoryOp::OpaqueReplace { block_id, raw } => {
            let index = model
                .blocks
                .iter()
                .position(|b| block_id_of(b) == block_id.as_str())
                .ok_or_else(|| "block not found".to_string())?;
            model.blocks[index] = Block::Opaque(crate::model::OpaqueBlock {
                id: block_id.clone(),
                raw: raw.clone(),
                language: None,
                label: Some("Raw Markdown (preserved)".into()),
            });
            Ok(())
        }
        DocumentHistoryOp::Snapshot { model: snap, .. } => {
            *model = snap.clone();
            Ok(())
        }
        _ => Err("op not implemented in replay".into()),
    }
}

pub fn invert_history_op(_model: &DocumentModel, _op: &DocumentHistoryOp) -> Option<DocumentHistoryOp> {
    None
}

fn block_id_of(block: &Block) -> &str {
    match block {
        Block::Paragraph { id, .. } => id,
        Block::Heading { id, .. } => id,
        Block::Blockquote { id, .. } => id,
        Block::CodeBlock { id, .. } => id,
        Block::BulletList { id, .. } => id,
        Block::OrderedList { id, .. } => id,
        Block::HorizontalRule { id } => id,
        Block::Image { id, .. } => id,
        Block::Opaque(o) => &o.id,
        Block::Table { id, .. } => id,
    }
}

fn inlines_of(block: &Block) -> Vec<crate::model::InlineSpan> {
    match block {
        Block::Paragraph { inlines, .. } => inlines.clone(),
        Block::Heading { inlines, .. } => inlines.clone(),
        Block::Blockquote { inlines, .. } => inlines.clone(),
        Block::CodeBlock { text, .. } => vec![crate::model::InlineSpan {
            text: text.clone(),
            marks: vec![],
        }],
        _ => vec![],
    }
}

fn inline_plain(inlines: &[crate::model::InlineSpan]) -> String {
    inlines.iter().map(|s| s.text.as_str()).collect()
}
