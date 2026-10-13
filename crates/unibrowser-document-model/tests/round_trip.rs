use std::fs;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use unibrowser_document_model::{parse_markdown, serialize_markdown, DocumentEditSession};
use unibrowser_document_model::markdown::SerializeOptions;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/markdown")
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
fn fixture_corpus_byte_identical_without_edits() {
    let dir = fixture_dir();
    if !dir.is_dir() {
        return;
    }
    let mut count = 0;
    for entry in fs::read_dir(&dir).expect("fixtures dir").flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let bytes = fs::read(&path).expect("read fixture");
        let before = sha256(&bytes);
        let session = DocumentEditSession::from_markdown_bytes(&bytes);
        let out = session.serialize();
        let after = sha256(&out);
        assert_eq!(before, after, "round-trip changed bytes for {}", path.display());
        count += 1;
    }
    assert!(count >= 20, "expected at least 20 markdown fixtures, got {count}");
}

#[test]
fn edited_block_may_change_only_that_region() {
    let sample = b"# Title\n\nBody one.\n\nBody two.\n";
    let mut session = DocumentEditSession::from_markdown_bytes(sample);
    let id = session.model.blocks[1].clone();
    let block_id = match id {
        unibrowser_document_model::Block::Paragraph { id, .. } => id,
        _ => panic!("expected paragraph"),
    };
    session.set_block_style(&block_id, unibrowser_document_model::session::BlockStyle::Heading(2))
        .expect("style");
    let out = session.serialize();
    let out_str = String::from_utf8_lossy(&out);
    assert!(out_str.contains("## Body one."));
    assert!(out_str.contains("Body two."));
}

#[test]
fn opaque_survives_parse() {
    let raw = b"<custom>\n  preserved\n</custom>\n\nAfter.\n";
    let parsed = parse_markdown(raw);
    assert!(parsed.model.blocks.iter().any(|b| matches!(
        b,
        unibrowser_document_model::Block::Opaque(_)
    )));
    let out = serialize_markdown(raw, &parsed.model, &parsed.source_map, &SerializeOptions::default());
    assert_eq!(raw.as_slice(), out.as_slice());
}
