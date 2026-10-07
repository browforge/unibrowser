//! Proving mod for command-system runtime, contributions, and detached toolbars.

use unibrowser_engine::{
    CommandRegistry, HandlerRegistration, ModRuntimeRegistry, NativeModRuntime,
    document::DocumentSession,
};
use unibrowser_mod_api::{
    CommandMeta, ControlRuntimeSnapshot, Enablement, ModPermissions, ModRuntimeDescriptor,
    RuntimeCapabilityKind, commands, control_ids, MOD_RUNTIME_API_V1,
};

pub const BACKEND_ID: &str = "cmd-proving";
pub const PROVIDER_ID: &str = "cmd-proving::native";
pub const SUBSCRIPT_APPLY: &str = "cmd-proving.subscript.apply";

fn markdown_backend(doc: &DocumentSession) -> bool {
    let id = doc.backend.backend_id();
    id == "toy-markdown" || id == "base-text"
}

fn subscript_enable(doc: &DocumentSession) -> Enablement {
    if !markdown_backend(doc) {
        return Enablement::Disabled {
            reason: "Subscript emulation needs a Markdown or text document".into(),
        };
    }
    Enablement::Enabled
}

fn exec_subscript(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("markdown-subscript-emulate".into())
}

fn exec_chip(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("markdown-subscript-emulate".into())
}

fn runtime_snapshot(control_id: &str) -> Option<ControlRuntimeSnapshot> {
    if control_id == control_ids::TEXT_SUBSCRIPT {
        return Some(ControlRuntimeSnapshot {
            api_version: MOD_RUNTIME_API_V1,
            control_id: control_id.into(),
            capability: RuntimeCapabilityKind::Emulated,
            reason: None,
            emulation_note: Some(
                "Wraps the selection in Markdown <sub> tags (proving mod).".into(),
            ),
            selection: "off".into(),
            value: None,
            mixed: false,
        });
    }
    if control_id == "cmd-proving.control.chip" {
        return Some(ControlRuntimeSnapshot {
            api_version: MOD_RUNTIME_API_V1,
            control_id: control_id.into(),
            capability: RuntimeCapabilityKind::Supported,
            reason: None,
            emulation_note: None,
            selection: "off".into(),
            value: None,
            mixed: false,
        });
    }
    None
}

fn runtime_execute(command_id: &str) -> Result<String, String> {
    if command_id == commands::TEXT_SUBSCRIPT || command_id == SUBSCRIPT_APPLY {
        return Ok("markdown-subscript-emulate".into());
    }
    Err(format!("cmd-proving runtime cannot execute {}", command_id))
}

pub fn register_runtime(registry: &mut ModRuntimeRegistry) {
    registry.register_native(NativeModRuntime {
        descriptor: ModRuntimeDescriptor {
            mod_name: "cmd-proving".into(),
            provider_id: PROVIDER_ID.into(),
            api_version: MOD_RUNTIME_API_V1,
            permissions: ModPermissions(
                ModPermissions::PROVIDE_CAPABILITY
                    | ModPermissions::READ_SELECTION
                    | ModPermissions::EDIT_DOCUMENT
                    | ModPermissions::COMMAND_MANAGER
                    | ModPermissions::TOOLBAR
                    | ModPermissions::DETACH_TOOLBAR,
            ),
        },
        snapshot: runtime_snapshot,
        execute: Some(runtime_execute),
    });
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: SUBSCRIPT_APPLY,
        title: "Apply subscript (proving)",
        icon: "subscript",
        default_keybinding: None,
        aliases: &["proving subscript"],
    });
    for (cmd, backend) in [
        (commands::TEXT_SUBSCRIPT, "toy-markdown"),
        (commands::TEXT_SUBSCRIPT, "base-text"),
        (SUBSCRIPT_APPLY, BACKEND_ID),
    ] {
        registry.register_handler(HandlerRegistration {
            command_id: cmd.into(),
            backend_id: backend.into(),
            enable: subscript_enable,
            execute: if cmd == SUBSCRIPT_APPLY {
                exec_chip
            } else {
                exec_subscript
            },
        });
    }
    registry.register_handler(HandlerRegistration {
        command_id: SUBSCRIPT_APPLY.into(),
        backend_id: "toy-markdown".into(),
        enable: subscript_enable,
        execute: exec_chip,
    });
    registry.register_handler(HandlerRegistration {
        command_id: SUBSCRIPT_APPLY.into(),
        backend_id: "base-text".into(),
        enable: subscript_enable,
        execute: exec_chip,
    });
}
