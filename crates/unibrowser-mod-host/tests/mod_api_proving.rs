//! ADR 0025 proving: mod A provides audio.levels@1, mod B calls it; missing API degrades.

use std::sync::Arc;

use unibrowser_mod_host::{AudioLevelsApi, ModApiRegistry, resolve_mod_graph};

struct LevelsA;
impl AudioLevelsApi for LevelsA {
    fn peak_level(&self) -> f64 {
        0.42
    }
}

fn manifests_ab() -> (unibrowser_mod_api::ModManifest, unibrowser_mod_api::ModManifest) {
    use std::collections::BTreeMap;
    let mut a = unibrowser_mod_api::ModManifest {
        name: "api-proving-a".into(),
        version: "0.1.0".into(),
        title: "API proving A".into(),
        author: String::new(),
        unibrowser_api: "0.1.0".into(),
        dependencies: BTreeMap::new(),
        optional_dependencies: BTreeMap::new(),
        incompatibilities: BTreeMap::new(),
        provides: BTreeMap::from([("audio.levels".into(), "1.0.0".into())]),
        api_dependencies: BTreeMap::new(),
        description: String::new(),
        library: None,
        wasm_module: None,
        capabilities: vec![],
        capability_provides: vec![],
        contributes: None,
        permissions: None,
    };
    let mut b = a.clone();
    b.name = "api-proving-b".into();
    b.title = "API proving B".into();
    b.provides.clear();
    b.api_dependencies
        .insert("audio.levels".into(), "^1.0".into());
    (a, b)
}

#[test]
fn mod_b_calls_mod_a_api() {
    let (a, b) = manifests_ab();
    let report = resolve_mod_graph(&[a, b]).expect("graph");
    assert_eq!(report.api_providers["audio.levels"], "api-proving-a");

    let registry = ModApiRegistry::new();
    registry
        .provide_audio_levels("api-proving-a", "1.0.0", Arc::new(LevelsA))
        .expect("provide");
    let api = registry.resolve_audio_levels("^1.0").expect("resolve");
    assert!((api.peak_level() - 0.42).abs() < 0.001);
}

#[test]
fn mod_b_degrades_when_api_missing() {
    let (_a, b) = manifests_ab();
    let registry = ModApiRegistry::new();
    let err = resolve_mod_graph(&[b.clone()]).unwrap_err();
    assert!(err.to_string().contains("audio.levels"));

    let peak = registry
        .resolve_audio_levels("^1.0")
        .map(|a| a.peak_level())
        .unwrap_or_else(|e| {
            registry.note(format!("degraded: {}", e));
            0.0
        });
    assert_eq!(peak, 0.0);
    assert!(registry.take_notes().iter().any(|n| n.contains("degraded")));
}
