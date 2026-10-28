//! External media refs, Collect into project, referenced-part index ([PKG-08], [REC-02]).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::opc_bundle::{content_hash_hex, ensure_unpacked_package_layout, is_content_hash_asset_name};
use crate::{
    write_manifest, ProjectError, ProjectManifest, ProjectPackage, DIR_ASSETS, DIR_REFS,
    HISTORY_FILE, MANIFEST_FILE, PROJECT_FILE,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaRefRecord {
    pub id: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_hash: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub struct CollectResult {
    pub embedded_hashes: Vec<String>,
    pub nodes_updated: usize,
}

/// JSON files under `refs/` describing linked external media.
pub fn list_media_refs(root: &Path) -> Result<Vec<MediaRefRecord>, ProjectError> {
    let refs_dir = root.join(DIR_REFS);
    if !refs_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&refs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let data = fs::read_to_string(&path)?;
        let mut rec: MediaRefRecord = serde_json::from_str(&data)?;
        if rec.asset_hash.is_none() {
            rec.asset_hash = rec.content_hash.clone();
        }
        out.push(rec);
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

fn write_media_ref(root: &Path, rec: &MediaRefRecord) -> Result<(), ProjectError> {
    fs::create_dir_all(root.join(DIR_REFS))?;
    let path = root.join(DIR_REFS).join(format!("{}.json", rec.id));
    let data = serde_json::to_string_pretty(rec)?;
    unibrowser_fs::atomic_write(&path, data.as_bytes()).map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

/// Link an external file without copying bytes into `assets/` yet.
pub fn add_media_ref(
    package: &mut ProjectPackage,
    source: &Path,
    label: &str,
) -> Result<MediaRefRecord, ProjectError> {
    if !source.is_file() {
        return Err(ProjectError::Msg("ref source is not a file".into()));
    }
    let id = format!(
        "ref-{}",
        list_media_refs(&package.root)?.len() + 1
    );
    let bytes = fs::read(source)?;
    let hash = content_hash_hex(&bytes);
    let rec = MediaRefRecord {
        id,
        path: source.to_string_lossy().to_string(),
        content_hash: Some(hash.clone()),
        asset_hash: Some(hash),
        label: label.into(),
    };
    write_media_ref(&package.root, &rec)?;
    Ok(rec)
}

fn ref_json_path(root: &Path, ref_id: &str) -> PathBuf {
    root.join(DIR_REFS).join(format!("{}.json", ref_id))
}

/// Copy referenced bytes into `assets/<sha256>` and attach to a new board node.
pub fn collect_media_ref(
    package: &mut ProjectPackage,
    ref_id: &str,
) -> Result<CollectResult, ProjectError> {
    let refs = list_media_refs(&package.root)?;
    let rec = refs
        .iter()
        .find(|r| r.id == ref_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown ref {}", ref_id)))?;
    let source = Path::new(&rec.path);
    if !source.is_file() {
        return Err(ProjectError::Msg("referenced file missing on disk".into()));
    }
    ensure_unpacked_package_layout(&package.root, &mut package.manifest)?;
    let bytes = fs::read(source)?;
    let hash = rec
        .asset_hash
        .clone()
        .or_else(|| rec.content_hash.clone())
        .unwrap_or_else(|| content_hash_hex(&bytes));
    let dest = package.root.join(DIR_ASSETS).join(&hash);
    if !dest.is_file() {
        fs::write(&dest, &bytes)?;
    }
    let node_id = format!("node-{}", package.manifest.nodes.len() + 1);
    package.manifest.nodes.push(crate::ProjectNode {
        id: node_id,
        node_type: crate::guess_asset_node_type(source).into(),
        label: rec.label.clone(),
        asset: Some(hash.clone()),
        asset_hash: Some(hash.clone()),
        x: 0.0,
        y: 0.0,
        color: None,
    });
    write_manifest(&package.root, &package.manifest)?;
    fs::remove_file(ref_json_path(&package.root, ref_id))?;
    Ok(CollectResult {
        embedded_hashes: vec![hash],
        nodes_updated: 1,
    })
}

/// Collect every pending ref in `refs/`.
pub fn collect_all_media_refs(package: &mut ProjectPackage) -> Result<CollectResult, ProjectError> {
    let ids: Vec<String> = list_media_refs(&package.root)?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let mut embedded = Vec::new();
    let mut nodes = 0usize;
    for id in ids {
        let r = collect_media_ref(package, &id)?;
        embedded.extend(r.embedded_hashes);
        nodes += r.nodes_updated;
    }
    Ok(CollectResult {
        embedded_hashes: embedded,
        nodes_updated: nodes,
    })
}

/// Content hashes referenced explicitly on `project.json` nodes.
pub fn referenced_asset_hashes_from_manifest(manifest: &ProjectManifest) -> HashSet<String> {
    let mut out = HashSet::new();
    for node in &manifest.nodes {
        if let Some(hash) = &node.asset_hash {
            if is_content_hash_asset_name(hash) {
                out.insert(hash.clone());
            }
        }
    }
    out
}

fn explicit_asset_hash_from_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) if is_content_hash_asset_name(s) => Some(s.clone()),
        _ => None,
    }
}

fn collect_asset_hashes_from_payload(value: &serde_json::Value, out: &mut HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(v) = map.get("asset_hash") {
                if let Some(h) = explicit_asset_hash_from_value(v) {
                    out.insert(h);
                }
            }
            for (k, v) in map {
                if k != "asset_hash" {
                    collect_asset_hashes_from_payload(v, out);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_asset_hashes_from_payload(v, out);
            }
        }
        _ => {}
    }
}

/// Asset hashes referenced by explicit `asset_hash` fields in `.unibrow-history` ops.
pub fn referenced_asset_hashes_from_history(
    root: &Path,
    retain_from_seq: Option<u64>,
) -> Result<HashSet<String>, ProjectError> {
    let path = root.join(HISTORY_FILE);
    if !path.is_file() {
        return Ok(HashSet::new());
    }
    let (_, ops) =
        unibrowser_document_model::history_file::read_header_and_ops(&path).map_err(ProjectError::Msg)?;
    let mut out = HashSet::new();
    for op in ops {
        if retain_from_seq.is_some_and(|min| op.seq < min) {
            continue;
        }
        collect_asset_hashes_from_payload(&op.payload, &mut out);
    }
    Ok(out)
}

/// Every `assets/<sha256>` hash referenced from the manifest and history log.
pub fn list_referenced_content_hashes(root: &Path, manifest: &ProjectManifest) -> HashSet<String> {
    let mut set = referenced_asset_hashes_from_manifest(manifest);
    if let Ok(h) = referenced_asset_hashes_from_history(root, None) {
        set.extend(h);
    }
    set
}

/// Relative package parts referenced for save/GC (metadata + content hashes).
pub fn list_referenced_package_parts(
    root: &Path,
    manifest: &ProjectManifest,
) -> HashSet<String> {
    let mut parts = HashSet::new();
    parts.insert(PROJECT_FILE.to_string());
    if root.join(MANIFEST_FILE).is_file() {
        parts.insert(MANIFEST_FILE.to_string());
    }
    parts.insert(HISTORY_FILE.to_string());
    for hash in list_referenced_content_hashes(root, manifest) {
        parts.insert(format!("assets/{}", hash));
    }
    parts
}

/// Remove unreferenced `assets/<sha256>` blobs; never delete hashes still in history.
pub fn gc_content_addressed_assets(
    root: &Path,
    manifest: &ProjectManifest,
    retain_from_seq: Option<u64>,
) -> Result<Vec<String>, ProjectError> {
    let assets_dir = root.join(DIR_ASSETS);
    if !assets_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut keep = referenced_asset_hashes_from_manifest(manifest);
    keep.extend(referenced_asset_hashes_from_history(root, retain_from_seq)?);
    let mut removed = Vec::new();
    for entry in fs::read_dir(&assets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_content_hash_asset_name(&name) {
            continue;
        }
        if !keep.contains(&name) {
            fs::remove_file(&path)?;
            removed.push(name);
        }
    }
    removed.sort();
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{add_asset_copy, add_media_ref, create_package};

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ub-refs-{}-{}", label, std::process::id()))
    }

    #[test]
    fn pkg08_collect_embeds_ref_and_gc_drops_orphans() {
        let dir = temp_root("collect-gc");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Refs").expect("create");
        let external = dir.join("clip.bin");
        fs::write(&external, b"video-bytes").expect("write");
        let _ref = add_media_ref(&mut pkg, &external, "Clip").expect("ref");
        let collected = collect_all_media_refs(&mut pkg).expect("collect");
        assert_eq!(collected.embedded_hashes.len(), 1);
        assert!(list_media_refs(&dir).expect("list").is_empty());

        let hash = collected.embedded_hashes[0].clone();
        let orphan = dir.join(DIR_ASSETS).join(
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        fs::write(&orphan, b"x").expect("orphan");
        pkg.manifest.nodes.clear();
        write_manifest(&dir, &pkg.manifest).expect("clear nodes");
        let removed = gc_content_addressed_assets(&dir, &pkg.manifest, None).expect("gc");
        let orphan_name = orphan.file_name().unwrap().to_string_lossy().to_string();
        assert!(removed.contains(&orphan_name));
        assert!(!dir.join(DIR_ASSETS).join(&hash).is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rec02_gc_keeps_explicit_asset_hash_not_decoy_string() {
        let dir = temp_root("decoy");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Decoy").expect("create");
        let src = dir.join("keep.bin");
        fs::write(&src, b"keep-me").expect("write");
        add_asset_copy(&mut pkg, &src, "file", "Keep").expect("asset");
        let hash = pkg.manifest.nodes[0].asset_hash.clone().expect("hash");
        pkg.manifest.nodes.clear();
        write_manifest(&dir, &pkg.manifest).expect("clear nodes");
        let decoy = "1111111111111111111111111111111111111111111111111111111111111111";
        let hist_path = dir.join(HISTORY_FILE);
        let (_, ops) =
            unibrowser_document_model::history_file::read_header_and_ops(&hist_path).expect("read");
        let entry = unibrowser_document_model::history_file::HistoryLogEntry {
            op_id: "note-1".into(),
            seq: ops.last().map(|o| o.seq + 1).unwrap_or(1),
            parent_seq: ops.last().map(|o| o.seq).unwrap_or(0),
            timestamp: unibrowser_document_model::history_file::now_rfc3339(),
            plugin_id: "base-text".into(),
            plugin_version: "0.1.0".into(),
            op_type: "base-text.text.replace".into(),
            op_schema_version: 1,
            payload: serde_json::json!({
                "text": decoy,
                "note": "hash-like string must not pin assets"
            }),
        };
        unibrowser_document_model::history_file::push_op_entry(&hist_path, entry).expect("push");
        let keep_entry = unibrowser_document_model::history_file::HistoryLogEntry {
            op_id: "snap-1".into(),
            seq: ops.last().map(|o| o.seq + 2).unwrap_or(2),
            parent_seq: ops.last().map(|o| o.seq + 1).unwrap_or(1),
            timestamp: unibrowser_document_model::history_file::now_rfc3339(),
            plugin_id: "unibrowser-core".into(),
            plugin_version: "0.1.0".into(),
            op_type: "unibrowser.history.snapshot".into(),
            op_schema_version: 1,
            payload: serde_json::json!({ "asset_hash": hash }),
        };
        unibrowser_document_model::history_file::push_op_entry(&hist_path, keep_entry).expect("push");
        fs::write(dir.join(DIR_ASSETS).join(decoy), b"orphan").expect("orphan");
        let removed = gc_content_addressed_assets(&dir, &pkg.manifest, None).expect("gc");
        assert!(removed.contains(&decoy.to_string()));
        assert!(!removed.contains(&hash));
        assert!(dir.join(DIR_ASSETS).join(&hash).is_file());
        let _ = fs::remove_dir_all(&dir);
    }
}
