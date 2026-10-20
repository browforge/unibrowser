//! OPC zip layout for `.unibrow` archives ([ADR 0032]).

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

use crate::{
    ProjectError, ProjectManifest, CONTAINER_FORMAT_VERSION, DIR_ASSETS, DIR_DERIVED, DIR_REFS,
    HISTORY_FILE, MANIFEST_FILE, PROJECT_FILE,
};

const OPC_CONTENT_TYPES: &str = "[Content_Types].xml";
const OPC_ROOT_RELS: &str = "_rels/.rels";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageContainerManifest {
    pub container_format_version: u32,
    pub title: String,
    pub project_format_version: u32,
}

pub fn content_hash_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

pub fn is_content_hash_asset_name(name: &str) -> bool {
    name.len() == 64 && name.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn container_manifest_from_project(project: &ProjectManifest) -> PackageContainerManifest {
    PackageContainerManifest {
        container_format_version: CONTAINER_FORMAT_VERSION,
        title: project.title.clone(),
        project_format_version: project.format_version,
    }
}

pub fn write_container_manifest(root: &Path, project: &ProjectManifest) -> Result<(), ProjectError> {
    let container = container_manifest_from_project(project);
    let data = serde_json::to_string_pretty(&container)?;
    let dest = root.join(MANIFEST_FILE);
    unibrowser_fs::atomic_write(&dest, data.as_bytes()).map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

pub fn ensure_package_history_file(root: &Path) -> Result<(), ProjectError> {
    let path = root.join(HISTORY_FILE);
    if path.is_file() {
        return Ok(());
    }
    let header = unibrowser_document_model::history_file::default_header();
    let marker = unibrowser_document_model::history_file::LOG_MARKER;
    let body = format!("{}\n{}\n", header.trim_end(), marker);
    unibrowser_fs::atomic_write(&path, body.as_bytes()).map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

pub fn compression_for_part(rel_path: &str) -> CompressionMethod {
    let norm = rel_path.replace('\\', "/");
    if norm == OPC_CONTENT_TYPES || norm.ends_with(".rels") {
        return CompressionMethod::Deflated;
    }
    if norm.ends_with(".json")
        || norm.ends_with(".yaml")
        || norm.ends_with(".yml")
        || norm.ends_with(".md")
        || norm.ends_with(".txt")
        || norm.ends_with(HISTORY_FILE)
    {
        return CompressionMethod::Deflated;
    }
    let lower = norm.to_ascii_lowercase();
    const STORED_EXT: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "avif", "mp4", "webm", "mov", "mkv", "mp3",
        "wav", "ogg", "flac", "aac", "gz", "zip", "pdf",
    ];
    if let Some(ext) = lower.rsplit('.').next() {
        if STORED_EXT.iter().any(|e| *e == ext) {
            return CompressionMethod::Stored;
        }
    }
    if lower.starts_with("assets/") && !lower.rsplit('/').next().unwrap_or("").contains('.') {
        return CompressionMethod::Stored;
    }
    CompressionMethod::Deflated
}

fn opc_content_types_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="json" ContentType="application/json"/>
  <Override PartName="/manifest.json" ContentType="application/vnd.unibrowser.package-manifest+json"/>
  <Override PartName="/project.json" ContentType="application/vnd.unibrowser.project+json"/>
  <Override PartName="/.unibrow-history" ContentType="application/vnd.unibrowser.history+yaml"/>
</Types>"#
}

fn opc_root_rels_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.unibrowser.app/package/2026/manifest" Target="manifest.json"/>
  <Relationship Id="rId2" Type="http://schemas.unibrowser.app/package/2026/project" Target="project.json"/>
  <Relationship Id="rId3" Type="http://schemas.unibrowser.app/package/2026/history" Target=".unibrow-history"/>
</Relationships>"#
}

pub fn zip_has_entry(path: &Path, entry: &str) -> Result<bool, ProjectError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let name = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?
            .name()
            .replace('\\', "/");
        if name == entry {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn is_opc_unibrow_zip(path: &Path) -> bool {
    zip_has_entry(path, OPC_CONTENT_TYPES).unwrap_or(false)
        && (zip_has_entry(path, MANIFEST_FILE).unwrap_or(false)
            || zip_has_entry(path, PROJECT_FILE).unwrap_or(false))
}

pub fn is_legacy_unibrow_zip(path: &Path) -> bool {
    !is_opc_unibrow_zip(path) && zip_has_entry(path, PROJECT_FILE).unwrap_or(false)
}

fn walk_package_files(
    root: &Path,
    current: &Path,
    skip_derived: bool,
    out: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), ProjectError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|_| ProjectError::Msg("strip_prefix failed".into()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if skip_derived
            && (rel_str == DIR_DERIVED || rel_str.starts_with(&format!("{}/", DIR_DERIVED)))
        {
            continue;
        }
        if path.is_dir() {
            walk_package_files(root, &path, skip_derived, out)?;
        } else if path.is_file() {
            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            out.insert(rel_str, buf);
        }
    }
    Ok(())
}

