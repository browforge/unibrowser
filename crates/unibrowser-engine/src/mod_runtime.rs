//! Native and WASM mod runtime registration (CMD-08).

use std::collections::HashMap;

use unibrowser_mod_api::{
    CapabilityState, ControlRuntimeSnapshot, ModPermissions, ModRuntimeDescriptor,
    ResolvedControl, RuntimeCapabilityKind, SelectionState, MOD_RUNTIME_API_V1,
};

pub type NativeSnapshotFn = fn(control_id: &str) -> Option<ControlRuntimeSnapshot>;
pub type NativeExecuteFn = fn(command_id: &str) -> Result<String, String>;

#[derive(Clone)]
pub struct NativeModRuntime {
    pub descriptor: ModRuntimeDescriptor,
    pub snapshot: NativeSnapshotFn,
    pub execute: Option<NativeExecuteFn>,
}

#[derive(Default)]
pub struct ModRuntimeRegistry {
    pub natives: HashMap<String, NativeModRuntime>,
    pub diagnostics: Vec<String>,
}

impl ModRuntimeRegistry {
    pub fn register_native(&mut self, runtime: NativeModRuntime) {
        if runtime.descriptor.api_version != MOD_RUNTIME_API_V1 {
            self.diagnostics.push(format!(
                "mod {} runtime api {} != host {}",
                runtime.descriptor.mod_name,
                runtime.descriptor.api_version,
                MOD_RUNTIME_API_V1
            ));
            return;
        }
        if !runtime.descriptor.permissions.has(ModPermissions::PROVIDE_CAPABILITY) {
            self.diagnostics.push(format!(
                "mod {} missing PROVIDE_CAPABILITY permission",
                runtime.descriptor.mod_name
            ));
            return;
        }
        self.natives
            .insert(runtime.descriptor.provider_id.clone(), runtime);
    }

    pub fn snapshot_for(&self, provider_id: &str, control_id: &str) -> Option<ControlRuntimeSnapshot> {
        self.natives
            .get(provider_id)
            .and_then(|n| (n.snapshot)(control_id))
    }

    pub fn apply_to_control(&self, control: &mut ResolvedControl) {
        let provider_id = control
            .provider_id
            .as_deref()
            .or_else(|| match &control.capability {
                CapabilityState::Emulated { provider_id, .. } => Some(provider_id.as_str()),
                _ => None,
            });
        let Some(provider_id) = provider_id else {
            return;
        };
        let Some(snap) = self.snapshot_for(provider_id, &control.control_id) else {
            return;
        };
        control.capability = runtime_kind_to_capability(provider_id, &snap);
        control.selection = parse_runtime_selection(&snap.selection, snap.mixed);
        if snap.value.is_some() {
            control.value = snap.value.clone();
        }
    }

    pub fn execute(&self, provider_id: &str, command_id: &str) -> Result<String, String> {
        let native = self
            .natives
            .get(provider_id)
            .ok_or_else(|| format!("unknown runtime provider {}", provider_id))?;
        if !native.descriptor.permissions.has(ModPermissions::EDIT_DOCUMENT) {
            return Err("mod lacks EDIT_DOCUMENT permission".into());
        }
        let run = native
            .execute
            .ok_or_else(|| "mod runtime has no execute callback".to_string())?;
        run(command_id)
    }

    pub fn wasm_mirror_snapshot(
        mod_name: &str,
        native: &NativeModRuntime,
        control_id: &str,
    ) -> Option<ControlRuntimeSnapshot> {
        let mirror_id = format!("{}::wasm-mirror", mod_name);
        if native.descriptor.provider_id == mirror_id {
            return None;
        }
        (native.snapshot)(control_id).map(|snap| ControlRuntimeSnapshot {
            api_version: snap.api_version,
            control_id: snap.control_id,
            capability: snap.capability,
            reason: snap.reason,
            emulation_note: snap.emulation_note,
            selection: snap.selection,
            value: snap.value,
            mixed: snap.mixed,
        })
    }

    pub fn wasm_mirror_descriptor(mod_name: &str) -> ModRuntimeDescriptor {
        ModRuntimeDescriptor {
            mod_name: mod_name.into(),
            provider_id: format!("{}::wasm-mirror", mod_name),
            api_version: MOD_RUNTIME_API_V1,
            permissions: ModPermissions(ModPermissions::PROVIDE_CAPABILITY),
        }
    }
}

fn runtime_kind_to_capability(
    provider_id: &str,
    snap: &ControlRuntimeSnapshot,
) -> CapabilityState {
    match snap.capability {
        RuntimeCapabilityKind::Supported => CapabilityState::Supported,
        RuntimeCapabilityKind::Emulated => CapabilityState::Emulated {
            provider_id: provider_id.into(),
            note: snap
                .emulation_note
                .clone()
                .unwrap_or_else(|| "Approximated by mod".into()),
        },
        RuntimeCapabilityKind::Unsupported => CapabilityState::Unsupported {
            reason: snap
                .reason
                .clone()
                .unwrap_or_else(|| "Unsupported in this context".into()),
        },
        RuntimeCapabilityKind::Locked => CapabilityState::Locked {
            reason: snap
                .reason
                .clone()
                .unwrap_or_else(|| "Locked".into()),
        },
    }
}

fn parse_runtime_selection(raw: &str, mixed: bool) -> SelectionState {
    if mixed {
        return SelectionState::Mixed;
    }
    match raw {
        "on" => SelectionState::On,
        "mixed" => SelectionState::Mixed,
        "indeterminate" => SelectionState::Indeterminate,
        "value" => SelectionState::Value,
        "mixed_value" => SelectionState::MixedValue,
        _ => SelectionState::Off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_snapshot(control_id: &str) -> Option<ControlRuntimeSnapshot> {
        Some(ControlRuntimeSnapshot {
            api_version: MOD_RUNTIME_API_V1,
            control_id: control_id.into(),
            capability: RuntimeCapabilityKind::Emulated,
            reason: None,
            emulation_note: Some("stub".into()),
            selection: "off".into(),
            value: None,
            mixed: false,
        })
    }

    #[test]
    fn wasm_mirror_matches_native_snapshot() {
        let mut reg = ModRuntimeRegistry::default();
        let native = NativeModRuntime {
            descriptor: ModRuntimeDescriptor {
                mod_name: "cmd-proving".into(),
                provider_id: "cmd-proving::native".into(),
                api_version: MOD_RUNTIME_API_V1,
                permissions: ModPermissions(ModPermissions::PROVIDE_CAPABILITY),
            },
            snapshot: stub_snapshot,
            execute: None,
        };
        reg.register_native(native.clone());
        let a = reg.snapshot_for("cmd-proving::native", "x").expect("snap");
        let b = ModRuntimeRegistry::wasm_mirror_snapshot("cmd-proving", &native, "x").expect("mirror");
        assert_eq!(a, b);
    }

    #[test]
    fn registers_native_provider() {
        let mut reg = ModRuntimeRegistry::default();
        reg.register_native(NativeModRuntime {
            descriptor: ModRuntimeDescriptor {
                mod_name: "test-mod".into(),
                provider_id: "test-mod::native".into(),
                api_version: MOD_RUNTIME_API_V1,
                permissions: ModPermissions(ModPermissions::PROVIDE_CAPABILITY),
            },
            snapshot: stub_snapshot,
            execute: None,
        });
        assert!(reg.snapshot_for("test-mod::native", "x").is_some());
    }
}
