use std::fs;
use tempfile::tempdir;
use unibrowser_document_model::{
    append_document_op, encode_snapshot_blob, parse_markdown, pop_last_op, push_op_entry,
    serialize_session_state, session_at_history_head, sidecar_path_for_file, DocumentHistoryOp,
};

#[test]
fn undo_after_reopen_restores_prior_snapshot() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "# Hello\n\nWorld.\n").expect("write md");
    let parsed = parse_markdown(b"# Hello\n\nWorld.\n");
    let blob = encode_snapshot_blob(
        &parsed.source_map,
        &base64_encode(b"# Hello\n\nWorld.\n"),
    );
    let snap_a = DocumentHistoryOp::Snapshot {
        model: parsed.model,
        source_map_blob: blob,
    };
    append_document_op(&md, &snap_a, "0.1.0").expect("append a");

    let parsed_b = parse_markdown(b"# Hello\n\nEdited.\n");
    let blob_b = encode_snapshot_blob(
        &parsed_b.source_map,
        &base64_encode(b"# Hello\n\nEdited.\n"),
    );
    let snap_b = DocumentHistoryOp::Snapshot {
        model: parsed_b.model,
        source_map_blob: blob_b,
    };
    append_document_op(&md, &snap_b, "0.1.0").expect("append b");

    let sidecar = unibrowser_document_model::sidecar_path_for_file(&md);
    let popped = pop_last_op(&sidecar).expect("pop");
    assert!(popped.is_some());

    let state = session_at_history_head(&md).expect("head").expect("state");
    let markdown = serialize_session_state(&state).expect("serialize");
    assert!(markdown.contains("World."));
    assert!(!markdown.contains("Edited."));
}

#[test]
fn redo_after_undo_from_history_head_restores_latest() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "# Hello\n\nWorld.\n").expect("write md");
    let parsed_b = parse_markdown(b"# Hello\n\nEdited.\n");
    let blob_b = encode_snapshot_blob(
        &parsed_b.source_map,
        &base64_encode(b"# Hello\n\nEdited.\n"),
    );
    let snap_b = DocumentHistoryOp::Snapshot {
        model: parsed_b.model,
        source_map_blob: blob_b,
    };
    append_document_op(&md, &snap_b, "0.1.0").expect("append");

    let sidecar = sidecar_path_for_file(&md);
    let popped = pop_last_op(&sidecar).expect("undo pop");
    let entry = popped.expect("entry");
    let undone = session_at_history_head(&md).expect("head").expect("state");
    let undone_md = serialize_session_state(&undone).expect("serialize");
    assert!(undone_md.contains("World."));
    assert!(!undone_md.contains("Edited."));

    push_op_entry(&sidecar, entry).expect("redo push");
    let redone = session_at_history_head(&md).expect("head").expect("state");
    let redone_md = serialize_session_state(&redone).expect("serialize");
    assert!(redone_md.contains("Edited."));
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
