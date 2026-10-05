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
    let file_name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "asset".into());
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

#[cfg(test)]
mod tests {
    use super::*;

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
