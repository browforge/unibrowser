use crate::history_file::{list_ops, read_header_and_ops, sidecar_path_for_file, HistoryLogEntry};
use crate::history_undo::{session_state_after_bytes, HistorySessionState};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryReplayBlock {
    pub plugin_id: String,
    pub required_version: String,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HistoryReplayReport {
    pub state: HistorySessionState,
    pub applied_seq: u64,
    pub blocked: Option<HistoryReplayBlock>,
    pub read_only: bool,
}

fn authoritative_bootstrap_bytes(authoritative: &Path) -> Result<Vec<u8>, String> {
    let lower = authoritative
        .to_string_lossy()
        .to_ascii_lowercase();
    if lower.ends_with(".md") || lower.ends_with(".markdown") {
        std::fs::read(authoritative).map_err(|e| e.to_string())
    } else {
        Ok(Vec::new())
    }
}

fn plugin_satisfies_op(
    catalog: &HashMap<String, String>,
    entry: &HistoryLogEntry,
) -> Option<HistoryReplayBlock> {
    let installed = catalog.get(&entry.plugin_id);
    match installed {
        None => Some(HistoryReplayBlock {
            plugin_id: entry.plugin_id.clone(),
            required_version: entry.plugin_version.clone(),
            installed_version: None,
        }),
        Some(v) if v != &entry.plugin_version => Some(HistoryReplayBlock {
            plugin_id: entry.plugin_id.clone(),
            required_version: entry.plugin_version.clone(),
            installed_version: Some(v.clone()),
        }),
        _ => None,
    }
}

pub fn replay_session_with_catalog(
    authoritative: &Path,
    catalog: &HashMap<String, String>,
) -> Result<HistoryReplayReport, String> {
    let sidecar = sidecar_path_for_file(authoritative);
    let (header, ops) = read_header_and_ops(&sidecar)?;
    let bootstrap = authoritative_bootstrap_bytes(authoritative)?;
    if ops.is_empty() {
        let parsed = crate::markdown::parse_markdown(&bootstrap);
        return Ok(HistoryReplayReport {
            state: HistorySessionState {
                model: parsed.model,
                source_map: parsed.source_map,
                original_bytes: bootstrap,
            },
            applied_seq: 0,
            blocked: header_pin_mismatch(&header, catalog),
            read_only: header_pin_mismatch(&header, catalog).is_some(),
        });
    }

    let mut applied: Vec<HistoryLogEntry> = Vec::new();
    let mut blocked: Option<HistoryReplayBlock> = None;
    for entry in &ops {
        if let Some(issue) = plugin_satisfies_op(catalog, entry) {
            blocked = Some(issue);
            break;
        }
        applied.push(entry.clone());
    }

    let state = session_state_after_bytes(&bootstrap, &applied)?;
    let applied_seq = applied.last().map(|o| o.seq).unwrap_or(0);
    let read_only = blocked.is_some();
    Ok(HistoryReplayReport {
        state,
        applied_seq,
        blocked,
        read_only,
    })
}

pub fn header_pin_mismatch(
    header: &str,
    catalog: &HashMap<String, String>,
) -> Option<HistoryReplayBlock> {
    use crate::history_pins::pinned_plugin_versions;
    for (plugin_id, required) in pinned_plugin_versions(header) {
        let installed = catalog.get(&plugin_id);
        match installed {
            None => {
                return Some(HistoryReplayBlock {
                    plugin_id,
                    required_version: required,
                    installed_version: None,
                });
            }
            Some(v) if v != &required => {
                return Some(HistoryReplayBlock {
                    plugin_id,
                    required_version: required,
                    installed_version: Some(v.clone()),
                });
            }
            _ => {}
        }
    }
    None
}

pub fn copy_history_sidecar(from_authoritative: &Path, to_authoritative: &Path) -> Result<bool, String> {
    let src = sidecar_path_for_file(from_authoritative);
    if !src.is_file() {
        return Ok(false);
    }
    let dst = sidecar_path_for_file(to_authoritative);
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history_file::append_document_op;
    use crate::history_pins::parse_pin_header;
    use crate::{parse_markdown, DocumentHistoryOp};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn hist03_header_updates_on_new_plugin_version() {
        let dir = tempdir().expect("tempdir");
        let md = dir.path().join("note.md");
        fs::write(&md, "a\n").expect("write");
        let parsed = parse_markdown(b"a\n");
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model.clone(),
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("append");
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model.clone(),
                source_map_blob: String::new(),
            },
            "9.9.9",
        )
        .expect("append v2");
        let raw = fs::read_to_string(sidecar_path_for_file(&md)).expect("read");
        assert!(
            raw.contains("base-text@9.9.9"),
            "pin header must record new exact version"
        );
        let pin = parse_pin_header(&raw);
        assert!(pin.plugins.contains_key("base-text@9.9.9"));
    }

    #[test]
    fn hist03_replay_stops_when_plugin_version_missing() {
        let dir = tempdir().expect("tempdir");
        let md = dir.path().join("note.md");
        fs::write(&md, "a\n").expect("write");
        let parsed = parse_markdown(b"a\n");
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model.clone(),
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("first");
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model,
                source_map_blob: String::new(),
            },
            "99.0.0",
        )
        .expect("second");
        let catalog = HashMap::from([("base-text".to_string(), "0.1.0".to_string())]);
        let report = replay_session_with_catalog(&md, &catalog).expect("replay");
        assert_eq!(report.applied_seq, 1);
        assert!(report.read_only);
        let block = report.blocked.expect("blocked");
        assert_eq!(block.plugin_id, "base-text");
        assert_eq!(block.required_version, "99.0.0");
    }

    #[test]
    fn hist05_copy_sidecar_to_docx_path() {
        let dir = tempdir().expect("tempdir");
        let md = dir.path().join("report.md");
        fs::write(&md, "# Title\n").expect("write");
        let parsed = parse_markdown(b"# Title\n");
        append_document_op(
            &md,
            &DocumentHistoryOp::Snapshot {
                model: parsed.model,
                source_map_blob: String::new(),
            },
            "0.1.0",
        )
        .expect("append");
        let docx = dir.path().join("report.docx");
        fs::write(&docx, b"PK").expect("stub docx");
        assert!(copy_history_sidecar(&md, &docx).expect("copy"));
        let ops = list_ops(&sidecar_path_for_file(&docx)).expect("list");
        assert_eq!(ops.len(), 1);
    }
}
