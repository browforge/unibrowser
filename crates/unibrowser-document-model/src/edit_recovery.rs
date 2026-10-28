//! Append-only edit journals for crash recovery (REC-01).
//!
//! Journals live outside the user's project. Each record is length-prefixed with a CRC32
//! trailer so a torn final record is ignored on read.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const MAGIC: &[u8] = b"UB-EDIT-JOURNAL\0";
const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JournalHeader {
    pub format_version: u32,
    pub path: String,
    pub backend_id: String,
    pub journal_id: String,
    pub opened_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JournalRecord {
    Baseline { content_b64: String },
    Checkpoint { seq: u64, content_b64: String },
}

#[derive(Debug, Clone)]
pub struct RecoveredEdit {
    pub header: JournalHeader,
    pub content: Vec<u8>,
    pub checkpoint_seq: u64,
}

#[derive(Debug, Clone)]
pub struct PendingJournal {
    pub journal_id: String,
    pub path: String,
    pub backend_id: String,
}

fn journal_path(dir: &Path, journal_id: &str) -> PathBuf {
    dir.join(format!("{}.journal", journal_id))
}

pub fn journal_id_for_path(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = 0xedb8_8320 ^ (crc >> 1);
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xffff_ffff
}

fn append_record(file: &mut File, record: &JournalRecord) -> Result<(), String> {
    let body = serde_json::to_vec(record).map_err(|e| e.to_string())?;
    let len = body.len() as u32;
    let sum = crc32(&body);
    file.write_all(&len.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&body).map_err(|e| e.to_string())?;
    file.write_all(&sum.to_le_bytes()).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(())
}

fn read_records(bytes: &[u8]) -> (Vec<JournalRecord>, bool) {
    let mut out = Vec::new();
    let mut off = 0usize;
    let mut torn = false;
    while off + 8 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[off..off + 4].try_into().expect("len")) as usize;
        off += 4;
        if off + len + 4 > bytes.len() {
            torn = true;
            break;
        }
        let body = &bytes[off..off + len];
        off += len;
        let sum = u32::from_le_bytes(bytes[off..off + 4].try_into().expect("sum"));
        off += 4;
        if crc32(body) != sum {
            torn = true;
            break;
        }
        match serde_json::from_slice::<JournalRecord>(body) {
            Ok(rec) => out.push(rec),
            Err(_) => {
                torn = true;
                break;
            }
        }
    }
    (out, torn)
}

fn read_journal_file(path: &Path) -> Result<(JournalHeader, Vec<JournalRecord>, bool), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    if data.len() < MAGIC.len() + 4 {
        return Err("journal truncated".into());
    }
    if &data[..MAGIC.len()] != MAGIC {
        return Err("journal magic mismatch".into());
    }
    let header_len = u32::from_le_bytes(data[MAGIC.len()..MAGIC.len() + 4].try_into().expect("hl"))
        as usize;
    let start = MAGIC.len() + 4;
    let end = start + header_len;
    if end > data.len() {
        return Err("journal header truncated".into());
    }
    let header: JournalHeader =
        serde_json::from_slice(&data[start..end]).map_err(|e| e.to_string())?;
    let (records, torn) = read_records(&data[end..]);
    Ok((header, records, torn))
}

fn write_header(file: &mut File, header: &JournalHeader) -> Result<(), String> {
    file.write_all(MAGIC).map_err(|e| e.to_string())?;
    let header_bytes = serde_json::to_vec(header).map_err(|e| e.to_string())?;
    let len = header_bytes.len() as u32;
    file.write_all(&len.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&header_bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(())
}

fn b64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn b64_decode(text: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|e| e.to_string())
}

/// Start a journal after the last known saved bytes (call on first dirty edit or session open).
pub fn begin_journal(
    dir: &Path,
    path: &str,
    backend_id: &str,
    baseline: &[u8],
) -> Result<JournalHeader, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let journal_id = journal_id_for_path(path);
    let dest = journal_path(dir, &journal_id);
    let header = JournalHeader {
        format_version: FORMAT_VERSION,
        path: path.to_string(),
        backend_id: backend_id.to_string(),
        journal_id: journal_id.clone(),
        opened_at_ms: now_ms(),
    };
    let mut file = File::create(&dest).map_err(|e| e.to_string())?;
    write_header(&mut file, &header)?;
    append_record(
        &mut file,
        &JournalRecord::Baseline {
            content_b64: b64_encode(baseline),
        },
    )?;
    Ok(header)
}