fn write_zip_entry(
    zip: &mut ZipWriter<File>,
    name: &str,
    bytes: &[u8],
    method: CompressionMethod,
) -> Result<(), ProjectError> {
    let options: FileOptions<'_, ()> = SimpleFileOptions::default()
        .compression_method(method)
        .last_modified_time(zip::DateTime::default_for_write());
    zip.start_file(name, options)
        .map_err(|e| ProjectError::Msg(e.to_string()))?;
    zip.write_all(bytes).map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

pub fn write_opc_zip_from_root(
    root: &Path,
    dest_zip: &Path,
    skip_derived: bool,
) -> Result<(), ProjectError> {
    if !root.is_dir() || !root.join(PROJECT_FILE).is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    if let Some(parent) = dest_zip.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut parts = BTreeMap::new();
    walk_package_files(root, root, skip_derived, &mut parts)?;

    parts.insert(OPC_CONTENT_TYPES.to_string(), opc_content_types_xml().into());
    parts.insert(OPC_ROOT_RELS.to_string(), opc_root_rels_xml().into());

    if !parts.contains_key(HISTORY_FILE) {
        ensure_package_history_file(root)?;
        let data = fs::read(root.join(HISTORY_FILE))?;
        parts.insert(HISTORY_FILE.to_string(), data);
    }
    if !parts.contains_key(MANIFEST_FILE) {
        let project_data = fs::read(root.join(PROJECT_FILE))?;
        let project: ProjectManifest = serde_json::from_slice(&project_data)?;
        write_container_manifest(root, &project)?;
        parts.insert(MANIFEST_FILE.to_string(), fs::read(root.join(MANIFEST_FILE))?);
    }

    let file = File::create(dest_zip)?;
    let mut zip = ZipWriter::new(file);

    let mut ordered: Vec<String> = parts.keys().cloned().collect();
    ordered.sort();
    for key in ordered {
        let method = compression_for_part(&key);
        write_zip_entry(&mut zip, &key, &parts[&key], method)?;
    }
    zip.finish().map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

pub fn materialize_zip_to_dir(zip_path: &Path, dest: &Path) -> Result<(), ProjectError> {
    materialize_zip_to_dir_cancellable(zip_path, dest, &|| false)
}

pub fn materialize_zip_to_dir_cancellable(
    zip_path: &Path,
    dest: &Path,
    should_cancel: &dyn Fn() -> bool,
) -> Result<(), ProjectError> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        if should_cancel() {
            let _ = fs::remove_dir_all(dest);
            return Err(ProjectError::Msg("unpack cancelled".into()));
        }
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?;
        let name = entry.name().replace('\\', "/");
        if name.ends_with('/') {
            fs::create_dir_all(dest.join(&name))?;
            continue;
        }
        let out_path = dest.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
}

pub fn migrate_assets_to_content_hashes(
    root: &Path,
    manifest: &mut ProjectManifest,
) -> Result<(), ProjectError> {
    migrate_assets_to_content_hashes_with_hook(root, manifest, None)
}

pub fn migrate_assets_to_content_hashes_with_hook(
    root: &Path,
    manifest: &mut ProjectManifest,
    before_part_write: Option<&dyn Fn(&str)>,
) -> Result<(), ProjectError> {
    let assets_dir = root.join(DIR_ASSETS);
    if !assets_dir.is_dir() {
        return Ok(());
    }
    let mut renames: Vec<(String, String)> = Vec::new();
    for node in &mut manifest.nodes {
        let Some(ref name) = node.asset else { continue };
        if is_content_hash_asset_name(name) {
            continue;
        }
        let path = assets_dir.join(name);
        if !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path)?;
        let hash = content_hash_hex(&bytes);
        let dest = assets_dir.join(&hash);
        if !dest.is_file() {
            if let Some(hook) = before_part_write {
                hook(&format!("assets/{}", hash));
            }
            fs::write(&dest, &bytes)?;
        }
        if name != &hash {
            renames.push((name.clone(), hash.clone()));
            node.asset = Some(hash);
        }
    }
    for (old, _) in renames {
        let still_used = manifest
            .nodes
            .iter()
            .any(|n| n.asset.as_deref() == Some(old.as_str()));
        if !still_used {
            let p = assets_dir.join(&old);
            if p.is_file() {
                let _ = fs::remove_file(p);
            }
        }
    }
    Ok(())
}

pub fn ensure_unpacked_package_layout(
    root: &Path,
    manifest: &mut ProjectManifest,
) -> Result<(), ProjectError> {
    migrate_assets_to_content_hashes(root, manifest)?;
    write_container_manifest(root, manifest)?;
    ensure_package_history_file(root)?;
    Ok(())
}

/// Compare uncompressed part bytes between two OPC zips (order-independent).
pub fn opc_part_bytes_equal(a: &Path, b: &Path) -> Result<bool, ProjectError> {
    let map_a = opc_part_map(a)?;
    let map_b = opc_part_map(b)?;
    if map_a.len() != map_b.len() {
        return Ok(false);
    }
    for (k, va) in &map_a {
        match map_b.get(k) {
            Some(vb) if vb == va => {}
            _ => return Ok(false),
        }
    }
    Ok(true)
}

pub fn opc_part_map(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, ProjectError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut out = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ProjectError::Msg(e.to_string()))?;
        let name = entry.name().replace('\\', "/");
        if name.ends_with('/') {
            continue;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        out.insert(name, buf);
    }
    Ok(out)
}

pub fn write_legacy_plain_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    if !root.is_dir() || !root.join(PROJECT_FILE).is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    if let Some(parent) = dest_zip.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(dest_zip)?;
    let mut zip = ZipWriter::new(file);
    let mut parts = BTreeMap::new();
    walk_package_files(root, root, false, &mut parts)?;
    let keys: Vec<String> = parts.keys().cloned().collect();
    for key in keys {
        write_zip_entry(
            &mut zip,
            &key,
            &parts[&key],
            CompressionMethod::Deflated,
        )?;
    }
    zip.finish().map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}
