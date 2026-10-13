use crate::markdown::{parse_markdown, serialize_markdown, SerializeOptions};
use crate::model::{validate_document_model, Block, DocumentModel, SourceMap};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct DocumentEditSession {
    pub model: DocumentModel,
    pub source_map: SourceMap,
    pub original_bytes: Vec<u8>,
    pub dirty_blocks: HashSet<String>,
    pub touched_blocks: HashSet<String>,
}

impl DocumentEditSession {
    pub fn from_markdown_bytes(bytes: &[u8]) -> Self {
        let parsed = parse_markdown(bytes);
        let _ = validate_document_model(&parsed.model);
        Self {
            model: parsed.model,
            source_map: parsed.source_map,
            original_bytes: bytes.to_vec(),
            dirty_blocks: HashSet::new(),
            touched_blocks: HashSet::new(),
        }
    }

    pub fn mark_block_touched(&mut self, block_id: &str) {
        self.touched_blocks.insert(block_id.to_string());
        self.dirty_blocks.insert(block_id.to_string());
    }

    pub fn serialize(&self) -> Vec<u8> {
        serialize_markdown(
            &self.original_bytes,
            &self.model,
            &self.source_map,
            &SerializeOptions {
                dirty_blocks: self.dirty_blocks.clone(),
                touched_blocks: self.touched_blocks.clone(),
            },
        )
    }

    pub fn set_block_style(&mut self, block_id: &str, style: BlockStyle) -> Result<(), String> {
        let index = self
            .model
            .blocks
            .iter()
            .position(|b| block_id_of(b) == block_id)
            .ok_or_else(|| "block not found".to_string())?;
        let block = self.model.blocks[index].clone();
        let inlines = inlines_of(&block);
        let new_block = match style {
            BlockStyle::Paragraph => Block::Paragraph {
                id: block_id.to_string(),
                inlines,
            },
            BlockStyle::Heading(level) => Block::Heading {
                id: block_id.to_string(),
                level,
                inlines,
            },
            BlockStyle::Blockquote => Block::Blockquote {
                id: block_id.to_string(),
                inlines,
            },
            BlockStyle::CodeBlock => Block::CodeBlock {
                id: block_id.to_string(),
                language: None,
                text: inline_plain(&inlines),
                fence: Some("```".into()),
            },
        };
        self.model.blocks[index] = new_block;
        self.mark_block_touched(block_id);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BlockStyle {
    Paragraph,
    Heading(u8),
    Blockquote,
    CodeBlock,
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
        Block::BulletList { items, .. } | Block::OrderedList { items, .. } => {
            items.first().map(|i| i.inlines.clone()).unwrap_or_default()
        }
        _ => vec![crate::model::InlineSpan {
            text: String::new(),
            marks: vec![],
        }],
    }
}

fn inline_plain(inlines: &[crate::model::InlineSpan]) -> String {
    inlines.iter().map(|s| s.text.as_str()).collect()
}
