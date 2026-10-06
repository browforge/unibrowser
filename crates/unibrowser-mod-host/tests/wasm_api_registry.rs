//! WASM host routes mod API calls through `ModApiRegistry` (ADR 0025).

use std::sync::Arc;

use unibrowser_mod_host::{AudioLevelsApi, ModApiRegistry};

struct StubLevels(f64);
impl AudioLevelsApi for StubLevels {
    fn peak_level(&self) -> f64 {
        self.0
    }
}

#[test]
fn guest_audio_levels_peak_uses_registry() {
    let registry = Arc::new(ModApiRegistry::new());
    registry
        .provide_audio_levels("audio-mod", "1.0.0", Arc::new(StubLevels(0.77)))
        .expect("provide");
    assert!((registry.guest_audio_levels_peak("^1.0") - 0.77).abs() < 0.001);
    assert_eq!(registry.guest_audio_levels_peak("^9.0"), -1.0);
}
