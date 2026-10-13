//! `.ubtemplate` packs and built-in project starters ([ADR 0036]).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    ProjectError, ProjectManifest, ProjectPackage, PROJECT_FILE, DIR_ASSETS, write_manifest,
};

pub const TEMPLATE_MANIFEST: &str = "template.json";
pub const MEMORY_ROOT_PREFIX: &str = "unibrowser-memory://session/";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TemplateRelationships {
    #[serde(default)]
    pub requires: HashMap<String, String>,
    #[serde(default)]
    pub recommends: HashMap<String, String>,
    #[serde(default)]
    pub optional: HashMap<String, String>,
    #[serde(default)]
    pub incompatible: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateHubMeta {
    pub publisher: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateManifest {
    pub template_schema_version: u32,
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub relationships: TemplateRelationships,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_app_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hub: Option<TemplateHubMeta>,
    #[serde(default)]
    pub source: TemplateSourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemplateSourceKind {
    #[default]
    Builtin,
    User,
    Installed,
}

#[derive(Debug, Clone)]
pub struct TemplatePack {
    pub manifest: TemplateManifest,
    pub project: ProjectManifest,
    pub root: PathBuf,
    pub assets: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTemplateIndexEntry {
    pub id: String,
    pub version: String,
    pub title: String,
    pub path: String,
    pub added_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserTemplateIndex {
    pub entries: Vec<UserTemplateIndexEntry>,
}

pub fn is_memory_project_root(path: &Path) -> bool {
    path.to_string_lossy().starts_with(MEMORY_ROOT_PREFIX)
}

pub fn memory_session_id(path: &Path) -> Option<String> {
    let s = path.to_string_lossy();
    if !s.starts_with(MEMORY_ROOT_PREFIX) {
        return None;
    }
    let id = s.trim_start_matches(MEMORY_ROOT_PREFIX);
    if id.is_empty() || id.contains('/') {
        return None;
    }
    Some(id.to_string())
}

pub fn memory_root_for_session(session_id: &str) -> PathBuf {
    PathBuf::from(format!("{}{}", MEMORY_ROOT_PREFIX, session_id))
}

pub fn create_memory_package(title: impl Into<String>, session_id: &str) -> ProjectPackage {
    let title = title.into();
    ProjectPackage {
        root: memory_root_for_session(session_id),
        manifest: ProjectManifest::empty(title),
    }
}

pub fn load_template_pack_dir(dir: &Path, source: TemplateSourceKind) -> Result<TemplatePack, ProjectError> {
    let manifest_path = dir.join(TEMPLATE_MANIFEST);
    let project_path = dir.join(PROJECT_FILE);
    if !manifest_path.is_file() {
        return Err(ProjectError::Msg(format!("missing {}", TEMPLATE_MANIFEST)));
    }
    if !project_path.is_file() {
        return Err(ProjectError::Msg(format!("missing {}", PROJECT_FILE)));
    }
    let manifest_data = fs::read_to_string(&manifest_path)?;
    let mut manifest: TemplateManifest = serde_json::from_str(&manifest_data)?;
    manifest.source = source;
    manifest.pack_path = Some(dir.to_string_lossy().into_owned());
    if manifest.template_schema_version != 1 {
        return Err(ProjectError::Msg(format!(
            "unsupported template_schema_version {}",
            manifest.template_schema_version
        )));
    }
    let project_data = fs::read_to_string(&project_path)?;
    let project: ProjectManifest = serde_json::from_str(&project_data)?;
    crate::validate_manifest(&project)?;
    let assets = read_assets_dir(&dir.join(DIR_ASSETS))?;
    Ok(TemplatePack {
        manifest,
        project,
        root: dir.to_path_buf(),
        assets,
    })
}

fn read_assets_dir(assets_dir: &Path) -> Result<HashMap<String, Vec<u8>>, ProjectError> {
    let mut out = HashMap::new();
    if !assets_dir.is_dir() {
        return Ok(out);
    }
    for entry in fs::read_dir(assets_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let bytes = fs::read(entry.path())?;
        out.insert(name, bytes);
    }
    Ok(out)
}

pub fn apply_template_project(
    package: &mut ProjectPackage,
    pack: &TemplatePack,
    template_pin: Option<&str>,
) -> Result<(), ProjectError> {
    package.manifest = pack.project.clone();
    if let Some(pin) = template_pin {
        package.manifest.description = Some(format!("applied_template:{}", pin));
    }
    if !is_memory_project_root(&package.root) {
        let assets_dir = package.root.join(DIR_ASSETS);
        fs::create_dir_all(&assets_dir)?;
        for (name, bytes) in &pack.assets {
            fs::write(assets_dir.join(name), bytes)?;
        }
        write_manifest(&package.root, &package.manifest)?;
    }
    Ok(())
}

pub fn apply_project_json_to_package(
    package: &mut ProjectPackage,
    project: &ProjectManifest,
) -> Result<(), ProjectError> {
    crate::validate_manifest(project)?;
    package.manifest = project.clone();
    if !is_memory_project_root(&package.root) {
        write_manifest(&package.root, &package.manifest)?;
    }
    Ok(())
}

pub fn export_ubtemplate_pack(
    dest: &Path,
    manifest: &TemplateManifest,
    project: &ProjectManifest,
    asset_files: &[(String, PathBuf)],
) -> Result<(), ProjectError> {
    if dest.exists() {
        return Err(ProjectError::Msg(format!("already exists: {}", dest.display())));
    }
    fs::create_dir_all(dest.join(DIR_ASSETS))?;
    let manifest_path = dest.join(TEMPLATE_MANIFEST);
    let project_path = dest.join(PROJECT_FILE);
    let manifest_json = serde_json::to_string_pretty(manifest)?;
    let project_json = serde_json::to_string_pretty(project)?;
    fs::write(&manifest_path, manifest_json)?;
    fs::write(&project_path, project_json)?;
    for (rel, src) in asset_files {
        if src.is_file() {
            fs::copy(src, dest.join(DIR_ASSETS).join(rel))?;
        }
    }
    Ok(())
}

pub fn user_template_index_path(config_dir: &Path) -> PathBuf {
    config_dir.join("templates/user/index.json")
}

pub fn load_user_template_index(config_dir: &Path) -> UserTemplateIndex {
    let path = user_template_index_path(config_dir);
    if !path.is_file() {
        return UserTemplateIndex::default();
    }
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => UserTemplateIndex::default(),
    }
}

pub fn save_user_template_index(config_dir: &Path, index: &UserTemplateIndex) -> Result<(), ProjectError> {
    let path = user_template_index_path(config_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(index)?;
    fs::write(&path, data)?;
    Ok(())
}

pub fn list_builtin_template_dirs(builtin_root: &Path) -> Vec<PathBuf> {
    if !builtin_root.is_dir() {
        return Vec::new();
    }
    let mut dirs: Vec<PathBuf> = match fs::read_dir(builtin_root) {
        Ok(read) => read
            .filter_map(|e| {
                let p = e.ok()?.path();
                if p.is_dir() && p.join(TEMPLATE_MANIFEST).is_file() {
                    Some(p)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_package;

    #[test]
    fn memory_root_round_trip() {
        let root = memory_root_for_session("abc-123");
        assert!(is_memory_project_root(&root));
        assert_eq!(memory_session_id(&root).as_deref(), Some("abc-123"));
    }

    #[test]
    fn apply_template_in_memory_skips_disk() {
        let mut pkg = create_memory_package("T", "sess1");
        let project = ProjectManifest::empty("From template");
        let pack = TemplatePack {
            manifest: TemplateManifest {
                template_schema_version: 1,
                id: "test".into(),
                version: "1.0.0".into(),
                title: "Test".into(),
                description: "d".into(),
                categories: vec![],
                tags: vec![],
                relationships: TemplateRelationships::default(),
                default_workspace: None,
                min_app_version: None,
                hub: None,
                source: TemplateSourceKind::Builtin,
                pack_path: None,
            },
            project,
            root: PathBuf::from("/tmp/unused"),
            assets: HashMap::new(),
        };
        apply_template_project(&mut pkg, &pack, Some("test@1.0.0")).expect("apply");
        assert_eq!(pkg.manifest.title, "From template");
        assert!(!pkg.root.join(PROJECT_FILE).exists());
    }
}
