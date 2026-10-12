//! Declarative mod contributions for the command system (CMD-07).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::content_kind::{validate_content_kind_id, ContentKindRegistry};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModContributes {
    #[serde(default)]
    pub commands: Vec<ContributedCommand>,
    #[serde(default)]
    pub controls: Vec<ContributedControl>,
    #[serde(default)]
    pub groups: Vec<ContributedGroup>,
    #[serde(default, rename = "command_families", alias = "families")]
    pub families: Vec<ContributedFamily>,
    #[serde(default)]
    pub tabs: Vec<ContributedTab>,
    #[serde(default)]
    pub toolbars: Vec<ContributedToolbar>,
    #[serde(default)]
    pub capability_providers: Vec<ContributedCapabilityProvider>,
    #[serde(default)]
    pub panels: Vec<ContributedPanel>,
    #[serde(default)]
    pub views: Vec<ContributedView>,
    #[serde(default)]
    pub status_items: Vec<ContributedStatusItem>,
    #[serde(default)]
    pub services: Vec<ContributedService>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedCommand {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedControl {
    pub id: String,
    #[serde(rename = "type")]
    pub control_type: String,
    pub command: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub insert: Option<InsertPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InsertPoint {
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedGroup {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub controls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedFamily {
    pub id: String,
    #[serde(default)]
    pub extend: bool,
    #[serde(default)]
    pub controls: Vec<ContributedControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedTab {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub families: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedToolbar {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub floatable: bool,
    #[serde(default)]
    pub detachable: bool,
    /// When false, toolbar layout is omitted until demo/dev flags enable proving toolbars.
    #[serde(default = "default_true")]
    pub default_visible: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedCapabilityProvider {
    pub id: String,
    #[serde(default)]
    pub families: Vec<String>,
    #[serde(default)]
    pub contexts: Vec<String>,
    #[serde(default)]
    pub content_kinds: Vec<String>,
    #[serde(default)]
    pub host_formats: Vec<String>,
    #[serde(default)]
    pub format_states: Vec<ContributedHostFormatCapability>,
    #[serde(default)]
    pub runtime: String,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedHostFormatCapability {
    pub host_format: String,
    /// `supported`, `emulated`, or `unsupported`
    pub state: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedPanel {
    pub id: String,
    pub region: String,
    pub title: String,
    #[serde(default)]
    pub content_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedView {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedStatusItem {
    pub id: String,
    pub slot: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub content_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributedService {
    pub id: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributionValidationError {
    pub mod_name: String,
    pub message: String,
}

pub fn validate_contributes(mod_name: &str, c: &ModContributes) -> Vec<ContributionValidationError> {
    validate_contributes_with_kinds(mod_name, c, &ContentKindRegistry::with_core_kinds())
}

pub fn validate_contributes_with_kinds(
    mod_name: &str,
    c: &ModContributes,
    kinds: &ContentKindRegistry,
) -> Vec<ContributionValidationError> {
    let mut errors = Vec::new();
    for cmd in &c.commands {
        if cmd.id.starts_with("unibrowser.") {
            errors.push(ContributionValidationError {
                mod_name: mod_name.to_string(),
                message: format!("mod cannot claim core command id {}", cmd.id),
            });
        }
        if !cmd.id.starts_with(&format!("{}.", mod_name)) && !cmd.id.contains('.') {
            errors.push(ContributionValidationError {
                mod_name: mod_name.to_string(),
                message: format!("command id {} must be namespaced", cmd.id),
            });
        }
    }
    for fam in &c.families {
        if fam.id.starts_with("unibrowser.") && !fam.extend {
            errors.push(ContributionValidationError {
                mod_name: mod_name.to_string(),
                message: format!("mod cannot replace core family {}", fam.id),
            });
        }
    }
    for panel in &c.panels {
        if !["left", "right", "bottom"].contains(&panel.region.as_str()) {
            errors.push(ContributionValidationError {
                mod_name: mod_name.to_string(),
                message: format!("panel {} has invalid region {}", panel.id, panel.region),
            });
        }
    }
    for item in &c.status_items {
        if !["left", "right"].contains(&item.slot.as_str()) {
            errors.push(ContributionValidationError {
                mod_name: mod_name.to_string(),
                message: format!("status item {} has invalid slot {}", item.id, item.slot),
            });
        }
    }
    for provider in &c.capability_providers {
        for kind in &provider.content_kinds {
            if let Err(msg) = validate_content_kind_id(kind, None) {
                errors.push(ContributionValidationError {
                    mod_name: mod_name.to_string(),
                    message: msg,
                });
            } else if let Err(msg) = kinds.validate_reference(kind) {
                errors.push(ContributionValidationError {
                    mod_name: mod_name.to_string(),
                    message: msg,
                });
            }
        }
    }
    errors
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandLayoutFileV2 {
    pub layout_version: u32,
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextLayout>,
    #[serde(default)]
    pub orphaned_contributions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextLayout {
    #[serde(default)]
    pub family_order: Vec<String>,
    #[serde(default)]
    pub hidden_controls: Vec<HiddenControlEntry>,
    #[serde(default)]
    pub provider_overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenControlEntry {
    pub control_id: String,
    #[serde(default)]
    pub all_contexts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorAnchor {
    pub monitor_name: String,
    pub monitor_edid: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarLayoutEntry {
    pub toolbar_id: String,
    pub anchor: MonitorAnchor,
}

pub const LAYOUT_VERSION_V2: u32 = 2;

#[cfg(test)]
mod validate_tests {
    use super::{ContributedCommand, ModContributes, validate_contributes};

    #[test]
    fn rejects_core_command_namespace() {
        let c = ModContributes {
            commands: vec![ContributedCommand {
                id: "unibrowser.text.evil".into(),
                title: "Evil".into(),
                icon: String::new(),
            }],
            ..ModContributes::default()
        };
        let errs = validate_contributes("acme-mod", &c);
        assert!(!errs.is_empty());
    }
}
