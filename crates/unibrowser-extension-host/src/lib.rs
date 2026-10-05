//! Minimal extension-host surface for VSIX command execution ([ADR 0018](../../docs/adr/0018-extension-host-stub.md)).
//! See [docs/extension-host.md](../../docs/extension-host.md).

mod activation;

pub use activation::{ActivationRecord, activated_extensions, matching_on_command_event, record_command_activation};

pub const HOST_VERSION: &str = "0.1.0-stub";

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
    HostStatus {
        running: false,
        capabilities: vec![
            HostCapability::CommandsMetadata,
            HostCapability::KeybindingsMetadata,
        ],
        message: "Extension host not started — contributed commands run stub handlers only.".into(),
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
