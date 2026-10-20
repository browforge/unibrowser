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
        DocumentHistoryOp::TextReplace {
            block_id,
            from,
            to,
            text,
            ..
        } => {
            let index = model
                .blocks
                .iter()
                .position(|b| block_id_of(b) == block_id.as_str())
                .ok_or_else(|| "block not found".to_string())?;
            let block = model.blocks[index].clone();
            model.blocks[index] = replace_block_text(&block, *from, *to, text)?;
            Ok(())
        }
        DocumentHistoryOp::MarkToggle {
            block_id,
            from,
            to,
            mark,
        } => {
            let index = model
                .blocks
                .iter()
                .position(|b| block_id_of(b) == block_id.as_str())
                .ok_or_else(|| "block not found".to_string())?;
            let block = model.blocks[index].clone();
            model.blocks[index] = toggle_block_mark(&block, *from, *to, mark)?;
            Ok(())
        }
        DocumentHistoryOp::Snapshot { model: snap, .. } => {
            *model = snap.clone();
            Ok(())
        }
    }
}

pub fn invert_history_op(model: &DocumentModel, op: &DocumentHistoryOp) -> Option<DocumentHistoryOp> {
    match op {
        DocumentHistoryOp::TextReplace {
            block_id,
            from,
            to,
            text,
            marks,
        } => {
            let block = model
                .blocks
                .iter()
                .find(|b| block_id_of(b) == block_id.as_str())?;
            let plain = block_plain(block);
            if *from > plain.len() || *to > plain.len() || *from > *to {
                return None;
            }
            let old = plain[*from..*to].to_string();
            let new_to = from + text.len();
            Some(DocumentHistoryOp::TextReplace {
                block_id: block_id.clone(),
                from: *from,
                to: new_to,
                text: old,
                marks: marks.clone(),
            })
        }
        DocumentHistoryOp::MarkToggle {
            block_id,
            from,
            to,
            mark,
        } => Some(DocumentHistoryOp::MarkToggle {
            block_id: block_id.clone(),
            from: *from,
            to: *to,
            mark: mark.clone(),
        }),
        DocumentHistoryOp::BlockSetStyle {
            block_id,
            style: _style,
            emulation_note,
        } => {
            let block = model
                .blocks
                .iter()
                .find(|b| block_id_of(b) == block_id.as_str())?;
            let prior = block_style_name(block);
            Some(DocumentHistoryOp::BlockSetStyle {
                block_id: block_id.clone(),
                style: prior,
                emulation_note: emulation_note.clone(),
            })
        }
        DocumentHistoryOp::OpaqueReplace { block_id, .. } => {
            let block = model
                .blocks
                .iter()
                .find(|b| block_id_of(b) == block_id.as_str())?;
            if let Block::Opaque(o) = block {
                return Some(DocumentHistoryOp::OpaqueReplace {
                    block_id: block_id.clone(),
                    raw: o.raw.clone(),
                });
            }
            None
        }
        DocumentHistoryOp::Snapshot { .. } => None,
    }
}

fn block_plain(block: &Block) -> String {
    inline_plain(&inlines_of(block))
}

fn block_style_name(block: &Block) -> String {
    match block {
        Block::Paragraph { .. } => "paragraph".into(),
        Block::Heading { level, .. } => format!("heading_{}", level),
        Block::Blockquote { .. } => "blockquote".into(),
        Block::CodeBlock { .. } => "code_block".into(),
        _ => "paragraph".into(),
    }
}

fn replace_block_text(block: &Block, from: usize, to: usize, text: &str) -> Result<Block, String> {
    let plain = block_plain(block);
    if from > to || to > plain.len() {
        return Err("invalid text range".into());
    }
    let merged = format!("{}{}{}", &plain[..from], text, &plain[to..]);
    let inlines = vec![crate::model::InlineSpan {
        text: merged,
        marks: vec![],
    }];
    Ok(block_with_inlines(block, inlines))
}

fn toggle_block_mark(block: &Block, from: usize, to: usize, mark: &str) -> Result<Block, String> {
    let kind = parse_mark_kind(mark)?;
    let inlines = inlines_of(block);
    let plain = inline_plain(&inlines);
    if from > to || to > plain.len() {
        return Err("invalid mark range".into());
    }
    let toggled = apply_mark_toggle_on_inlines(&inlines, from, to, kind);
    Ok(block_with_inlines(block, toggled))
}

fn parse_mark_kind(mark: &str) -> Result<crate::model::MarkKind, String> {
    use crate::model::MarkKind;
    match mark {
        "bold" => Ok(MarkKind::Bold),
        "italic" => Ok(MarkKind::Italic),
        "strikethrough" => Ok(MarkKind::Strikethrough),
        "code" => Ok(MarkKind::Code),
        "underline" => Ok(MarkKind::Underline),
        _ => Err(format!("unsupported mark {}", mark)),
    }
}

fn block_with_inlines(block: &Block, inlines: Vec<crate::model::InlineSpan>) -> Block {
    match block {
        Block::Paragraph { id, .. } => Block::Paragraph {
            id: id.clone(),
            inlines,
        },
        Block::Heading { id, level, .. } => Block::Heading {
            id: id.clone(),
            level: *level,
            inlines,
        },
        Block::Blockquote { id, .. } => Block::Blockquote {
            id: id.clone(),
            inlines,
        },
        Block::CodeBlock { id, language, fence, .. } => Block::CodeBlock {
            id: id.clone(),
            language: language.clone(),
            text: inline_plain(&inlines),
            fence: fence.clone(),
        },
        other => other.clone(),
    }
}

fn apply_mark_toggle_on_inlines(
    inlines: &[crate::model::InlineSpan],
    from: usize,
    to: usize,
    kind: crate::model::MarkKind,
) -> Vec<crate::model::InlineSpan> {
    use crate::model::{InlineMark, InlineSpan};
    let mut out = Vec::new();
    let mut offset = 0usize;
    for span in inlines {
        let len = span.text.len();
        let span_start = offset;
        let span_end = offset + len;
        offset = span_end;
        if span_end <= from || span_start >= to {
            out.push(span.clone());
            continue;
        }
        let local_from = from.saturating_sub(span_start);
        let local_to = to.min(span_end) - span_start;
        if local_from > 0 {
            out.push(InlineSpan {
                text: span.text[..local_from].to_string(),
                marks: span.marks.clone(),
            });
        }
        let mid = &span.text[local_from..local_to];
        let has = span.marks.iter().any(|m| m.kind == kind);
        let mut marks = span.marks.clone();
        if has {
            marks.retain(|m| m.kind != kind);
        } else {
            marks.push(InlineMark {
                kind: kind.clone(),
                href: None,
                emulation: None,
            });
        }
        out.push(InlineSpan {
            text: mid.to_string(),
            marks,
        });
        if local_to < span.text.len() {
            out.push(InlineSpan {
                text: span.text[local_to..].to_string(),
                marks: span.marks.clone(),
            });
        }
    }
    out
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
