//! Versioned runtime callbacks for mod capability and emulation (CMD-08).

use serde::{Deserialize, Serialize};

pub const MOD_RUNTIME_API_V1: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModPermissions(pub u32);

impl ModPermissions {
    pub const PROVIDE_CAPABILITY: u32 = 1;
    pub const READ_SELECTION: u32 = 2;
    pub const EDIT_DOCUMENT: u32 = 4;
    pub const COMMAND_MANAGER: u32 = 8;
    pub const TOOLBAR: u32 = 16;
    pub const DETACH_TOOLBAR: u32 = 32;

    pub fn has(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCapabilityKind {
    Supported,
    Emulated,
    Unsupported,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlRuntimeSnapshot {
    pub api_version: u32,
    pub control_id: String,
    pub capability: RuntimeCapabilityKind,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub emulation_note: Option<String>,
    #[serde(default)]
    pub selection: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub mixed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModRuntimeDescriptor {
    pub mod_name: String,
    pub provider_id: String,
    pub api_version: u32,
    pub permissions: ModPermissions,
}
