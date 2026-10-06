use std::collections::HashMap;
use std::time::Duration;

use unibrowser_mod_api::{
    CapabilityState, ProviderCandidate, ProviderResolution, ProviderSource,
};

use super::context::ResolutionContext;

const CALLBACK_TIMEOUT: Duration = Duration::from_millis(120);

#[derive(Debug, Clone)]
pub struct ModProvider {
    pub provider_id: String,
    pub mod_name: String,
    pub priority: i32,
    pub dependency_index: usize,
    pub families: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderRegistry {
    pub mod_providers: Vec<ModProvider>,
    pub backend_provider_id: String,
    pub diagnostics: Vec<String>,
}

#[derive(Default)]
pub struct ProviderRuntime {
    pub last_resolution: Option<ProviderResolution>,
}

impl ProviderRegistry {
    pub fn register_mod_provider(&mut self, provider: ModProvider) {
        self.mod_providers.push(provider);
        self.mod_providers.sort_by(|a, b| {
            a.dependency_index
                .cmp(&b.dependency_index)
                .then(b.priority.cmp(&a.priority))
                .then(a.provider_id.cmp(&b.provider_id))
        });
    }

    pub fn apply_override(
        &self,
        control_id: &str,
        _command_id: &str,
        base: CapabilityState,
        ctx: &ResolutionContext,
        user_overrides: &HashMap<String, String>,
    ) -> CapabilityState {
        let mut candidates = self.collect_candidates(control_id, ctx);
        let winner = if let Some(user) = user_overrides.get(control_id) {
            candidates.push(ProviderCandidate {
                provider_id: user.clone(),
                source: ProviderSource::UserSelected,
                priority: i32::MAX,
                dependency_index: 0,
            });
            Some(user.clone())
        } else if !self.backend_provider_id.is_empty() {
            candidates.push(ProviderCandidate {
                provider_id: self.backend_provider_id.clone(),
                source: ProviderSource::Backend,
                priority: 1000,
                dependency_index: 0,
            });
            Some(self.backend_provider_id.clone())
        } else {
            candidates.first().map(|c| c.provider_id.clone())
        };

        let reason = match &winner {
            Some(id) if user_overrides.contains_key(control_id) => {
                format!("User selected provider {}", id)
            }
            Some(id) => format!("Active backend provider {}", id),
            None => "No provider; using family default".into(),
        };

        let _resolution = ProviderResolution {
            winner,
            reason,
            candidates,
        };

        if user_overrides.contains_key(control_id) {
            return base;
        }
        base
    }

    fn collect_candidates(&self, control_id: &str, _ctx: &ResolutionContext) -> Vec<ProviderCandidate> {
        self.mod_providers
            .iter()
            .map(|p| ProviderCandidate {
                provider_id: p.provider_id.clone(),
                source: ProviderSource::Mod,
                priority: p.priority,
                dependency_index: p.dependency_index,
            })
            .collect()
    }

    pub fn callback_timeout(&self) -> Duration {
        CALLBACK_TIMEOUT
    }

    pub fn note_failure(&mut self, provider_id: &str, detail: &str) {
        self.diagnostics.push(format!(
            "Provider {} failed: {} (degraded to unsupported)",
            provider_id,
            detail
        ));
    }

    pub fn failure_capability(provider_id: &str, detail: &str) -> CapabilityState {
        CapabilityState::Unsupported {
            reason: format!("Provider {} failed: {}", provider_id, detail),
        }
    }
}
