use serde::{Deserialize, Serialize};
use unibrowser_document_model::{
    parse_markdown, DocumentEditSession, DocumentModel, SourceMap,
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
