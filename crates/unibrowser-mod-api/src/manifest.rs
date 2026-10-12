use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::capability_surface::CapabilityProvideEntry;
use crate::contributes::ModContributes;

/// Factorio-style mod manifest (`info.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModManifest {
    pub name: String,
    pub version: String,
    pub title: String,
    #[serde(default)]
    pub author: String,
    /// Semver of `unibrowser-mod-api` this mod targets.
    pub unibrowser_api: String,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub optional_dependencies: BTreeMap<String, String>,
    /// Mods that must not be present (`! old-mod` in Factorio terms).
    #[serde(default)]
    pub incompatibilities: BTreeMap<String, String>,
    /// Named API surfaces this mod implements (`audio.levels` → `1.0.0`).
    #[serde(default)]
    pub provides: BTreeMap<String, String>,
    /// Required APIs from other mods (`audio.levels` → `^1.0`).
    #[serde(default)]
    pub api_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub description: String,
    /// Optional native library file name (platform-specific).
    #[serde(default)]
    pub library: Option<String>,
    /// Optional WASM module file name (T2 sandbox).
    #[serde(default)]
    pub wasm_module: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Surface providers this mod implements (ADR 0030). Distinct from API `provides` map.
    #[serde(default, rename = "capability_provides")]
    pub capability_provides: Vec<CapabilityProvideEntry>,
    #[serde(default)]
    pub contributes: Option<ModContributes>,
}
