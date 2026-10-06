//! Bundled AI assistant mod — palette hook; UI in desktop shell.

use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{CommandMeta, Enablement};

pub const BACKEND_ID: &str = "ai-assistant";
pub const CMD_TOGGLE: &str = "ai-assistant.toggle";

pub fn register(registry: &mut CommandRegistry) {
    registry.register_command(CommandMeta {
        id: CMD_TOGGLE,
        title: "Toggle AI assistant",
        icon: "sparkle",
        default_keybinding: None,
        aliases: &["ai sidebar", "chat assistant"],
    });
    registry.register_handler(unibrowser_engine::HandlerRegistration {
        command_id: CMD_TOGGLE.into(),
        backend_id: BACKEND_ID.into(),
        enable: |_| Enablement::Enabled,
        execute: |_| Ok("Toggle AI assistant panel from the sidebar control.".into()),
    });
}
