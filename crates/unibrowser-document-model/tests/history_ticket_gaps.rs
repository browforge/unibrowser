use std::fs;

use tempfile::tempdir;
use unibrowser_document_model::{
    append_document_op, jump_to_seq, list_ops, parse_markdown, session_at_history_head,
    serialize_session_state, squash_op_range, truncate_head, trim_history_before,
    DocumentHistoryOp, sidecar_path_for_file,
};

#[test]
fn hist06_truncate_head_keeps_ops_up_to_seq() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "# x\n").expect("write");
    let parsed = parse_markdown(b"# x\n");
    for _ in 0..3 {
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model.clone(),
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("append");
    }
    let sidecar = sidecar_path_for_file(&md);
    truncate_head(&sidecar, 2).expect("truncate");
    let ops = list_ops(&sidecar).expect("list");
    assert_eq!(ops.len(), 2);
    assert_eq!(ops.last().map(|o| o.seq), Some(2));
}

#[test]
fn hist03_pin_header_not_rewritten_for_new_plugin_version() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "a\n").expect("write");
    let parsed = parse_markdown(b"a\n");
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: parsed.model.clone(),
            source_map_blob: String::new(),
        },
        "0.1.0",
    )
    .expect("append");
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: parsed.model.clone(),
            source_map_blob: String::new(),
        },
        "9.9.9",
    )
    .expect("append v2");

    let raw = fs::read_to_string(sidecar_path_for_file(&md)).expect("read");
    assert!(
        raw.contains("base-text@0.1.0"),
        "header still shows default pin; op carries 9.9.9 in log only"
    );
    let ops = list_ops(&sidecar_path_for_file(&md)).expect("list");
    assert_eq!(ops[1].plugin_version, "9.9.9");
}

#[test]
fn hist06_trim_before_keeps_from_seq_onward() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "# x\n").expect("write");
    let parsed = parse_markdown(b"# x\n");
    for _ in 0..4 {
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model.clone(),
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("append");
    }
    trim_history_before(&md, 3).expect("trim");
    let ops = list_ops(&sidecar_path_for_file(&md)).expect("list");
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].seq, 1);
    assert_eq!(ops[1].seq, 2);
}

#[test]
fn hist06_squash_range_replaces_span_with_snapshot() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "a\n").expect("write");
    let parsed = parse_markdown(b"a\n");
    for _ in 0..3 {
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model.clone(),
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("append");
    }
    squash_op_range(&md, 2, 3, "0.1.0").expect("squash");
    let ops = list_ops(&sidecar_path_for_file(&md)).expect("list");
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[1].op_type, "unibrowser.history.snapshot");
}

#[test]
fn hist02_jump_appends_snapshot_preserving_prior_ops() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "# Hello\n\nWorld.\n").expect("write");
    let parsed_a = parse_markdown(b"# Hello\n\nWorld.\n");
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: parsed_a.model,
            source_map_blob: String::new(),
        },
        "0.1.0",
    )
    .expect("a");
    let parsed_b = parse_markdown(b"# Hello\n\nEdited.\n");
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: parsed_b.model,
            source_map_blob: String::new(),
        },
        "0.1.0",
    )
    .expect("b");
    jump_to_seq(&md, 1, "0.1.0").expect("jump");
    let ops = list_ops(&sidecar_path_for_file(&md)).expect("list");
    assert_eq!(ops.len(), 3, "jump must not delete tail ops");
    let head = session_at_history_head(&md).expect("head").expect("state");
    let md_out = serialize_session_state(&head).expect("serialize");
    assert!(md_out.contains("World."));
    assert!(!md_out.contains("Edited."));
}

#[test]
fn hist05_docx_sidecar_path_is_nested_extension() {
    let dir = tempdir().expect("tempdir");
    let docx = dir.path().join("report.docx");
    fs::write(&docx, b"PK").expect("write stub");
    let sidecar = sidecar_path_for_file(&docx);
    assert_eq!(sidecar, dir.path().join("report.docx.unibrow-history"));
}
