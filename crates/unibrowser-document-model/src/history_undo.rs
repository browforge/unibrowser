use crate::history::{apply_history_op, DocumentHistoryOp};
use crate::history_file::{list_ops, sidecar_path_for_file, HistoryLogEntry};
use crate::markdown::parse_markdown;
use crate::model::{DocumentModel, SourceMap};
use crate::session::DocumentEditSession;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HistorySessionState {
    pub model: DocumentModel,
    pub source_map: SourceMap,
    pub original_bytes: Vec<u8>,
}

#[derive(Debug, serde::Deserialize)]
struct SnapshotBlob {
    source_map: SourceMap,
    original_base64: String,
}

pub fn decode_snapshot_blob(blob: &str) -> Result<(SourceMap, Vec<u8>), String> {
    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(blob)
        .map_err(|e| e.to_string())?;
    let parsed: SnapshotBlob = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&parsed.original_base64)
        .map_err(|e| e.to_string())?;
    Ok((parsed.source_map, bytes))
}

pub fn encode_snapshot_blob(source_map: &SourceMap, original_base64: &str) -> String {
    use base64::Engine;
    let json = serde_json::json!({
        "source_map": source_map,
        "original_base64": original_base64,
    });
    base64::engine::general_purpose::STANDARD.encode(json.to_string())
}

fn op_from_entry(entry: &HistoryLogEntry) -> Result<DocumentHistoryOp, String> {
    serde_json::from_value(entry.payload.clone()).map_err(|e| e.to_string())
}

pub fn session_state_after_bytes(bytes: &[u8], ops: &[HistoryLogEntry]) -> Result<HistorySessionState, String> {
    let parsed = parse_markdown(bytes);
    let mut model = parsed.model;
    let mut source_map = parsed.source_map;
    let mut original_bytes = bytes.to_vec();
    for entry in ops {
        let op = op_from_entry(entry)?;
        let blob = match &op {
            DocumentHistoryOp::Snapshot { source_map_blob, .. } => Some(source_map_blob.as_str()),
            DocumentHistoryOp::RestoreAt { source_map_blob, .. } => Some(source_map_blob.as_str()),
            _ => None,
        };
        if let Some(source_map_blob) = blob {
            if !source_map_blob.is_empty() {
                let (sm, ob) = decode_snapshot_blob(source_map_blob)?;
                source_map = sm;
                original_bytes = ob;
            }
        }
        apply_history_op(&mut model, &op)?;
    }
    Ok(HistorySessionState {
        model,
        source_map,
        original_bytes,
    })
}

pub fn session_state_after_ops(authoritative: &Path, ops: &[HistoryLogEntry]) -> Result<HistorySessionState, String> {
    let bytes = std::fs::read(authoritative).map_err(|e| e.to_string())?;
    session_state_after_bytes(&bytes, ops)
}

pub fn session_at_history_head(authoritative: &Path) -> Result<Option<HistorySessionState>, String> {
    let sidecar = sidecar_path_for_file(authoritative);
    let ops = list_ops(&sidecar)?;
    if ops.is_empty() {
        let bytes = std::fs::read(authoritative).map_err(|e| e.to_string())?;
        let parsed = parse_markdown(&bytes);
        return Ok(Some(HistorySessionState {
            model: parsed.model,
            source_map: parsed.source_map,
            original_bytes: bytes,
        }));
    }
    Ok(Some(session_state_after_ops(authoritative, &ops)?))
}

pub fn serialize_session_state(state: &HistorySessionState) -> Result<String, String> {
    let session = DocumentEditSession {
        model: state.model.clone(),
        source_map: state.source_map.clone(),
        original_bytes: state.original_bytes.clone(),
        dirty_blocks: Default::default(),
        touched_blocks: Default::default(),
    };
    let out = session.serialize();
    Ok(String::from_utf8_lossy(&out).into_owned())
}
