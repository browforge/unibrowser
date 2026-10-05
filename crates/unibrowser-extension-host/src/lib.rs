//! Minimal extension-host surface for VSIX command execution ([ADR 0018](../../docs/adr/0018-extension-host-stub.md)).

pub const HOST_VERSION: &str = "0.0.0-stub";

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
        message: "Extension host not started — palette lists VSIX commands as disabled.".into(),
    }
}
