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
            description: String::new(),
            library: None,
            capabilities: vec![],
        }
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
