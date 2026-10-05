//! Undo snapshots for `.unibrow` `project.json` (timeline edits).

use std::fs;
use std::path::Path;

use crate::undo_stack::UndoStacks;

pub fn read_manifest_json(package_root: &Path) -> Result<String, String> {
    let path = package_root.join(unibrowser_project::PROJECT_FILE);
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn write_manifest_json(package_root: &Path, json: &str) -> Result<(), String> {
    let manifest: unibrowser_project::ProjectManifest =
        serde_json::from_str(json).map_err(|e| e.to_string())?;
    unibrowser_project::write_manifest(package_root, &manifest).map_err(|e| e.to_string())
}

pub fn record_manifest_undo(
    stacks: &mut UndoStacks,
    project_path_key: &str,
    before_json: String,
    package_root: &Path,
) -> Result<(), String> {
    let after = read_manifest_json(package_root)?;
    stacks.record_change(project_path_key, before_json, &after);
    Ok(())
}
