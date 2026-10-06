use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use semver::Version;
use semver::VersionReq;
use thiserror::Error;
use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{MOD_API_VERSION, ModManifest};

use crate::deps::{DepError, resolve_mod_graph};

/// T0 dev: static registration from in-tree `base-*` sources.
/// Product/community mods: manifest + compiled `cdylib`/WASM entry (see docs/adr/0002).
pub type ModRegisterFn = fn(&mut CommandRegistry);

#[derive(Debug, Error)]
pub enum ModHostError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("dependency error: {0}")]
    Deps(#[from] DepError),
    #[error("mod `{0}` API version `{1}` incompatible with host `{2}`")]
    ApiIncompatible(String, String, String),
    #[error("unknown bundled mod `{0}`")]
    UnknownMod(String),
    #[error("no info.json in {0}")]
    MissingManifest(PathBuf),
    #[error("dynamic mod: {0}")]
    Dynamic(String),
}

#[derive(Debug, Clone)]
pub struct LoadedMod {
    pub manifest: ModManifest,
    pub path: PathBuf,
}

pub struct ModHost {
    pub mods: Vec<LoadedMod>,
    pub load_order: Vec<String>,
    /// Non-blocking resolver notes (optional deps, degraded APIs).
    pub resolve_notes: Vec<String>,
}

impl ModHost {
    pub fn discover_and_load(
        mod_roots: &[PathBuf],
        registry: &mut CommandRegistry,
        bundled: &HashMap<&str, ModRegisterFn>,
    ) -> Result<Self, ModHostError> {
        let mut manifests = Vec::new();
        let mut paths = HashMap::new();
        let mut contribution_notes = Vec::new();

        for root in mod_roots {
            if !root.is_dir() {
                continue;
            }
            for entry in fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();
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
                if let Some(contrib) = &manifest.contributes {
                    let errs = unibrowser_mod_api::validate_contributes(&manifest.name, contrib);
                    for err in errs {
                        contribution_notes.push(format!(
                            "mod {} contribution: {}",
                            err.mod_name, err.message
                        ));
                    }
                }
                if paths.contains_key(&manifest.name) {
                    continue;
                }
                manifests.push(manifest.clone());
                paths.insert(manifest.name.clone(), path);
            }
        }

        let report = resolve_mod_graph(&manifests)?;
        let load_order = report.load_order;
        let mut resolve_notes = report.notes;
        resolve_notes.extend(contribution_notes);
        let mods: Vec<LoadedMod> = load_order
            .iter()
            .filter_map(|name| {
                paths.get(name).map(|p| LoadedMod {
                    manifest: manifests.iter().find(|m| &m.name == name).cloned().unwrap(),
                    path: p.clone(),
                })
            })
            .collect();

        for name in &load_order {
            if let Some(register) = bundled.get(name.as_str()) {
                register(registry);
            } else {
                return Err(ModHostError::UnknownMod(name.clone()));
            }
        }

        Ok(Self {
            mods,
            load_order,
            resolve_notes,
        })
    }
}

pub(crate) fn check_api_version(manifest: &ModManifest) -> Result<(), ModHostError> {
    let host = Version::parse(MOD_API_VERSION).expect("host api semver");
    let mod_api = Version::parse(&manifest.unibrowser_api).map_err(|_| {
        ModHostError::ApiIncompatible(
            manifest.name.clone(),
            manifest.unibrowser_api.clone(),
            MOD_API_VERSION.to_string(),
        )
    })?;
    if mod_api.major != host.major || mod_api.minor > host.minor {
        return Err(ModHostError::ApiIncompatible(
            manifest.name.clone(),
            manifest.unibrowser_api.clone(),
            MOD_API_VERSION.to_string(),
        ));
    }
    let req = VersionReq::parse(&format!("={}", MOD_API_VERSION)).expect("req");
    if mod_api.major == host.major && !req.matches(&mod_api) && mod_api > host {
        return Err(ModHostError::ApiIncompatible(
            manifest.name.clone(),
            manifest.unibrowser_api.clone(),
            MOD_API_VERSION.to_string(),
        ));
    }
    Ok(())
}

pub fn default_mods_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../mods")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn loads_bundled_mods_from_manifests() {
        let mut registry = CommandRegistry::with_builtins();
        let bundled: HashMap<&str, ModRegisterFn> = [
            ("base-files", base_files::register as ModRegisterFn),
            ("base-web", base_web::register as ModRegisterFn),
            ("base-text", base_text::register as ModRegisterFn),
            ("base-png", base_png::register as ModRegisterFn),
            ("base-svg", base_svg::register as ModRegisterFn),
            ("base-docx", base_docx::register as ModRegisterFn),
            ("base-pdf", base_pdf::register as ModRegisterFn),
            ("base-dxf", base_dxf::register as ModRegisterFn),
            ("base-js", base_js::register as ModRegisterFn),
            ("base-video", base_video::register as ModRegisterFn),
            ("base-audio", base_audio::register as ModRegisterFn),
            ("toy-markdown", toy_markdown::register as ModRegisterFn),
            ("base-hex", base_hex::register as ModRegisterFn),
            ("base-epub", base_epub::register as ModRegisterFn),
            ("base-csv", base_csv::register as ModRegisterFn),
            ("spell-en-nz", spell_en_nz::register as ModRegisterFn),
            ("grammar-harper", grammar_harper::register as ModRegisterFn),
            ("dxf-checker", dxf_checker::register as ModRegisterFn),
            ("unibrow-checker", unibrow_checker::register as ModRegisterFn),
            ("live-proving", live_proving::register as ModRegisterFn),
            ("ai-assistant", ai_assistant::register as ModRegisterFn),
        ]
        .into_iter()
        .collect();
        let host = ModHost::discover_and_load(&[default_mods_dir()], &mut registry, &bundled)
            .expect("load");
        assert_eq!(host.load_order.len(), 21);
        assert!(registry.command_meta("unibrowser.font.size.increase").is_some());
    }
}
