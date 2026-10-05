//! Minimal `.unibrow` directory bundle ([ADR 0015](../../docs/adr/0015-native-project-format.md)).

mod zip_bundle;

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROJECT_FILE: &str = "project.json";
pub const DIR_ASSETS: &str = "assets";
pub const DIR_REFS: &str = "refs";
pub const DIR_DERIVED: &str = "derived";

pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a .unibrow package (missing {0})")]
    NotPackage(String),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub title: String,
    pub nodes: Vec<ProjectNode>,
}

impl ProjectManifest {
    pub fn empty(title: impl Into<String>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            title: title.into(),
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectPackage {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
}

pub fn is_unibrow_path(path: &Path) -> bool {
    if path.is_dir() && path.join(PROJECT_FILE).is_file() {
        return true;
    }
    zip_bundle::is_unibrow_zip_file(path)
}

pub fn export_package_to_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    zip_bundle::export_directory_to_zip(root, dest_zip)
}

pub fn materialize_zip_package(zip_path: &Path, cache_dir: &Path) -> Result<PathBuf, ProjectError> {
    zip_bundle::materialize_zip_package(zip_path, cache_dir)
}

/// Directory containing `project.json` — materialises zip packages when needed.
pub fn resolve_package_root(path: &Path, zip_cache_dir: &Path) -> Result<PathBuf, ProjectError> {
    if path.is_dir() && path.join(PROJECT_FILE).is_file() {
        return Ok(path.to_path_buf());
    }
    if zip_bundle::is_unibrow_zip_file(path) {
        return zip_bundle::materialize_zip_package(path, zip_cache_dir);
    }
    Err(ProjectError::NotPackage(PROJECT_FILE.into()))
}

pub fn create_package(root: &Path, title: &str) -> Result<ProjectPackage, ProjectError> {
    if root.exists() {
        return Err(ProjectError::Msg(format!("already exists: {}", root.display())));
    }
    fs::create_dir_all(root.join(DIR_ASSETS))?;
    fs::create_dir_all(root.join(DIR_REFS))?;
    fs::create_dir_all(root.join(DIR_DERIVED))?;
    let manifest = ProjectManifest::empty(title);
    write_manifest(root, &manifest)?;
    Ok(ProjectPackage {
        root: root.to_path_buf(),
        manifest,
    })
}

pub fn open_package(root: &Path) -> Result<ProjectPackage, ProjectError> {
    let manifest_path = root.join(PROJECT_FILE);
    if !manifest_path.is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    let data = fs::read_to_string(&manifest_path)?;
    let manifest: ProjectManifest = serde_json::from_str(&data)?;
    if manifest.format_version != FORMAT_VERSION {
        return Err(ProjectError::Msg(format!(
            "unsupported format_version {}",
            manifest.format_version
        )));
    }
    Ok(ProjectPackage {
        root: root.to_path_buf(),
        manifest,
    })
}

pub fn write_manifest(root: &Path, manifest: &ProjectManifest) -> Result<(), ProjectError> {
    let data = serde_json::to_string_pretty(manifest)?;
    fs::write(root.join(PROJECT_FILE), data)?;
    Ok(())
}

pub fn guess_asset_node_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "image",
        "pdf" => "pdf",
        "docx" => "document",
        "dxf" => "drawing",
        "mp4" | "webm" => "video",
        "wav" | "mid" | "midi" => "audio",
        "js" | "rhai" => "script",
        _ => "file",
    }
}

pub fn sanitize_label_for_filename(label: &str) -> String {
    let mut s: String = label
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() || c == '.' {
                '-'
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "asset".into()
    } else {
        s
    }
}

pub fn unique_asset_file_name(package: &ProjectPackage, original: &str) -> String {
    unique_asset_file_name_excluding(package, original, None)
}

/// Pick a non-colliding name under `assets/`; `exclude` is the current file (may be reused).
pub fn unique_asset_file_name_excluding(
    package: &ProjectPackage,
    original: &str,
    exclude: Option<&str>,
) -> String {
    let assets_dir = package.root.join(DIR_ASSETS);
    let path = Path::new(original);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "asset".into());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string());
    let mut candidate = original.to_string();
    let mut n = 2u32;
    while {
        let dest = assets_dir.join(&candidate);
        dest.exists() && exclude.map(|e| e != candidate).unwrap_or(true)
    } {
        candidate = match &ext {
            Some(ext) if !ext.is_empty() => format!("{}-{}.{}", stem, n, ext),
            _ => format!("{}-{}", stem, n),
        };
        n += 1;
    }
    candidate
}

pub fn add_asset_copy(
    package: &mut ProjectPackage,
    source: &Path,
    node_type: &str,
    label: &str,
) -> Result<ProjectNode, ProjectError> {
    if !source.is_file() {
        return Err(ProjectError::Msg("source is not a file".into()));
    }
    let id = format!("node-{}", package.manifest.nodes.len() + 1);
    let original_name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "asset".into());
    let file_name = unique_asset_file_name(package, &original_name);
    let dest = package.root.join(DIR_ASSETS).join(&file_name);
    fs::copy(source, &dest)?;
    let node = ProjectNode {
        id,
        node_type: node_type.into(),
        label: label.into(),
        asset: Some(file_name),
    };
    package.manifest.nodes.push(node.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(node)
}

