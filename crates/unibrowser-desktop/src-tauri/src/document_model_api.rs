use std::path::Path;

use serde::{Deserialize, Serialize};
use unibrowser_document_model::{
    append_document_op, list_ops, merge_source_text, parse_markdown, source_projection,
    DocumentEditSession, DocumentHistoryOp, DocumentModel, SourceMap,
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
