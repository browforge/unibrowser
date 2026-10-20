use crate::history::DocumentHistoryOp;
use crate::history_file::{list_ops, read_header_and_ops, sidecar_path_for_file, write_file_atomic, HistoryLogEntry};
use crate::history_undo::{encode_snapshot_blob, session_state_after_ops, HistorySessionState};
use std::path::Path;

fn renumber_ops(ops: &mut [HistoryLogEntry]) {
    let mut parent = 0u64;
    for (i, op) in ops.iter_mut().enumerate() {
        let seq = (i + 1) as u64;
        op.seq = seq;
        op.parent_seq = parent;
        parent = seq;
    }
}

pub fn session_state_at_seq(authoritative: &Path, target_seq: u64) -> Result<Option<HistorySessionState>, String> {
    let sidecar = sidecar_path_for_file(authoritative);
    let ops = list_ops(&sidecar)?;
    if ops.iter().all(|o| o.seq > target_seq) {
        return Ok(None);
    }
    let prefix: Vec<HistoryLogEntry> = ops.into_iter().filter(|o| o.seq <= target_seq).collect();
    Ok(Some(session_state_after_ops(authoritative, &prefix)?))
}

pub fn jump_to_seq(
    authoritative: &Path,
    target_seq: u64,
    plugin_version: &str,
) -> Result<HistorySessionState, String> {
    let state = session_state_at_seq(authoritative, target_seq)?
        .ok_or_else(|| format!("history seq {} not found", target_seq))?;
    use base64::Engine;
    let original_base64 = base64::engine::general_purpose::STANDARD.encode(&state.original_bytes);
    let blob = encode_snapshot_blob(&state.source_map, &original_base64);
    let op = DocumentHistoryOp::RestoreAt {
        target_seq,
        model: state.model.clone(),
        source_map_blob: blob,
    };
    crate::history_file::append_document_op(authoritative, &op, plugin_version)?;
    Ok(state)
}

pub fn trim_history_before(authoritative: &Path, keep_from_seq: u64) -> Result<(), String> {
    let sidecar = sidecar_path_for_file(authoritative);
    let (header, ops) = read_header_and_ops(&sidecar)?;
    let kept: Vec<HistoryLogEntry> = ops.into_iter().filter(|o| o.seq >= keep_from_seq).collect();
    if kept.is_empty() {
        return Err("no ops remain after trim".into());
    }
    let mut kept = kept;
    renumber_ops(&mut kept);
    write_file_atomic(&sidecar, &header, &kept)
}

pub fn squash_op_range(
    authoritative: &Path,
    from_seq: u64,
    to_seq: u64,
    plugin_version: &str,
) -> Result<(), String> {
    if from_seq > to_seq {
        return Err("invalid squash range".into());
    }
    let sidecar = sidecar_path_for_file(authoritative);
    let (header, ops) = read_header_and_ops(&sidecar)?;
    if ops.is_empty() {
        return Err("empty history".into());
    }
    let state = session_state_at_seq(authoritative, to_seq)?
        .ok_or_else(|| "squash range exceeds log".to_string())?;
    use base64::Engine;
    let original_base64 = base64::engine::general_purpose::STANDARD.encode(&state.original_bytes);
    let blob = encode_snapshot_blob(&state.source_map, &original_base64);
    let payload = serde_json::to_value(&DocumentHistoryOp::Snapshot {
        model: state.model,
        source_map_blob: blob,
    })
    .map_err(|e| e.to_string())?;
    let op_type = "unibrowser.history.snapshot".to_string();

    let mut new_ops: Vec<HistoryLogEntry> = ops.iter().filter(|o| o.seq < from_seq).cloned().collect();
    let parent_seq = new_ops.last().map(|o| o.seq).unwrap_or(0);
    let seq = parent_seq + 1;
    new_ops.push(HistoryLogEntry {
        op_id: format!("squash-{}-{}", from_seq, to_seq),
        seq,
        parent_seq,
        timestamp: crate::history_file::now_rfc3339(),
        plugin_id: "base-text".into(),
        plugin_version: plugin_version.into(),
        op_type,
        op_schema_version: 1,
        payload,
    });
    new_ops.extend(ops.iter().filter(|o| o.seq > to_seq).cloned());
    renumber_ops(&mut new_ops);
    write_file_atomic(&sidecar, &header, &new_ops)
}
