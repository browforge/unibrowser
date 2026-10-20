use std::fs;

use tempfile::tempdir;
use unibrowser_document_model::{
    append_document_op, encode_snapshot_blob, jump_to_seq, parse_markdown, session_at_history_head,
    session_state_at_seq, Block, DocumentHistoryOp,
};

const BASELINE: &str = "# Demo markdown

Unibrowser renders Markdown in the document viewer.

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.
";

const EDITED: &str = "# Edited heading

Unibrowser renders Markdown in the document viewer.

Edit three

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.
";

fn block_kind(block: &Block) -> &'static str {
    match block {
        Block::Paragraph { .. } => "paragraph",
        Block::Heading { level, .. } => {
            if *level == 1 {
                "heading_1"
            } else {
                "heading_other"
            }
        }
        Block::Blockquote { .. } => "blockquote",
        _ => "other",
    }
}

#[test]
fn capture_baseline_second_block_is_paragraph_not_heading() {
    let parsed = parse_markdown(BASELINE.as_bytes());
    assert!(
        parsed.model.blocks.len() >= 2,
        "expected at least heading + paragraph"
    );
    assert_eq!(block_kind(&parsed.model.blocks[1]), "paragraph");
}

#[test]
fn capture_edited_has_edit_three_paragraph() {
    let parsed = parse_markdown(EDITED.as_bytes());
    let kinds: Vec<_> = parsed.model.blocks.iter().map(block_kind).collect();
    assert!(
        kinds.iter().any(|k| *k == "paragraph"),
        "edited doc should include a paragraph block"
    );
    let plain = parsed
        .model
        .blocks
        .iter()
        .map(|b| match b {
            Block::Paragraph { inlines, .. } | Block::Heading { inlines, .. } => {
                inlines.iter().map(|s| s.text.as_str()).collect::<String>()
            }
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("|");
    assert!(plain.contains("Edit three"));
}

#[test]
fn capture_restore_round_trips_paragraph_block_type() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("sample.md");
    fs::write(&md, BASELINE).expect("write");
    let baseline = parse_markdown(BASELINE.as_bytes());
    let blob = encode_snapshot_blob(&baseline.source_map, &base64_encode(BASELINE));
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: baseline.model,
            source_map_blob: blob,
        },
        "0.1.0",
    )
    .expect("snapshot");
    let edited = parse_markdown(EDITED.as_bytes());
    let edited_blob = encode_snapshot_blob(&edited.source_map, &base64_encode(EDITED));
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: edited.model,
            source_map_blob: edited_blob,
        },
        "0.1.0",
    )
    .expect("edited");
    jump_to_seq(&md, 1, "0.1.0").expect("jump");
    let head = session_at_history_head(&md).expect("head").expect("state");
    assert_eq!(block_kind(&head.model.blocks[1]), "paragraph");
    let at_one = session_state_at_seq(&md, 1).expect("at").expect("state");
    assert_eq!(block_kind(&at_one.model.blocks[1]), "paragraph");
}

fn base64_encode(text: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(text.as_bytes())
}
