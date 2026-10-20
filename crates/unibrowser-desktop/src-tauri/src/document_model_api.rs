use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use unibrowser_document_model::{
    append_document_op, encode_snapshot_blob, jump_to_seq, list_ops, merge_source_text,
    parse_markdown, pop_last_op, push_op_entry, serialize_session_state, session_at_history_head,
    sidecar_path_for_file, source_projection, squash_op_range,
    trim_history_before, Block, DocumentEditSession, DocumentHistoryOp, DocumentModel,
    HistoryLogEntry, SourceMap,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentParseResponse {
    pub model: DocumentModel,
    pub source_map: SourceMap,
    pub original_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct DocumentSerializeRequest {
    pub model: DocumentModel,
    pub source_map: SourceMap,
    pub original_base64: String,
    pub dirty_block_ids: Vec<String>,
    pub touched_block_ids: Vec<String>,
}

pub fn parse_markdown_content(content: &str) -> Result<DocumentParseResponse, String> {
    let bytes = content.as_bytes();
    let parsed = parse_markdown(bytes);
    Ok(DocumentParseResponse {
        model: parsed.model,
        source_map: parsed.source_map,
        original_base64: base64_encode(bytes),
    })
}

pub fn serialize_markdown_request(req: DocumentSerializeRequest) -> Result<String, String> {
    let original = base64_decode(&req.original_base64)?;
    let mut session = DocumentEditSession {
        model: req.model,
        source_map: req.source_map,
        original_bytes: original,
        dirty_blocks: req.dirty_block_ids.into_iter().collect(),
        touched_blocks: req.touched_block_ids.into_iter().collect(),
    };
    let out = session.serialize();
    Ok(String::from_utf8_lossy(&out).into_owned())
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn base64_decode(text: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|e| e.to_string())
}

fn session_from_request(req: &DocumentSerializeRequest) -> Result<DocumentEditSession, String> {
    let original = base64_decode(&req.original_base64)?;
    Ok(DocumentEditSession {
        model: req.model.clone(),
        source_map: req.source_map.clone(),
        original_bytes: original,
        dirty_blocks: req.dirty_block_ids.iter().cloned().collect(),
        touched_blocks: req.touched_block_ids.iter().cloned().collect(),
    })
}

#[derive(Debug, Serialize)]
pub struct DocumentSourceProjectionResponse {
    pub content: String,
}

pub fn source_projection_request(req: DocumentSerializeRequest) -> Result<DocumentSourceProjectionResponse, String> {
    let session = session_from_request(&req)?;
    Ok(DocumentSourceProjectionResponse {
        content: source_projection(&session),
    })
}

#[derive(Debug, Serialize)]
pub struct DocumentMergeSourceResponse {
    pub clean: bool,
    pub conflict_reason: Option<String>,
    pub model: DocumentModel,
    pub source_map: SourceMap,
    pub original_base64: String,
    pub dirty_block_ids: Vec<String>,
    pub touched_block_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentMergeSourceRequest {
    #[serde(flatten)]
    pub session: DocumentSerializeRequest,
    pub new_content: String,
}

pub fn merge_source_request(req: DocumentMergeSourceRequest) -> Result<DocumentMergeSourceResponse, String> {
    let session = session_from_request(&req.session)?;
    let merged = merge_source_text(&session, &req.new_content);
    Ok(DocumentMergeSourceResponse {
        clean: merged.clean,
        conflict_reason: merged.conflict_reason,
        model: merged.session.model,
        source_map: merged.session.source_map,
        original_base64: base64_encode(&merged.session.original_bytes),
        dirty_block_ids: merged.session.dirty_blocks.into_iter().collect(),
        touched_block_ids: merged.session.touched_blocks.into_iter().collect(),
    })
}

pub fn history_reset_for_path(path: &str) -> Result<(), String> {
    let sidecar = sidecar_path_for_file(Path::new(path));
    if sidecar.is_file() {
        std::fs::remove_file(&sidecar).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn history_list_for_path(path: &str) -> Result<Vec<unibrowser_document_model::HistoryLogEntry>, String> {
    let p = Path::new(path);
    list_ops(&unibrowser_document_model::sidecar_path_for_file(p))
}

pub fn history_append_for_path(path: &str, op: DocumentHistoryOp) -> Result<(), String> {
    let p = Path::new(path);
    append_document_op(p, &op, "0.1.0")?;
    Ok(())
}

pub fn history_append_snapshot_for_path(
    path: &str,
    model: DocumentModel,
    source_map: SourceMap,
    original_base64: &str,
) -> Result<(), String> {
    let blob = encode_snapshot_blob(&source_map, original_base64);
    let op = DocumentHistoryOp::Snapshot {
        model,
        source_map_blob: blob,
    };
    history_append_for_path(path, op)
}

pub fn try_undo_markdown_history(
    path: &str,
    redo: &mut Vec<HistoryLogEntry>,
) -> Result<Option<String>, String> {
    let p = Path::new(path);
    if !p.is_file() {
        return Ok(None);
    }
    let sidecar = sidecar_path_for_file(p);
    let popped = pop_last_op(&sidecar)?;
    if popped.is_none() {
        return Ok(None);
    }
    if let Some(entry) = popped {
        redo.push(entry);
    }
    let state = session_at_history_head(p)?.ok_or_else(|| "history head missing".to_string())?;
    Ok(Some(serialize_session_state(&state)?))
}

pub fn history_jump_for_path(path: &str, seq: u64) -> Result<String, String> {
    let p = Path::new(path);
    let state = jump_to_seq(p, seq, "0.1.0")?;
    serialize_session_state(&state)
}

pub fn history_head_session_for_path(path: &str) -> Result<DocumentParseResponse, String> {
    let p = Path::new(path);
    let state = session_at_history_head(p)?.ok_or_else(|| "history head missing".to_string())?;
    Ok(DocumentParseResponse {
        model: state.model,
        source_map: state.source_map,
        original_base64: base64_encode(&state.original_bytes),
    })
}

const CAPTURE_BASELINE_MD: &str = "# Demo markdown

Unibrowser renders Markdown in the document viewer.

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.

![Demo pack image](sample.png)

- **Bold** and _italic_
- [Architecture](../../docs/architecture.md) (link may not resolve in preview; open from repo)

## Next steps

Open `sample.unibrow` for the native project format.
";

const CAPTURE_EDITED_MD: &str = "# Edited heading

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

fn append_snapshot(path: &Path, markdown: &str) -> Result<(), String> {
    let parsed = parse_markdown(markdown.as_bytes());
    let blob = encode_snapshot_blob(&parsed.source_map, &base64_encode(markdown.as_bytes()));
    append_document_op(
        path,
        &DocumentHistoryOp::Snapshot {
            model: parsed.model,
            source_map_blob: blob,
        },
        "0.1.0",
    )?;
    Ok(())
}

/// Seed history capture demo: checkpoint + granular ops (not all snapshots).
pub fn history_seed_capture_demo_for_path(path: &str) -> Result<u64, String> {
    let p = Path::new(path);
    std::fs::write(p, CAPTURE_BASELINE_MD).map_err(|e| e.to_string())?;
    history_reset_for_path(path)?;
    append_snapshot(p, CAPTURE_BASELINE_MD)?;

    let baseline = parse_markdown(CAPTURE_BASELINE_MD.as_bytes());
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
    append_document_op(
        p,
        &DocumentHistoryOp::TextReplace {
            block_id: heading_id,
            from: 0,
            to: heading_text.len(),
            text: "Edited heading".into(),
            marks: vec![],
        },
        "0.1.0",
    )?;

    let head = session_at_history_head(p)?.ok_or_else(|| "head missing after heading op".to_string())?;
    let para_id = first_paragraph_id(&head.model).ok_or_else(|| "paragraph missing".to_string())?;
    append_document_op(
        p,
        &DocumentHistoryOp::TextReplace {
            block_id: para_id,
            from: 0,
            to: 0,
            text: "Edit three\n\n".into(),
            marks: vec![],
        },
        "0.1.0",
    )?;

    let edited = parse_markdown(CAPTURE_EDITED_MD.as_bytes());
    let edited_blob = encode_snapshot_blob(
        &edited.source_map,
        &base64_encode(CAPTURE_EDITED_MD.as_bytes()),
    );
    append_document_op(
        p,
        &DocumentHistoryOp::Snapshot {
            model: edited.model,
            source_map_blob: edited_blob,
        },
        "0.1.0",
    )?;

    Ok(1)
}

pub fn history_trim_before_for_path(path: &str, keep_from_seq: u64) -> Result<(), String> {
    trim_history_before(Path::new(path), keep_from_seq)
}

pub fn history_squash_range_for_path(path: &str, from_seq: u64, to_seq: u64) -> Result<(), String> {
    squash_op_range(Path::new(path), from_seq, to_seq, "0.1.0")
}

pub fn try_redo_markdown_history(
    path: &str,
    redo: &mut Vec<HistoryLogEntry>,
) -> Result<Option<String>, String> {
    let entry = redo.pop().ok_or_else(|| "nothing to redo".to_string())?;
    let p = Path::new(path);
    let sidecar = sidecar_path_for_file(p);
    push_op_entry(&sidecar, entry)?;
    let state = session_at_history_head(p)?.ok_or_else(|| "history head missing".to_string())?;
    Ok(Some(serialize_session_state(&state)?))
}

fn resolve_markdown_path_for_io(markdown_path: &str) -> Result<PathBuf, String> {
    let resolved = crate::launch_args::resolve_capture_path_str(markdown_path)?;
    let p = PathBuf::from(resolved);
    if p.exists() {
        return Ok(p);
    }
    let raw = markdown_path.trim();
    if raw.starts_with("demo-pack/") {
        let alt = crate::launch_args::resolve_capture_path_str(&format!("examples/{}", raw))?;
        let alt_p = PathBuf::from(alt);
        if alt_p.exists() {
            return Ok(alt_p);
        }
    }
    Ok(p)
}

pub fn save_markdown_as_docx(markdown_path: &str, model: &DocumentModel) -> Result<String, String> {
    let p = resolve_markdown_path_for_io(markdown_path)?;
    let parent = p.parent().unwrap_or_else(|| Path::new("."));
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".into());
    let out = parent.join(format!("{}.docx", stem));
    unibrowser_docx::save_docx_from_document_model_with_assets(&out, model, Some(parent))
        .map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().into_owned())
}

pub fn save_docx_from_model(path: &str, model: DocumentModel) -> Result<unibrowser_docx::DocxSaveReport, String> {
    let p = Path::new(path);
    unibrowser_docx::save_docx_from_document_model(p, &model).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct DocumentPromoteOpaqueRequest {
    #[serde(flatten)]
    pub session: DocumentSerializeRequest,
    pub block_id: String,
}

pub fn promote_opaque_request(req: DocumentPromoteOpaqueRequest) -> Result<DocumentParseResponse, String> {
    let mut session = session_from_request(&req.session)?;
    session.promote_opaque_block(&req.block_id)?;
    session.original_bytes = session.serialize();
    Ok(DocumentParseResponse {
        model: session.model,
        source_map: session.source_map,
        original_base64: base64_encode(&session.original_bytes),
    })
}

#[cfg(test)]
mod hist_audit_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use unibrowser_document_model::{append_document_op, encode_snapshot_blob, parse_markdown, DocumentHistoryOp};

    #[test]
    fn hist02_host_undo_redo_matches_disk_head() {
        let dir = tempdir().expect("tempdir");
        let md = dir.path().join("note.md");
        fs::write(&md, "# Hello\n\nWorld.\n").expect("write");
        let parsed_b = parse_markdown(b"# Hello\n\nEdited.\n");
        let blob_b = encode_snapshot_blob(
            &parsed_b.source_map,
            &base64_encode(b"# Hello\n\nEdited.\n"),
        );
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed_b.model,
                source_map_blob: blob_b,
            },
            "0.1.0",
        )
        .expect("append");

        let mut redo = Vec::new();
        let undone = try_undo_markdown_history(md.to_str().unwrap(), &mut redo)
            .expect("undo")
            .expect("content");
        assert!(undone.contains("World."));
        assert!(!undone.contains("Edited."));

        let redone = try_redo_markdown_history(md.to_str().unwrap(), &mut redo)
            .expect("redo")
            .expect("content");
        assert!(redone.contains("Edited."));
    }

    #[test]
    fn hist02_history_jump_returns_restored_markdown() {
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
        .expect("append a");
        let parsed_b = parse_markdown(b"# Hello\n\nEdited.\n");
        let blob_b = encode_snapshot_blob(
            &parsed_b.source_map,
            &base64_encode(b"# Hello\n\nEdited.\n"),
        );
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed_b.model,
                source_map_blob: blob_b,
            },
            "0.1.0",
        )
        .expect("append b");
        let jumped = history_jump_for_path(md.to_str().unwrap(), 1).expect("jump");
        assert!(jumped.contains("World."));
        assert!(!jumped.contains("Edited."));
        let rows = history_list_for_path(md.to_str().unwrap()).expect("list");
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn hist02_history_list_reads_sidecar() {
        let dir = tempdir().expect("tempdir");
        let md = dir.path().join("x.md");
        fs::write(&md, "a\n").expect("write");
        let parsed = parse_markdown(b"a\n");
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model,
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("append");
        let rows = history_list_for_path(md.to_str().unwrap()).expect("list");
        assert_eq!(rows.len(), 1);
    }
}
