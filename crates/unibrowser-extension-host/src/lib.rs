//! Minimal extension-host surface for VSIX command execution ([ADR 0018](../../docs/adr/0018-extension-host-stub.md)).
//! See [docs/extension-host.md](../../docs/extension-host.md).

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
    let activation = if activation_events.is_empty() {
        "none declared".to_string()
    } else {
        activation_events.join(", ")
    };
    format!(
        "VSIX stub: `{}` (extension `{}`). Activation events: {}. Full host not started.",
        command_id,
        extension_id,
        activation
    )
}
