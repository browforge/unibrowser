//! First-party proving mod: grayscale preview on `unibrowser.content.image` (CMD-13/14).

use unibrowser_engine::{CommandRegistry, HandlerRegistration, ModRuntimeRegistry, NativeModRuntime};
use unibrowser_mod_api::{
    CommandMeta, ControlRuntimeSnapshot, Enablement, ModPermissions, ModRuntimeDescriptor,
    RuntimeCallbackError, RuntimeCapabilityKind, MOD_RUNTIME_API_V1,
};
use unibrowser_mod_api::content_kind::ids as ck;

pub const BACKEND_ID: &str = "image-grayscale-preview";
pub const PROVIDER_ID: &str = "image-grayscale-preview::native";
pub const GRAYSCALE_TOGGLE: &str = "image-grayscale-preview.grayscale.toggle";
pub const GRAYSCALE_CONTROL: &str = "image-grayscale-preview.control.grayscale";

fn image_selection(doc: &unibrowser_engine::document::DocumentSession) -> bool {
    let path = doc.path.to_ascii_lowercase();
    let tree = unibrowser_engine::content_tree::document_content_tree(
        doc.backend.backend_id(),
        &doc.path,
    );
    let kinds = unibrowser_engine::content_tree::selection_content_kinds(
        &tree,
        &[if path.ends_with(".md") || path.ends_with(".markdown") {
            "embedded-image-md".into()
        } else if path.ends_with(".docx") {
            "embedded-image-1".into()
        } else {
            tree.nodes
                .iter()
                .find(|n| n.kind_id == ck::IMAGE)
                .map(|n| n.node_id.clone())
                .unwrap_or_else(|| "root".into())
        }],
    );
    kinds.iter().any(|k| k == ck::IMAGE)
}

fn enable_grayscale(doc: &unibrowser_engine::document::DocumentSession) -> Enablement {
    if image_selection(doc) {
        Enablement::Enabled
    } else {
        Enablement::Disabled {
            reason: "Select an image node to preview in grayscale.".into(),
        }
    }
}

fn exec_grayscale(_doc: &mut unibrowser_engine::document::DocumentSession) -> Result<String, String> {
    Ok("grayscale-preview-toggle".into())
}

fn runtime_snapshot(
    control_id: &str,
) -> Result<Option<ControlRuntimeSnapshot>, RuntimeCallbackError> {
    if control_id != GRAYSCALE_CONTROL {
        return Ok(None);
    }
    Ok(Some(ControlRuntimeSnapshot {
        api_version: MOD_RUNTIME_API_V1,
        control_id: control_id.into(),
        capability: RuntimeCapabilityKind::Supported,
        reason: None,
        emulation_note: None,
        selection: "off".into(),
        value: None,
        mixed: false,
    }))
}

pub fn register_runtime(registry: &mut ModRuntimeRegistry) {
    registry.register_native(NativeModRuntime {
        descriptor: ModRuntimeDescriptor {
            mod_name: "image-grayscale-preview".into(),
            provider_id: PROVIDER_ID.into(),
            api_version: MOD_RUNTIME_API_V1,
            permissions: ModPermissions(
                ModPermissions::PROVIDE_CAPABILITY | ModPermissions::READ_SELECTION,
            ),
        },
        snapshot: runtime_snapshot,
        family_snapshot: None,
        execute: None,
    });
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: GRAYSCALE_TOGGLE,
        title: "Grayscale preview",
        icon: "image",
        default_keybinding: None,
        aliases: &["grayscale preview"],
    });
    for backend in [
        "base-png",
        "base-svg",
        "base-docx",
        "toy-markdown",
        "base-text",
        BACKEND_ID,
    ] {
        registry.register_handler(HandlerRegistration {
            command_id: GRAYSCALE_TOGGLE.into(),
            backend_id: backend.into(),
            enable: enable_grayscale,
            execute: exec_grayscale,
        });
    }
}
