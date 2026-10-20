use std::fs;

use tempfile::tempdir;
use unibrowser_document_model::{
    append_document_op, list_ops, parse_markdown, sidecar_path_for_file, DocumentHistoryOp,
    LOG_MARKER,
};

#[test]
fn foreign_markdown_sidecar_name_and_log_marker() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("note.md");
    fs::write(&md, "# Hi\n").expect("write");
    let sidecar = sidecar_path_for_file(&md);
    assert_eq!(
        sidecar,
        dir.path().join("note.md.unibrow-history"),
        "foreign-file layout uses sibling sidecar"
    );

    let parsed = parse_markdown(b"# Hi\n");
    let op = DocumentHistoryOp::Snapshot {
        model: parsed.model,
        source_map_blob: String::new(),
    };
    append_document_op(&md, &op, "0.1.0").expect("append");

    let raw = fs::read_to_string(&sidecar).expect("read sidecar");
    assert!(raw.contains("historyFormat: 1"));
    assert!(raw.contains(LOG_MARKER));
    let ops = list_ops(&sidecar).expect("list");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].seq, 1);
    assert_eq!(ops[0].plugin_id, "base-text");
}

#[test]
fn append_increments_seq_and_survives_reopen() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("a.md");
    fs::write(&md, "one\n").expect("write");
    let parsed = parse_markdown(b"one\n");
    for _ in 0..2 {
        let op = DocumentHistoryOp::Snapshot {
            model: parsed.model.clone(),
            source_map_blob: String::new(),
        };
        append_document_op(&md, &op, "0.1.0").expect("append");
    }
    let sidecar = sidecar_path_for_file(&md);
    let ops = list_ops(&sidecar).expect("list");
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].seq, 1);
    assert_eq!(ops[1].seq, 2);
    assert_eq!(ops[1].parent_seq, 1);

    drop(ops);
    let again = list_ops(&sidecar).expect("reopen");
    assert_eq!(again.len(), 2);
}

#[test]
fn truncated_tail_line_is_ignored_on_open() {
    let dir = tempdir().expect("tempdir");
    let md = dir.path().join("b.md");
    fs::write(&md, "x\n").expect("write");
    let parsed = parse_markdown(b"x\n");
    append_document_op(
        &md,
        &DocumentHistoryOp::Snapshot {
            model: parsed.model,
            source_map_blob: String::new(),
        },
        "0.1.0",
    )
    .expect("append");

    let sidecar = sidecar_path_for_file(&md);
    let mut raw = fs::read_to_string(&sidecar).expect("read");
    raw.push_str("{\"op_id\":\"partial\",\"seq\":9,\"parent_seq\":1,\"timestamp\":\"1Z\",\"plugin_id\":\"base-text\",\"plugin_version\":\"0.1.0\",\"op_type\":\"x\",\"op_schema_version\":1,\"payload\":{\n");
    fs::write(&sidecar, raw).expect("corrupt tail");

    let ops = list_ops(&sidecar).expect("list should recover last complete op");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].seq, 1);
}

#[test]
fn folder_project_uses_per_file_sidecar_not_package_root() {
    let dir = tempdir().expect("tempdir");
    let show = dir.path().join("MyShow");
    fs::create_dir_all(&show).expect("mkdir");
    let md = show.join("clip.md");
    fs::write(&md, "clip\n").expect("write");
    let sidecar = sidecar_path_for_file(&md);
    assert_eq!(sidecar, show.join("clip.md.unibrow-history"));
    assert!(
        !show.join(".unibrow-history").exists(),
        "package-root history file is not created by markdown sidecar helper"
    );
}
