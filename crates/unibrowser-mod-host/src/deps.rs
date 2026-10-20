use std::collections::{BTreeMap, HashMap, VecDeque};

use semver::VersionReq;
use thiserror::Error;
use unibrowser_mod_api::ModManifest;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DepError {
    #[error("missing dependency `{0}` required by `{1}`")]
    Missing(String, String),
    #[error("cyclic mod dependencies involving `{0}`")]
    Cycle(String),
    #[error("dependency `{dep}` version mismatch for mod `{mod_name}`: required {required}")]
    VersionMismatch {
        mod_name: String,
        dep: String,
        required: String,
    },
    #[error("mod `{0}` incompatible with installed `{1}`")]
    Incompatible(String, String),
    #[error("API `{api}` required by `{mod_name}` is not provided")]
    ApiMissing { mod_name: String, api: String },
    #[error("API `{api}` version conflict: {detail}")]
    ApiConflict { api: String, detail: String },
}

#[derive(Debug, Clone, Default)]
pub struct ModResolveReport {
    pub load_order: Vec<String>,
    /// Non-blocking resolver notes (optional deps, degraded APIs).
    pub notes: Vec<String>,
    /// API id → providing mod name.
    pub api_providers: std::collections::BTreeMap<String, String>,
}

/// Full resolve: incompatibilities, required deps, optional notes, API bindings, load order.
pub fn resolve_mod_graph(manifests: &[ModManifest]) -> Result<ModResolveReport, DepError> {
    let by_name: HashMap<&str, &ModManifest> = manifests.iter().map(|m| (m.name.as_str(), m)).collect();
    let mut notes = Vec::new();

    for m in manifests {
        for (blocked, _req) in &m.incompatibilities {
            if by_name.contains_key(blocked.as_str()) {
                return Err(DepError::Incompatible(m.name.clone(), blocked.clone()));
            }
        }
    }

    for m in manifests {
        for (dep, req) in &m.optional_dependencies {
            if dep == "base" {
                continue;
            }
            if !by_name.contains_key(dep.as_str()) {
                notes.push(format!(
                    "Optional dependency `{}` ({}) not loaded for `{}` — features may be limited",
                    dep, req, m.name
                ));
                continue;
            }
            let other = by_name[dep.as_str()];
            let dep_version = semver::Version::parse(&other.version).map_err(|_| {
                DepError::VersionMismatch {
                    mod_name: m.name.clone(),
                    dep: dep.clone(),
                    required: req.clone(),
                }
            })?;
            let requirement = VersionReq::parse(req).map_err(|_| DepError::VersionMismatch {
                mod_name: m.name.clone(),
                dep: dep.clone(),
                required: req.clone(),
            })?;
            if !requirement.matches(&dep_version) {
                notes.push(format!(
                    "Optional dependency `{}` version mismatch for `{}` (wanted {})",
                    dep, m.name, req
                ));
            }
        }
    }

    let mut api_index: HashMap<String, (&str, semver::Version)> = HashMap::new();
    for m in manifests {
        for (api_id, ver_str) in &m.provides {
            let ver = semver::Version::parse(ver_str).map_err(|_| DepError::ApiConflict {
                api: api_id.clone(),
                detail: format!("invalid version `{}` on mod `{}`", ver_str, m.name),
            })?;
            if let Some((other_mod, _)) = api_index.get(api_id) {
                return Err(DepError::ApiConflict {
                    api: api_id.clone(),
                    detail: format!("provided by both `{}` and `{}`", other_mod, m.name),
                });
            }
            api_index.insert(api_id.clone(), (m.name.as_str(), ver));
        }
    }

    let mut api_providers = std::collections::BTreeMap::new();
    for m in manifests {
        for (api_id, req) in &m.api_dependencies {
            let (provider, ver) = api_index.get(api_id).ok_or_else(|| DepError::ApiMissing {
                mod_name: m.name.clone(),
                api: api_id.clone(),
            })?;
            let requirement = VersionReq::parse(req).map_err(|_| DepError::ApiConflict {
                api: api_id.clone(),
                detail: format!("invalid range `{}` on mod `{}`", req, m.name),
            })?;
            if !requirement.matches(ver) {
                return Err(DepError::ApiConflict {
                    api: api_id.clone(),
                    detail: format!(
                        "provider `{}` has {} but `{}` needs {}",
                        provider, ver, m.name, req
                    ),
                });
            }
            api_providers.insert(api_id.clone(), provider.to_string());
        }
    }

    let load_order = resolve_load_order(manifests)?;
    Ok(ModResolveReport {
        load_order,
        notes,
        api_providers,
    })
}