pub fn remove_node(package: &mut ProjectPackage, node_id: &str) -> Result<(), ProjectError> {
    let idx = package
        .manifest
        .nodes
        .iter()
        .position(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    let node = package.manifest.nodes.remove(idx);
    if let Some(asset) = node.asset {
        let path = package.root.join(DIR_ASSETS).join(asset);
        if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn rename_node_label(
    package: &mut ProjectPackage,
    node_id: &str,
    label: &str,
) -> Result<(), ProjectError> {
    let label = label.trim();
    if label.is_empty() {
        return Err(ProjectError::Msg("label cannot be empty".into()));
    }
    let idx = package
        .manifest
        .nodes
        .iter()
        .position(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    let old_name = package.manifest.nodes[idx].asset.clone();
    let mut new_asset = old_name.clone();
    if let Some(ref old_name) = old_name {
        let assets_dir = package.root.join(DIR_ASSETS);
        let old_path = assets_dir.join(old_name);
        if old_path.is_file() {
            let ext = Path::new(old_name)
                .extension()
                .map(|e| e.to_string_lossy().to_string());
            let base = sanitize_label_for_filename(label);
            let desired = match &ext {
                Some(ext) if !ext.is_empty() => format!("{}.{}", base, ext),
                _ => base.clone(),
            };
            let new_name =
                unique_asset_file_name_excluding(package, &desired, Some(old_name));
            if new_name != *old_name {
                let new_path = assets_dir.join(&new_name);
                fs::rename(&old_path, &new_path)?;
            }
            new_asset = Some(new_name);
        }
    }
    let node = &mut package.manifest.nodes[idx];
    node.asset = new_asset;
    node.label = label.to_string();
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_asset_name_when_taken() {
        let dir = std::env::temp_dir().join("unibrow-test-dup");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Dup").expect("create");
        let incoming = dir.join("incoming");
        fs::create_dir_all(&incoming).unwrap();
        let first = incoming.join("photo.png");
        fs::write(&first, b"1").unwrap();
        add_asset_copy(&mut pkg, &first, "image", "first").expect("first");
        let second = incoming.join("photo.png");
        fs::write(&second, b"2").unwrap();
        let node = add_asset_copy(&mut pkg, &second, "image", "second").expect("dup");
        assert_eq!(node.asset.as_deref(), Some("photo-2.png"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_node_drops_asset() {
        let dir = std::env::temp_dir().join("unibrow-test-remove");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Rm").expect("create");
        let src = dir.join("x.txt");
        fs::write(&src, b"x").unwrap();
        let node = add_asset_copy(&mut pkg, &src, "file", "x").expect("add");
        let id = node.id.clone();
        remove_node(&mut pkg, &id).expect("remove");
        assert!(pkg.manifest.nodes.is_empty());
        assert!(!dir.join(DIR_ASSETS).join("x.txt").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_node_renames_asset_file() {
        let dir = std::env::temp_dir().join("unibrow-test-rename");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Ren").expect("create");
        let src = dir.join("incoming.txt");
        fs::write(&src, b"hi").unwrap();
        let node = add_asset_copy(&mut pkg, &src, "file", "Old Label").expect("add");
        let id = node.id.clone();
        rename_node_label(&mut pkg, &id, "My Asset").expect("rename");
        assert_eq!(pkg.manifest.nodes[0].label, "My Asset");
        assert_eq!(pkg.manifest.nodes[0].asset.as_deref(), Some("my-asset.txt"));
        assert!(dir.join(DIR_ASSETS).join("my-asset.txt").is_file());
        assert!(!dir.join(DIR_ASSETS).join("incoming.txt").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_and_open_roundtrip() {
        let dir = std::env::temp_dir().join("unibrow-test-create");
        let _ = fs::remove_dir_all(&dir);
        let _pkg = create_package(&dir, "Demo").expect("create");
        assert!(dir.join(DIR_ASSETS).is_dir());
        let again = open_package(&dir).expect("open");
        assert_eq!(again.manifest.title, "Demo");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn zip_export_and_materialize() {
        let dir = std::env::temp_dir().join("unibrow-test-zip");
        let _ = fs::remove_dir_all(&dir);
        let _pkg = create_package(&dir, "ZipMe").expect("create");
        let zip_path = dir.with_extension("unibrow");
        export_package_to_zip(&dir, &zip_path).expect("export");
        assert!(zip_path.is_file());
        assert!(is_unibrow_path(&zip_path));
        let cache = dir.join("cache");
        let materialized = materialize_zip_package(&zip_path, &cache).expect("materialize");
        let opened = open_package(&materialized).expect("open");
        assert_eq!(opened.manifest.title, "ZipMe");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&zip_path);
    }
}
