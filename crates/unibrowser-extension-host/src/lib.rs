//! Minimal extension-host surface for VSIX command execution ([ADR 0018](../../docs/adr/0018-extension-host-stub.md)).
//! See [docs/extension-host.md](../../docs/extension-host.md).

mod activation;
mod node_host;

pub use activation::{ActivationRecord, activated_extensions, matching_on_command_event, record_command_activation};
pub use node_host::{
    execute_extension_command, host_script_path, node_on_path, vsix_palette_enablement,
    NodeExecuteResult, VSIX_NODE_HOST_NO_MAIN, VSIX_NODE_HOST_NO_NODE, VSIX_NODE_HOST_PREF_OFF,
};

pub const HOST_VERSION: &str = "0.2.0-node-spike";

pub const VSIX_STUB_ENABLED_HINT: &str =
    "VSIX stub — JavaScript extension host not started (see docs/extension-host.md)";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HostCapability {
    CommandsMetadata,
    KeybindingsMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostStatus {
    pub running: bool,
    pub capabilities: Vec<HostCapability>,
    pub message: String,
}

pub fn status() -> HostStatus {
    let node = node_on_path();
    let script = host_script_path().is_some();
    let message = if node && script {
        "Node sidecar available (opt-in via Settings → Advanced).".into()
    } else if !node {
        "Node.js not on PATH — VSIX JavaScript host unavailable.".into()
    } else {
        "Node host script missing — VSIX JavaScript host unavailable.".into()
    };
    HostStatus {
        running: node && script,
        capabilities: vec![
            HostCapability::CommandsMetadata,
            HostCapability::KeybindingsMetadata,
        ],
        message,
    }
}

/// Run a contributed command without loading extension JavaScript.
pub fn execute_vsix_stub(command_id: &str, extension_id: &str, activation_events: &[String]) -> String {
    let activated = record_command_activation(extension_id, command_id, activation_events);
    let activation = if activation_events.is_empty() {
        "none declared".to_string()
    } else {
        activation_events.join(", ")
    };
    let state = match activated {
        Some((rec, true)) => format!("activated via `{}`", rec.matched_event),
        Some((rec, false)) => format!("already activated via `{}`", rec.matched_event),
        None => "not activated (no matching onCommand event)".into(),
    };
    format!(
        "VSIX stub: `{}` (extension `{}`). Activation events: {}. State: {}. Full JS host not started.",
        command_id,
        extension_id,
        activation,
        state
    )
}
