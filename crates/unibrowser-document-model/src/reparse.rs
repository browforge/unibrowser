use crate::markdown::parse_markdown;
use crate::model::digest_bytes;
use crate::session::DocumentEditSession;

#[derive(Debug, Clone)]
pub struct SourceMergeResult {
    pub clean: bool,
    pub session: DocumentEditSession,
    pub conflict_reason: Option<String>,
}

pub fn source_projection(session: &DocumentEditSession) -> String {
    String::from_utf8_lossy(&session.serialize()).into_owned()
}

/// Re-parse source text and merge into the session when untouched block digests still match.
pub fn merge_source_text(session: &DocumentEditSession, new_text: &str) -> SourceMergeResult {
    let new_bytes = new_text.as_bytes();
    if new_bytes == session.original_bytes.as_slice() {
        return SourceMergeResult {
            clean: true,
            session: session.clone(),
            conflict_reason: None,
        };
    }
    let mut conflict = false;
    for entry in &session.source_map.entries {
        if session.dirty_blocks.contains(&entry.block_id)
            || session.touched_blocks.contains(&entry.block_id)
        {
            continue;
        }
        let end = entry.byte_end.min(new_bytes.len());
        let start = entry.byte_start.min(end);
        let slice = &new_bytes[start..end];
        if digest_bytes(slice) != entry.digest {
            conflict = true;
            break;
        }
    }
    if conflict {
        return SourceMergeResult {
            clean: false,
            session: session.clone(),
            conflict_reason: Some(
                "Source edit could not be mapped cleanly to the document model.".into(),
            ),
        };
    }
    let parsed = parse_markdown(new_bytes);
    let merged = DocumentEditSession {
        model: parsed.model,
        source_map: parsed.source_map,
        original_bytes: new_bytes.to_vec(),
        dirty_blocks: session.dirty_blocks.clone(),
        touched_blocks: session.touched_blocks.clone(),
    };
    SourceMergeResult {
        clean: true,
        session: merged,
        conflict_reason: None,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModeSwitchReport {
    pub bytes_unchanged: bool,
    pub before_sha256: String,
    pub after_sha256: String,
}

pub fn mode_switch_preserves_bytes(original: &[u8], modes: usize) -> ModeSwitchReport {
    let before_sha256 = digest_bytes(original);
    let mut session = DocumentEditSession::from_markdown_bytes(original);
    let mut current = original.to_vec();
    for _ in 0..modes {
        let _ = source_projection(&session);
        session = DocumentEditSession::from_markdown_bytes(&current);
        current = session.serialize();
    }
    let after_sha256 = digest_bytes(&current);
    ModeSwitchReport {
        bytes_unchanged: before_sha256 == after_sha256,
        before_sha256,
        after_sha256,
    }
}
