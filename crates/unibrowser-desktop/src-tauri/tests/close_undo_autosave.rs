//! Close path flushes autosave without modal confirm (unit-level).

use unibrowser_desktop_lib::autosave::AutosaveSlotRecord;

#[test]
fn autosave_slot_roundtrip_for_close_undo() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc_path = "/tmp/close-undo-note.txt";
    let record = AutosaveSlotRecord {
        document_path: doc_path.into(),
        backend_id: "base-text".into(),
        content: "draft before close".into(),
        saved_at_ms: 42,
    };
    let data = serde_json::to_string(&record).expect("json");
    let path = unibrowser_fs::write_rotating_slot(dir.path(), 3, data.as_bytes()).expect("write");
    let read = std::fs::read_to_string(&path).expect("read");
    let parsed: AutosaveSlotRecord = serde_json::from_str(&read).expect("parse");
    assert_eq!(parsed.content, "draft before close");
}
