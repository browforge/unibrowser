//! User-installed native mods under `<config>/mods` (see `docs/plugin-sdk/README.md`).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use unibrowser_mod_api::ModManifest;

use crate::loader::check_api_version;
use crate::ModHostError;

const STATE_FILE: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeModState {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct InstalledNativeMod {
    pub name: String,
    pub title: String,
    pub version: String,
    pub install_dir: PathBuf,
    pub enabled: bool,
    pub library_path: Option<PathBuf>,
    pub signature_ok: bool,
    pub load_block_reason: Option<String>,
}

pub fn list_installed(root: &Path) -> Result<Vec<InstalledNativeMod>, ModHostError> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let info = path.join("info.json");
        if !info.is_file() {
            continue;
        }
        let text = fs::read_to_string(&info)?;
        let manifest: ModManifest = serde_json::from_str(&text)?;
        check_api_version(&manifest)?;
        if let Some(perms) = &manifest.permissions {
            if let Err(e) = unibrowser_mod_api::validate_permissions(&manifest.name, perms) {
                out.push(InstalledNativeMod {
                    name: manifest.name.clone(),
                    title: manifest.title.clone(),
                    version: manifest.version.clone(),
                    install_dir: path,
                    enabled: false,
                    library_path: None,
                    signature_ok: false,
                    load_block_reason: Some(format!("permissions: {}", e)),
                });
                continue;
            }
        }
        let state = read_state(&path);
        let library_path = resolve_library_path(&path, &manifest);
        let (signature_ok, load_block_reason) = signature_status(&path);
        let load_block_reason = if library_path.is_none() {
            Some("native library file not found".into())
        } else {
            load_block_reason
        };
        out.push(InstalledNativeMod {
            name: manifest.name.clone(),
            title: manifest.title.clone(),
            version: manifest.version.clone(),
            install_dir: path,
            enabled: state.enabled,
            library_path,
            signature_ok,
            load_block_reason,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn signature_status(mod_dir: &Path) -> (bool, Option<String>) {
    match crate::signing::verify_mod_dir(mod_dir) {
        Ok(()) => (true, None),
        Err(e) => {
            if crate::signing::require_signed_mods() {
                (false, Some(e.to_string()))
            } else {
                (true, None)
            }
        }
    }
}

fn read_state(mod_dir: &Path) -> NativeModState {
    let path = mod_dir.join(STATE_FILE);
    if !path.is_file() {
        return NativeModState {
            enabled: default_enabled(),
        };
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(NativeModState {
            enabled: default_enabled(),
        })
}

fn write_state(mod_dir: &Path, state: &NativeModState) -> Result<(), ModHostError> {
    fs::write(
        mod_dir.join(STATE_FILE),
        serde_json::to_string_pretty(state)?,
    )?;
    Ok(())
}

pub fn set_enabled(root: &Path, name: &str, enabled: bool) -> Result<(), ModHostError> {
    for rec in list_installed(root)? {
        if rec.name != name {
            continue;
        }
        write_state(
            &rec.install_dir,
            &NativeModState { enabled },
        )?;
        return Ok(());
    }
    Err(ModHostError::Dynamic(format!("native mod not installed: {}", name)))
}

pub fn resolve_library_path(mod_dir: &Path, manifest: &ModManifest) -> Option<PathBuf> {
    if let Some(lib) = &manifest.library {
        let p = mod_dir.join(lib);
        if p.is_file() {
            return Some(p);
        }
    }
    let stem = manifest.name.replace('-', "_");
    let candidates: Vec<String> = {
        #[cfg(target_os = "windows")]
        {
            vec![format!("{}.dll", stem), format!("{}_mod.dll", stem)]
        }
        #[cfg(target_os = "macos")]
        {
            vec![
                format!("lib{}.dylib", stem),
                format!("lib{}_mod.dylib", stem),
            ]
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            vec![
                format!("lib{}.so", stem),
                format!("lib{}_mod.so", stem),
            ]
        }
    };
    for c in candidates {
        let p = mod_dir.join(&c);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Enabled mods with a resolvable library and passing signature policy.
pub fn enabled_load_candidates(root: &Path) -> Result<Vec<(PathBuf, PathBuf)>, ModHostError> {
    let mut out = Vec::new();
    for rec in list_installed(root)? {
        if !rec.enabled {
            continue;
        }
        crate::signing::verify_mod_dir(&rec.install_dir)?;
        if let Some(lib) = rec.library_path.filter(|p| p.is_file()) {
            out.push((rec.install_dir.clone(), lib));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_mod_with_state() {
        let root = std::env::temp_dir().join(format!(
            "ub-native-mod-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let mod_dir = root.join("demo-mod");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(
            mod_dir.join("info.json"),
            r#"{"name":"demo-mod","version":"0.0.1","title":"Demo","unibrowser_api":"0.1.0"}"#,
        )
        .unwrap();
        fs::write(mod_dir.join("state.json"), r#"{"enabled":false}"#).unwrap();
        let list = list_installed(&root).expect("list");
        assert_eq!(list.len(), 1);
        assert!(!list[0].enabled);
        let _ = fs::remove_dir_all(&root);
    }
}
