use std::fs;

use tempfile::tempdir;
use unibrowser_document_model::{
    append_document_op, apply_history_op, invert_history_op, list_ops, parse_markdown,
    session_state_after_ops, sidecar_path_for_file, DocumentHistoryOp,
};

#[test]
fn hist01_text_replace_replays_from_log() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "Hello world\n").expect("write");
    let parsed = parse_markdown(b"Hello world\n");
    let mut model = parsed.model;
    let op = DocumentHistoryOp::TextReplace {
        block_id: model.blocks[0].clone().id_from_block(),
        from: 6,
        to: 11,
        text: "earth".into(),
        marks: vec![],
    };
    apply_history_op(&mut model, &op).expect("apply");
    assert!(block_text(&model).contains("Hello earth"));

    append_document_op(&md, &op, "0.1.0").expect("append");
    let sidecar = sidecar_path_for_file(&md);
    let ops = list_ops(&sidecar).expect("list");
    let replayed = session_state_after_ops(&md, &ops).expect("replay");
    assert!(block_text(&replayed.model).contains("Hello earth"));
}

#[test]
fn hist02_invert_text_replace_roundtrip() {
    let parsed = parse_markdown(b"Hello world\n");
    let mut model = parsed.model;
    let block_id = model.blocks[0].clone().id_from_block();
    let op = DocumentHistoryOp::TextReplace {
        block_id,
        from: 6,
        to: 11,
        text: "earth".into(),
        marks: vec![],
    };
    let before = block_text(&model);
    let inverse = invert_history_op(&model, &op).expect("invert");
    apply_history_op(&mut model, &op).expect("apply");
    apply_history_op(&mut model, &inverse).expect("undo apply");
    assert_eq!(block_text(&model), before);
}

trait BlockIdExt {
    fn id_from_block(&self) -> String;
}

impl BlockIdExt for unibrowser_document_model::Block {
    fn id_from_block(&self) -> String {
        match self {
            unibrowser_document_model::Block::Paragraph { id, .. } => id.clone(),
            unibrowser_document_model::Block::Heading { id, .. } => id.clone(),
            _ => "x".into(),
        }
    }
}

fn block_text(model: &unibrowser_document_model::DocumentModel) -> String {
    match &model.blocks[0] {
        unibrowser_document_model::Block::Paragraph { inlines, .. } => {
            inlines.iter().map(|s| s.text.as_str()).collect()
        }
        _ => String::new(),
    }
}
