use std::fs;
use std::path::PathBuf;

use unibrowser_document_model::{mode_switch_preserves_bytes, DocumentEditSession};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/markdown")
}

#[test]
fn mode_switch_corpus_preserves_bytes() {
    let dir = fixture_dir();
    let mut count = 0;
    for entry in fs::read_dir(&dir).expect("fixtures").flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let bytes = fs::read(&path).expect("read");
        let report = mode_switch_preserves_bytes(&bytes, 8);
        assert!(
            report.bytes_unchanged,
            "mode switches changed bytes for {} ({} -> {})",
            path.display(),
            report.before_sha256,
            report.after_sha256
        );
        count += 1;
    }
    assert!(count >= 20);
}

#[test]
fn visual_round_trip_session_stable() {
    let sample = fs::read(fixture_dir().join("sample.md")).expect("sample");
    let mut session = DocumentEditSession::from_markdown_bytes(&sample);
    for _ in 0..4 {
        let bytes = session.serialize();
        session = DocumentEditSession::from_markdown_bytes(&bytes);
    }
    assert_eq!(sample, session.serialize());
}