/// Append a checkpoint when the buffer changes (caller may debounce).
pub fn append_checkpoint(dir: &Path, path: &str, content: &[u8]) -> Result<u64, String> {
    let journal_id = journal_id_for_path(path);
    let dest = journal_path(dir, &journal_id);
    if !dest.is_file() {
        return Err("journal missing".into());
    }
    let (header, records, _) = read_journal_file(&dest)?;
    if header.path != path {
        return Err("journal path mismatch".into());
    }
    let next_seq = records
        .iter()
        .filter_map(|r| match r {
            JournalRecord::Checkpoint { seq, .. } => Some(*seq),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        + 1;
    let mut file = OpenOptions::new()
        .append(true)
        .open(&dest)
        .map_err(|e| e.to_string())?;
    append_record(
        &mut file,
        &JournalRecord::Checkpoint {
            seq: next_seq,
            content_b64: b64_encode(content),
        },
    )?;
    Ok(next_seq)
}

pub fn clear_journal(dir: &Path, path: &str) -> Result<(), String> {
    let journal_id = journal_id_for_path(path);
    let dest = journal_path(dir, &journal_id);
    if dest.is_file() {
        fs::remove_file(&dest).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn list_pending_journals(dir: &Path) -> Result<Vec<PendingJournal>, String> {
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("journal") {
            continue;
        }
        if let Ok((header, records, _)) = read_journal_file(&path) {
            if records.iter().any(|r| matches!(r, JournalRecord::Checkpoint { .. }))
                && recovered_bytes(&records).is_some()
            {
                out.push(PendingJournal {
                    journal_id: header.journal_id,
                    path: header.path,
                    backend_id: header.backend_id,
                });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn recovered_bytes(records: &[JournalRecord]) -> Option<(Vec<u8>, u64)> {
    let mut baseline: Option<Vec<u8>> = None;
    let mut last: Option<(u64, Vec<u8>)> = None;
    for rec in records {
        match rec {
            JournalRecord::Baseline { content_b64 } => {
                baseline = b64_decode(content_b64).ok();
            }
            JournalRecord::Checkpoint { seq, content_b64 } => {
                if let Ok(bytes) = b64_decode(content_b64) {
                    last = Some((*seq, bytes));
                }
            }
        }
    }
    let (seq, content) = if let Some((s, c)) = last {
        (s, c)
    } else if let Some(b) = baseline {
        (0, b)
    } else {
        return None;
    };
    Some((content, seq))
}

pub fn recover_from_journal(dir: &Path, journal_id: &str) -> Result<RecoveredEdit, String> {
    let dest = journal_path(dir, journal_id);
    let (header, records, _) = read_journal_file(&dest)?;
    let (content, seq) = recovered_bytes(&records).ok_or("journal empty")?;
    Ok(RecoveredEdit {
        header,
        content,
        checkpoint_seq: seq,
    })
}

pub fn disk_matches_recovered(dir: &Path, journal_id: &str, disk: &[u8]) -> Result<bool, String> {
    let rec = recover_from_journal(dir, journal_id)?;
    Ok(rec.content == disk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ub-edit-rec-{}-{}",
            label,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn rec01_kill_mid_session_recovers_identical_content() {
        let dir = temp_dir("kill");
        let _ = fs::remove_dir_all(&dir);
        let doc_path = "/tmp/recover-me.md";
        let baseline = b"hello";
        let edited = b"hello world";
        begin_journal(&dir, doc_path, "base-text", baseline).expect("begin");
        append_checkpoint(&dir, doc_path, edited).expect("checkpoint");
        // simulate kill: no clear_journal
        let rec = recover_from_journal(&dir, &journal_id_for_path(doc_path)).expect("recover");
        assert_eq!(rec.content, edited);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rec01_torn_tail_ignores_partial_record() {
        let dir = temp_dir("torn");
        let _ = fs::remove_dir_all(&dir);
        let doc_path = "/tmp/torn.md";
        begin_journal(&dir, doc_path, "base-text", b"v1").expect("begin");
        append_checkpoint(&dir, doc_path, b"v2").expect("cp1");
        let dest = journal_path(&dir, &journal_id_for_path(doc_path));
        let mut bytes = fs::read(&dest).expect("read");
        bytes.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0xff]);
        fs::write(&dest, &bytes).expect("torn write");
        let rec = recover_from_journal(&dir, &journal_id_for_path(doc_path)).expect("recover");
        assert_eq!(rec.content, b"v2");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rec01_clean_save_clears_journal() {
        let dir = temp_dir("clear");
        let _ = fs::remove_dir_all(&dir);
        let doc_path = "/tmp/clear.md";
        begin_journal(&dir, doc_path, "base-text", b"a").expect("begin");
        append_checkpoint(&dir, doc_path, b"b").expect("cp");
        clear_journal(&dir, doc_path).expect("clear");
        let pending = list_pending_journals(&dir).expect("list");
        assert!(pending.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
