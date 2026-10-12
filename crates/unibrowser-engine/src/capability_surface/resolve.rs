use std::collections::HashMap;
use std::path::Path;

use super::registry::SurfaceProviderRegistry;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityPrefs {
    pub default_by_capability: HashMap<String, String>,
    pub open_with_by_extension: HashMap<String, String>,
    pub open_with_by_path: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSurfaceProvider {
    pub capability_id: String,
    pub mod_name: Option<String>,
    pub reason: String,
    pub used_fallback: bool,
}

impl ResolvedSurfaceProvider {
    pub fn missing(capability_id: &str) -> Self {
        Self {
            capability_id: capability_id.to_string(),
            mod_name: None,
            reason: "No installed provider".into(),
            used_fallback: true,
        }
    }
}

pub fn resolve_surface_provider(
    registry: &SurfaceProviderRegistry,
    capability_id: &str,
    prefs: &CapabilityPrefs,
    path: Option<&Path>,
    ext: Option<&str>,
) -> ResolvedSurfaceProvider {
    let candidates: Vec<_> = registry
        .providers
        .iter()
        .filter(|p| p.capability_id == capability_id)
        .collect();
    if candidates.is_empty() {
        return ResolvedSurfaceProvider::missing(capability_id);
    }

    if let Some(path) = path {
        let key = path.to_string_lossy().to_string();
        if let Some(mod_name) = prefs.open_with_by_path.get(&key) {
            if provider_exists(&candidates, mod_name) {
                return ResolvedSurfaceProvider {
                    capability_id: capability_id.to_string(),
                    mod_name: Some(mod_name.clone()),
                    reason: "Open-with choice for this path".into(),
                    used_fallback: false,
                };
            }
        }
    }

    if let Some(ext) = ext {
        let ext = ext.to_ascii_lowercase();
        if let Some(mod_name) = prefs.open_with_by_extension.get(&ext) {
            if provider_exists(&candidates, mod_name) {
                return ResolvedSurfaceProvider {
                    capability_id: capability_id.to_string(),
                    mod_name: Some(mod_name.clone()),
                    reason: format!("Open-with default for .{}", ext),
                    used_fallback: false,
                };
            }
        }
    }

    if let Some(mod_name) = prefs.default_by_capability.get(capability_id) {
        if provider_exists(&candidates, mod_name) {
            return ResolvedSurfaceProvider {
                capability_id: capability_id.to_string(),
                mod_name: Some(mod_name.clone()),
                reason: "User default provider".into(),
                used_fallback: false,
            };
        }
    }

    let conflict = registry
        .default_conflicts
        .iter()
        .find(|c| c.capability_id == capability_id);

    let defaults: Vec<_> = candidates.iter().filter(|p| p.default).collect();
    if defaults.len() == 1 && conflict.is_none() {
        let p = defaults[0];
        return ResolvedSurfaceProvider {
            capability_id: capability_id.to_string(),
            mod_name: Some(p.mod_name.clone()),
            reason: "Bundled default provider".into(),
            used_fallback: false,
        };
    }

    if candidates.len() == 1 {
        let p = candidates[0];
        return ResolvedSurfaceProvider {
            capability_id: capability_id.to_string(),
            mod_name: Some(p.mod_name.clone()),
            reason: "Only installed provider".into(),
            used_fallback: false,
        };
    }

    if let Some(p) = candidates.first() {
        return ResolvedSurfaceProvider {
            capability_id: capability_id.to_string(),
            mod_name: Some(p.mod_name.clone()),
            reason: if conflict.is_some() {
                "Multiple defaults; using highest-priority provider until you choose"
            } else {
                "Highest-priority provider"
            }
            .into(),
            used_fallback: false,
        };
    }

    ResolvedSurfaceProvider::missing(capability_id)
}

fn provider_exists(candidates: &[&super::registry::SurfaceProviderRecord], mod_name: &str) -> bool {
    candidates.iter().any(|p| p.mod_name == mod_name)
}

pub fn providers_for_capability<'a>(
    registry: &'a SurfaceProviderRegistry,
    capability_id: &'a str,
) -> Vec<&'a super::registry::SurfaceProviderRecord> {
    registry
        .providers
        .iter()
        .filter(|p| p.capability_id == capability_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_surface::registry::build_surface_provider_registry;
    use unibrowser_mod_api::{CapabilityProvideEntry, ModManifest};

    fn sample_mod(name: &str, provides: Vec<CapabilityProvideEntry>) -> ModManifest {
        ModManifest {
            name: name.into(),
            version: "0.1.0".into(),
            title: name.into(),
            author: String::new(),
            unibrowser_api: "0.1.0".into(),
            dependencies: Default::default(),
            optional_dependencies: Default::default(),
            incompatibilities: Default::default(),
            provides: Default::default(),
            api_dependencies: Default::default(),
            description: String::new(),
            library: None,
            wasm_module: None,
            capabilities: Vec::new(),
            capability_provides: provides,
            contributes: None,
        }
    }

    #[test]
    fn user_default_overrides_bundled_default() {
        use unibrowser_mod_api::capability_surface::ids;
        let reg = build_surface_provider_registry(&[
            sample_mod(
                "base-text",
                vec![CapabilityProvideEntry {
                    capability: ids::OPEN_TEXT.into(),
                    default: true,
                    priority: 100,
                    content_kinds: vec![],
                }],
            ),
            sample_mod(
                "toy-markdown",
                vec![CapabilityProvideEntry {
                    capability: ids::OPEN_TEXT.into(),
                    default: false,
                    priority: 50,
                    content_kinds: vec![],
                }],
            ),
        ]);
        let prefs = CapabilityPrefs {
            default_by_capability: HashMap::from([(ids::OPEN_TEXT.into(), "toy-markdown".into())]),
            open_with_by_extension: HashMap::new(),
            open_with_by_path: HashMap::new(),
        };
        let resolved = resolve_surface_provider(&reg, ids::OPEN_TEXT, &prefs, None, Some("md"));
        assert_eq!(resolved.mod_name.as_deref(), Some("toy-markdown"));
    }

    #[test]
    fn missing_provider_is_graceful() {
        use unibrowser_mod_api::capability_surface::ids;
        let reg = SurfaceProviderRegistry::default();
        let resolved = resolve_surface_provider(&reg, ids::RENDER_PDF, &CapabilityPrefs::default(), None, None);
        assert!(resolved.mod_name.is_none());
        assert!(resolved.used_fallback);
    }

    #[test]
    fn conflicting_defaults_still_resolve() {
        use unibrowser_mod_api::capability_surface::ids;
        let reg = build_surface_provider_registry(&[
            sample_mod(
                "mod-a",
                vec![CapabilityProvideEntry {
                    capability: ids::OPEN_TEXT.into(),
                    default: true,
                    priority: 10,
                    content_kinds: vec![],
                }],
            ),
            sample_mod(
                "mod-b",
                vec![CapabilityProvideEntry {
                    capability: ids::OPEN_TEXT.into(),
                    default: true,
                    priority: 20,
                    content_kinds: vec![],
                }],
            ),
        ]);
        assert_eq!(reg.default_conflicts.len(), 1);
        let resolved = resolve_surface_provider(
            &reg,
            ids::OPEN_TEXT,
            &CapabilityPrefs::default(),
            None,
            None,
        );
        assert_eq!(resolved.mod_name.as_deref(), Some("mod-b"));
    }
}
