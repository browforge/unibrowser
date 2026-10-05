//! Minimal `.unibrow` directory bundle ([ADR 0015](../../docs/adr/0015-native-project-format.md)).

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
    path.is_dir() && path.join(PROJECT_FILE).is_file()
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

pub fn unique_asset_file_name(package: &ProjectPackage, original: &str) -> String {
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
    while assets_dir.join(&candidate).exists() {
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
    let node = package
        .manifest
        .nodes
        .iter_mut()
        .find(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
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
    fn create_and_open_roundtrip() {
        let dir = std::env::temp_dir().join("unibrow-test-create");
        let _ = fs::remove_dir_all(&dir);
        let _pkg = create_package(&dir, "Demo").expect("create");
        assert!(dir.join(DIR_ASSETS).is_dir());
        let again = open_package(&dir).expect("open");
        assert_eq!(again.manifest.title, "Demo");
        let _ = fs::remove_dir_all(&dir);
    }
}