/// Topological load order with `base` virtual mod at version 0.1.0.
pub fn resolve_load_order(manifests: &[ModManifest]) -> Result<Vec<String>, DepError> {
    let by_name: HashMap<&str, &ModManifest> = manifests.iter().map(|m| (m.name.as_str(), m)).collect();

    for m in manifests {
        for (dep, req) in &m.dependencies {
            if dep == "base" {
                let version = semver::Version::parse("0.1.0").expect("valid");
                let requirement = VersionReq::parse(req).map_err(|_| DepError::VersionMismatch {
                    mod_name: m.name.clone(),
                    dep: dep.clone(),
                    required: req.clone(),
                })?;
                if !requirement.matches(&version) {
                    return Err(DepError::VersionMismatch {
                        mod_name: m.name.clone(),
                        dep: dep.clone(),
                        required: req.clone(),
                    });
                }
                continue;
            }
            if !by_name.contains_key(dep.as_str()) {
                return Err(DepError::Missing(dep.clone(), m.name.clone()));
            }
            let dep_manifest = by_name[dep.as_str()];
            let dep_version = semver::Version::parse(&dep_manifest.version).map_err(|_| {
                DepError::VersionMismatch {
                    mod_name: m.name.clone(),
                    dep: dep.clone(),
                    required: req.clone(),
                }
            })?;
            let requirement = VersionReq::parse(req).map_err(|_| DepError::VersionMismatch {
                mod_name: m.name.clone(),
                dep: dep.clone(),
                required: req.clone(),
            })?;
            if !requirement.matches(&dep_version) {
                return Err(DepError::VersionMismatch {
                    mod_name: m.name.clone(),
                    dep: dep.clone(),
                    required: req.clone(),
                });
            }
        }
    }

    let mut indegree: BTreeMap<String, usize> = BTreeMap::new();
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for m in manifests {
        indegree.entry(m.name.clone()).or_insert(0);
        for dep in m.dependencies.keys() {
            if dep == "base" {
                continue;
            }
            edges.entry(dep.clone()).or_default().push(m.name.clone());
            indegree
                .entry(m.name.clone())
                .and_modify(|d| *d += 1)
                .or_insert(1);
        }
    }

    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(n, _)| n.clone())
        .collect();

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node.clone());
        if let Some(children) = edges.get(&node) {
            for child in children {
                if let Some(entry) = indegree.get_mut(child) {
                    if *entry == 0 {
                        continue;
                    }
                    *entry -= 1;
                    if *entry == 0 {
                        queue.push_back(child.clone());
                    }
                }
            }
        }
    }

    if order.len() != manifests.len() {
        let stuck = manifests
            .iter()
            .find(|m| !order.contains(&m.name))
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "unknown".into());
        return Err(DepError::Cycle(stuck));
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn manifest(name: &str, deps: &[(&str, &str)]) -> ModManifest {
        ModManifest {
            name: name.into(),
            version: "0.1.0".into(),
            title: name.into(),
            author: String::new(),
            unibrowser_api: "0.1.0".into(),
            dependencies: deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<BTreeMap<_, _>>(),
            optional_dependencies: BTreeMap::new(),
            incompatibilities: BTreeMap::new(),
            provides: BTreeMap::new(),
            api_dependencies: BTreeMap::new(),
            description: String::new(),
            library: None,
            wasm_module: None,
            capabilities: vec![],
            capability_provides: vec![],
            contributes: None,
            permissions: None,
        }
    }

    #[test]
    fn optional_missing_is_non_blocking_note() {
        let m = manifest("consumer", &[]);
        let mut opt = m.clone();
        opt.optional_dependencies
            .insert("missing-mod".into(), ">=0.1.0".into());
        let report = resolve_mod_graph(&[opt]).expect("report");
        assert!(report.notes.iter().any(|n| n.contains("Optional")));
    }

    #[test]
    fn api_dependency_binds_provider() {
        let mut a = manifest("api-proving-a", &[]);
        a.provides.insert("audio.levels".into(), "1.0.0".into());
        let mut b = manifest("api-proving-b", &[]);
        b.api_dependencies.insert("audio.levels".into(), "^1.0".into());
        let report = resolve_mod_graph(&[a, b]).expect("report");
        assert_eq!(
            report.api_providers.get("audio.levels").map(String::as_str),
            Some("api-proving-a")
        );
    }

    #[test]
    fn unmet_api_dependency_errors_for_required_consumer() {
        let b = manifest("api-proving-b", &[]);
        let mut b = b;
        b.api_dependencies.insert("audio.levels".into(), "^1.0".into());
        let err = resolve_mod_graph(&[b]).unwrap_err();
        assert!(matches!(err, DepError::ApiMissing { .. }));
    }

    #[test]
    fn resolves_base_dependency() {
        let m = manifest("base-text", &[("base", ">=0.1.0")]);
        let order = resolve_load_order(&[m]).expect("order");
        assert_eq!(order, vec!["base-text"]);
    }

    #[test]
    fn detects_missing_dep() {
        let m = manifest("orphan", &[("base-text", ">=0.1.0")]);
        let err = resolve_load_order(&[m]).unwrap_err();
        assert!(matches!(err, DepError::Missing(_, _)));
    }
}
