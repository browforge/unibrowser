use base64::Engine;
use unibrowser_document_model::{
    encode_snapshot_blob, parse_markdown, Block, DocumentHistoryOp, DocumentModel, HistoryLogEntry,
};
use unibrowser_document_model::history_undo::session_state_after_bytes;

use crate::document_model::{history_entry_from_op, DocumentParseResponse};

pub const CAPTURE_BASELINE_MD: &str = "# Demo markdown

Unibrowser renders Markdown in the document viewer.

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.

![Demo pack image](sample.png)

- **Bold** and _italic_
- [Architecture](../../docs/architecture.md) (link may not resolve in preview; open from repo)

## Next steps

Open `sample.unibrow` for the native project format.
";

pub const CAPTURE_EDITED_MD: &str = "# Edited heading

Unibrowser renders Markdown in the document viewer.

Edit three

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.

![Demo pack image](sample.png)

- **Bold** and _italic_
- [Architecture](../../docs/architecture.md) (link may not resolve in preview; open from repo)

## Next steps

Open `sample.unibrow` for the native project format.
";

fn block_id_of(block: &Block) -> String {
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

fn block_plain(block: &Block) -> String {
    use unibrowser_document_model::model::InlineSpan;
    let inlines: &[InlineSpan] = match block {
        Block::Paragraph { inlines, .. } | Block::Heading { inlines, .. } | Block::Blockquote { inlines, .. } => {
            inlines
        }
        _ => return String::new(),
    };
    inlines.iter().map(|s| s.text.as_str()).collect()
}

fn first_paragraph_id(model: &DocumentModel) -> Option<String> {
    model
        .blocks
        .iter()
        .find(|b| matches!(b, Block::Paragraph { .. }))
        .map(block_id_of)
}

fn push_op(list: &mut Vec<HistoryLogEntry>, op: DocumentHistoryOp) -> Result<(), String> {
    let seq = list.last().map(|e| e.seq + 1).unwrap_or(1);
    list.push(history_entry_from_op(op, seq)?);
    Ok(())
}

pub fn build_history_capture_demo_entries() -> Result<Vec<HistoryLogEntry>, String> {
    let mut list = Vec::new();
    let baseline = parse_markdown(CAPTURE_BASELINE_MD.as_bytes());
    let baseline_blob = encode_snapshot_blob(
        &baseline.source_map,
        &base64::engine::general_purpose::STANDARD.encode(CAPTURE_BASELINE_MD.as_bytes()),
    );
    let heading_id = baseline
        .model
        .blocks
        .iter()
        .find(|b| matches!(b, Block::Heading { .. }))
        .map(block_id_of)
        .ok_or_else(|| "capture baseline missing heading".to_string())?;
    let heading_text = block_plain(
        baseline
            .model
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Heading { .. }))
            .expect("heading"),
    );
    push_op(
        &mut list,
        DocumentHistoryOp::Snapshot {
            model: baseline.model,
            source_map_blob: baseline_blob,
        },
    )?;
    push_op(
        &mut list,
        DocumentHistoryOp::TextReplace {
            block_id: heading_id,
            from: 0,
            to: heading_text.len(),
            text: "Edited heading".into(),
            marks: vec![],
        },
    )?;

    let head = session_state_after_bytes(CAPTURE_BASELINE_MD.as_bytes(), &list)?;
    let para_id = first_paragraph_id(&head.model).ok_or_else(|| "paragraph missing".to_string())?;
    push_op(
        &mut list,
        DocumentHistoryOp::TextReplace {
            block_id: para_id,
            from: 0,
            to: 0,
            text: "Edit three\n\n".into(),
            marks: vec![],
        },
    )?;

    let edited = parse_markdown(CAPTURE_EDITED_MD.as_bytes());
    let edited_blob = encode_snapshot_blob(
        &edited.source_map,
        &base64::engine::general_purpose::STANDARD.encode(CAPTURE_EDITED_MD.as_bytes()),
    );
    push_op(
        &mut list,
        DocumentHistoryOp::Snapshot {
            model: edited.model,
            source_map_blob: edited_blob,
        },
    )?;
    Ok(list)
}

pub fn history_head_from_entries(entries: &[HistoryLogEntry]) -> Result<DocumentParseResponse, String> {
    let state = session_state_after_bytes(CAPTURE_BASELINE_MD.as_bytes(), entries)?;
    Ok(DocumentParseResponse {
        model: state.model,
        source_map: state.source_map,
        original_base64: base64::engine::general_purpose::STANDARD.encode(&state.original_bytes),
    })
}
