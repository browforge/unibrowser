//! Sample mod: compact toolbar control + docked text_compact toolbar (TXT-05).

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{CommandMeta, Enablement};

pub const BACKEND_ID: &str = "text-toolbar-sample";

fn sample_enabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Enabled
}

fn sample_exec(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("sample-marker".into())
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: "text-toolbar-sample.insert.marker",
        title: "Insert sample marker",
        icon: "sparkles",
        default_keybinding: None,
        aliases: &["sample toolbar marker"],
    });
    registry.register_handler(HandlerRegistration {
        command_id: "text-toolbar-sample.insert.marker".into(),
        backend_id: BACKEND_ID.into(),
        enable: sample_enabled,
        execute: sample_exec,
    });
}
