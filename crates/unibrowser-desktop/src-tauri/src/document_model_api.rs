use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use unibrowser_document_model::{
    append_document_op, encode_snapshot_blob, list_ops, merge_source_text, parse_markdown,
    pop_last_op, push_op_entry, serialize_session_state, session_at_history_head,
    sidecar_path_for_file, source_projection, DocumentEditSession, DocumentHistoryOp,
    DocumentModel, HistoryLogEntry, SourceMap,
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
