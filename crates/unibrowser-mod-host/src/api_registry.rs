//! Host-mediated mod API registry (ADR 0025).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use semver::{Version, VersionReq};

#[derive(Debug, Clone)]
pub struct ProvidedApi {
    pub api_id: String,
    pub version: Version,
    pub provider_mod: String,
}

/// Simple proving surface for `audio.levels@1`.
pub trait AudioLevelsApi: Send + Sync {
    fn peak_level(&self) -> f64;
}

struct AudioEntry {
    meta: ProvidedApi,
    handler: Arc<dyn AudioLevelsApi>,
}

#[derive(Default)]
pub struct ModApiRegistry {
    audio_levels: Mutex<HashMap<String, AudioEntry>>,
    notes: Mutex<Vec<String>>,
}

impl ModApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_notes(&self) -> Vec<String> {
        self.notes.lock().map(|n| n.clone()).unwrap_or_default()
    }

    pub fn note(&self, msg: impl Into<String>) {
        if let Ok(mut n) = self.notes.lock() {
            n.push(msg.into());
        }
    }

    pub fn provide_audio_levels(
        &self,
        provider_mod: &str,
        version: &str,
        handler: Arc<dyn AudioLevelsApi>,
    ) -> Result<(), String> {
        let ver = Version::parse(version).map_err(|e| e.to_string())?;
        let api_id = "audio.levels".to_string();
        let mut map = self.audio_levels.lock().map_err(|e| e.to_string())?;
        if map.contains_key(&api_id) {
            return Err(format!(
                "API {} already provided by another mod",
                api_id
            ));
        }
        map.insert(
            api_id.clone(),
            AudioEntry {
                meta: ProvidedApi {
                    api_id,
                    version: ver,
                    provider_mod: provider_mod.to_string(),
                },
                handler,
            },
        );
        Ok(())
    }

    pub fn resolve_audio_levels(
        &self,
        requirement: &str,
    ) -> Result<Arc<dyn AudioLevelsApi>, String> {
        let map = self.audio_levels.lock().map_err(|e| e.to_string())?;
        let entry = map
            .get("audio.levels")
            .ok_or_else(|| "API audio.levels not provided".to_string())?;
        let req = VersionReq::parse(requirement).map_err(|e| e.to_string())?;
        if !req.matches(&entry.meta.version) {
            return Err(format!(
                "audio.levels version {} does not satisfy {}",
                entry.meta.version,
                requirement
            ));
        }
        Ok(entry.handler.clone())
    }

    pub fn list_provided(&self) -> Vec<ProvidedApi> {
        self.audio_levels
            .lock()
            .map(|m| m.values().map(|e| e.meta.clone()).collect())
            .unwrap_or_default()
    }
}
