//! Native and WASM mod runtime registration (CMD-08).

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use unibrowser_mod_api::{
    CapabilityState, ControlRuntimeSnapshot, HostEditRequest, ModPermissions,
    ModRuntimeDescriptor, ResolvedControl, RuntimeCallbackError, RuntimeCapabilityKind,
    SelectionState, MOD_CAPABILITY_TIMEOUT, MOD_EXECUTION_TIMEOUT, MOD_RUNTIME_API_V1,
};

pub type NativeSnapshotFn =
    fn(control_id: &str) -> Result<Option<ControlRuntimeSnapshot>, RuntimeCallbackError>;
pub type NativeExecuteFn =
    fn(command_id: &str) -> Result<HostEditRequest, RuntimeCallbackError>;

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

    pub fn snapshot_for(
        &self,
        provider_id: &str,
        control_id: &str,
    ) -> Result<Option<ControlRuntimeSnapshot>, RuntimeCallbackError> {
        let runtime = self
            .natives
            .get(provider_id)
            .ok_or_else(|| RuntimeCallbackError::UnknownProvider(provider_id.into()))?;
        let callback = runtime.snapshot;
        let control_id_owned = control_id.to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(callback(&control_id_owned));
        });
        let snapshot = rx
            .recv_timeout(MOD_CAPABILITY_TIMEOUT)
            .map_err(|_| RuntimeCallbackError::TimedOut)??;
        if let Some(snapshot) = &snapshot {
            validate_snapshot(snapshot, control_id)?;
        }
        Ok(snapshot)
    }

    pub fn apply_to_control(
        &self,
        control: &mut ResolvedControl,
    ) -> Result<(), RuntimeCallbackError> {
        if control
            .command_ids
            .iter()
            .any(|id| crate::command_system::save_commands_always_supported(id))
        {
            return Ok(());
        }
        let provider_id = control
            .provider_id
            .as_deref()
            .or_else(|| match &control.capability {
                CapabilityState::Emulated { provider_id, .. } => Some(provider_id.as_str()),
                _ => None,
            });
        let Some(provider_id) = provider_id else {
            return Ok(());
        };
        let runtime = self
            .natives
            .get(provider_id)
            .ok_or_else(|| RuntimeCallbackError::UnknownProvider(provider_id.into()))?;
        let Some(snap) = self.snapshot_for(provider_id, &control.control_id)? else {
            return Ok(());
        };
        control.capability = runtime_kind_to_capability(provider_id, &snap);
        if runtime
            .descriptor
            .permissions
            .has(ModPermissions::READ_SELECTION)
        {
            control.selection = parse_runtime_selection(&snap.selection, snap.mixed);
            if snap.value.is_some() {
                control.value = snap.value.clone();
            }
        }
        Ok(())
    }

    pub fn execute(
        &self,
        provider_id: &str,
        command_id: &str,
    ) -> Result<HostEditRequest, RuntimeCallbackError> {
        let native = self
            .natives
            .get(provider_id)
            .ok_or_else(|| RuntimeCallbackError::UnknownProvider(provider_id.into()))?;
        if !native.descriptor.permissions.has(ModPermissions::EDIT_DOCUMENT) {
            return Err(RuntimeCallbackError::PermissionDenied(
                "mod lacks EDIT_DOCUMENT permission".into(),
            ));
        }
        let run = native
            .execute
            .ok_or_else(|| RuntimeCallbackError::ProviderFailed(
                "mod runtime has no execute callback".into(),
            ))?;
        let command = command_id.to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(run(&command));
        });
        rx.recv_timeout(MOD_EXECUTION_TIMEOUT)
            .map_err(|_| RuntimeCallbackError::TimedOut)?
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

fn validate_snapshot(
    snapshot: &ControlRuntimeSnapshot,
    requested_control_id: &str,
) -> Result<(), RuntimeCallbackError> {
    if snapshot.api_version != MOD_RUNTIME_API_V1 {
        return Err(RuntimeCallbackError::InvalidResponse(format!(
            "snapshot api {} != host {}",
            snapshot.api_version, MOD_RUNTIME_API_V1
        )));
    }
    if snapshot.control_id != requested_control_id {
        return Err(RuntimeCallbackError::InvalidResponse(format!(
            "snapshot control {} != requested {}",
            snapshot.control_id, requested_control_id
        )));
    }
    Ok(())
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

    fn stub_snapshot(
        control_id: &str,
    ) -> Result<Option<ControlRuntimeSnapshot>, RuntimeCallbackError> {
        Ok(Some(ControlRuntimeSnapshot {
            api_version: MOD_RUNTIME_API_V1,
            control_id: control_id.into(),
            capability: RuntimeCapabilityKind::Emulated,
            reason: None,
            emulation_note: Some("stub".into()),
            selection: "off".into(),
            value: None,
            mixed: false,
        }))
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
        assert!(reg.snapshot_for("test-mod::native", "x").unwrap().is_some());
    }

    #[test]
    fn runtime_cannot_override_save_control() {
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
        let mut control = ResolvedControl {
            control_id: "test.save".into(),
            control_type: unibrowser_mod_api::ControlType::Button,
            command_ids: vec![unibrowser_mod_api::commands::TEXT_SAVE.into()],
            capability: CapabilityState::Supported,
            selection: SelectionState::Off,
            value: None,
            label: "Save".into(),
            short_label: None,
            icon: "save".into(),
            shortcut: None,
            density_variant: unibrowser_mod_api::DensityVariant::Compact,
            provider_id: Some("test-mod::native".into()),
            context_revision: 1,
            group_id: "test".into(),
            family_id: "test".into(),
        };
        reg.apply_to_control(&mut control).expect("save bypass");
        assert_eq!(control.capability, CapabilityState::Supported);
    }
}
