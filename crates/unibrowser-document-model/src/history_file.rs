use crate::history::DocumentHistoryOp;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const LOG_MARKER: &str = "--- UNIBROW HISTORY LOG ---";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryLogEntry {
    pub op_id: String,
    pub seq: u64,
    pub parent_seq: u64,
    pub timestamp: String,
    pub plugin_id: String,
    pub plugin_version: String,
    pub op_type: String,
    pub op_schema_version: u32,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryHead {
    pub head_seq: u64,
    pub redo_stack: Vec<u64>,
}

pub fn sidecar_path_for_file(authoritative: &Path) -> PathBuf {
    let name = authoritative
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".into());
    authoritative
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}.unibrow-history", name))
}

fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}Z", secs)
}

fn read_header_and_ops(path: &Path) -> Result<(String, Vec<HistoryLogEntry>), String> {
    if !path.is_file() {
        return Ok((default_header(), Vec::new()));
    }
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut header_lines = Vec::new();
    let mut ops = Vec::new();
    let mut in_log = false;
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if !in_log {
            if line.trim() == LOG_MARKER {
                in_log = true;
                continue;
            }
            header_lines.push(line);
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<HistoryLogEntry>(&line) {
            Ok(entry) => ops.push(entry),
            Err(_) => break,
        }
    }
    let header = if header_lines.is_empty() {
        default_header()
    } else {
        header_lines.join("\n")
    };
    Ok((header, ops))
}

fn default_header() -> String {
    "historyFormat: 1\nplugins:\n  base-text@0.1.0:\n    op_types:\n      block.set_style: 1\n      text.replace: 1\n      opaque.replace: 1\n"
        .to_string()
}

fn write_file_atomic(path: &Path, header: &str, ops: &[HistoryLogEntry]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("unibrow-history.part");
    let mut file = fs::File::create(&tmp).map_err(|e| e.to_string())?;
    file.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
    if !header.ends_with('\n') {
        file.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    file.write_all(LOG_MARKER.as_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    for op in ops {
        let line = serde_json::to_string(op).map_err(|e| e.to_string())?;
        file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_head(path: &Path) -> Result<HistoryHead, String> {
    let (_, ops) = read_header_and_ops(path)?;
    let head_seq = ops.last().map(|o| o.seq).unwrap_or(0);
    Ok(HistoryHead {
        head_seq,
        redo_stack: Vec::new(),
    })
}

pub fn list_ops(path: &Path) -> Result<Vec<HistoryLogEntry>, String> {
    let (_, ops) = read_header_and_ops(path)?;
    Ok(ops)
}

pub fn append_document_op(
    authoritative: &Path,
    op: &DocumentHistoryOp,
    plugin_version: &str,
) -> Result<HistoryLogEntry, String> {
    let sidecar = sidecar_path_for_file(authoritative);
    let (header, mut ops) = read_header_and_ops(&sidecar)?;
    let parent_seq = ops.last().map(|o| o.seq).unwrap_or(0);
    let seq = parent_seq + 1;
    let (op_type, payload) = history_op_payload(op)?;
    let entry = HistoryLogEntry {
        op_id: format!("op-{}-{}", seq, now_rfc3339()),
        seq,
        parent_seq,
        timestamp: now_rfc3339(),
        plugin_id: "base-text".into(),
        plugin_version: plugin_version.into(),
        op_type,
        op_schema_version: 1,
        payload,
    };
    ops.push(entry.clone());
    write_file_atomic(&sidecar, &header, &ops)?;
    Ok(entry)
}

fn history_op_payload(op: &DocumentHistoryOp) -> Result<(String, serde_json::Value), String> {
    let value = serde_json::to_value(op).map_err(|e| e.to_string())?;
    let op_type = value
        .get("op_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    Ok((op_type, value))
}

pub fn truncate_head(path: &Path, target_seq: u64) -> Result<(), String> {
    let (header, ops) = read_header_and_ops(path)?;
    let kept: Vec<HistoryLogEntry> = ops.into_iter().filter(|o| o.seq <= target_seq).collect();
    write_file_atomic(path, &header, &kept)
}

pub fn ensure_sidecar_exists(authoritative: &Path) -> Result<PathBuf, String> {
    let sidecar = sidecar_path_for_file(authoritative);
    if !sidecar.is_file() {
        write_file_atomic(&sidecar, &default_header(), &[])?;
    }
    Ok(sidecar)
}

pub fn pop_last_op(sidecar: &Path) -> Result<Option<HistoryLogEntry>, String> {
    let (header, mut ops) = read_header_and_ops(sidecar)?;
    if ops.is_empty() {
        return Ok(None);
    }
    let popped = ops.pop();
    write_file_atomic(sidecar, &header, &ops)?;
    Ok(popped)
}

pub fn push_op_entry(sidecar: &Path, entry: HistoryLogEntry) -> Result<(), String> {
    let (header, mut ops) = read_header_and_ops(sidecar)?;
    ops.push(entry);
    write_file_atomic(sidecar, &header, &ops)?;
    Ok(())
}

pub fn touch_sidecar(authoritative: &Path) -> Result<(), String> {
    let sidecar = sidecar_path_for_file(authoritative);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&sidecar)
        .map_err(|e| e.to_string())?;
    Ok(())
}
