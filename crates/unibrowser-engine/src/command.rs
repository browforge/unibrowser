use std::collections::HashMap;

use unibrowser_mod_api::{
    CommandMeta, Enablement, HostContext, commands,
};

use crate::document::DocumentSession;

pub type EnableFn = fn(&DocumentSession) -> Enablement;
pub type ExecuteFn = fn(&mut DocumentSession) -> Result<String, String>;

#[derive(Clone)]
pub struct HandlerRegistration {
    pub command_id: String,
    pub backend_id: String,
    pub enable: EnableFn,
    pub execute: ExecuteFn,
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, CommandMeta>,
    handlers: Vec<HandlerRegistration>,
}

impl CommandRegistry {
    pub fn with_builtins() -> Self {
        let mut reg = Self::default();
        reg.register_command(CommandMeta {
            id: commands::FONT_SIZE_INCREASE,
            title: "Increase font size",
            icon: "font-size-up",
            default_keybinding: Some("Ctrl+="),
            aliases: &["bigger text", "font bigger"],
        });
        reg.register_command(CommandMeta {
            id: commands::FONT_SIZE_DECREASE,
            title: "Decrease font size",
            icon: "font-size-down",
            default_keybinding: Some("Ctrl+-"),
            aliases: &["smaller text"],
        });
        reg
    }

    pub fn register_command(&mut self, meta: CommandMeta) {
        self.commands.insert(meta.id.to_string(), meta);
    }

    pub fn register_handler(&mut self, handler: HandlerRegistration) {
        self.handlers.push(handler);
    }

    pub fn commands(&self) -> impl Iterator<Item = &CommandMeta> {
        self.commands.values()
    }

    pub fn command_meta(&self, id: &str) -> Option<&CommandMeta> {
        self.commands.get(id)
    }

    pub fn handlers_for(&self, command_id: &str) -> Vec<&HandlerRegistration> {
        self.handlers
            .iter()
            .filter(|h| h.command_id == command_id)
            .collect()
    }

    pub fn resolve_handler(
        &self,
        command_id: &str,
        active_backend_id: &str,
    ) -> Option<&HandlerRegistration> {
        self.handlers
            .iter()
            .find(|h| h.command_id == command_id && h.backend_id == active_backend_id)
    }

    pub fn enablement(&self, command_id: &str, doc: &DocumentSession) -> Enablement {
        let backend_id = doc.backend.backend_id();
        match self.resolve_handler(command_id, backend_id) {
            Some(h) => (h.enable)(doc),
            None => Enablement::Disabled {
                reason: "No handler for this document type".into(),
            },
        }
    }

    pub fn enablement_ctx(&self, command_id: &str, ctx: &HostContext) -> Enablement {
        if self.resolve_handler(command_id, &ctx.backend_id).is_some() {
            // Handlers need full session for enable; palette uses snapshot approximation.
            if ctx.capabilities.font_size && ctx.selection.has_text_selection {
                Enablement::Enabled
            } else if !ctx.capabilities.font_size {
                Enablement::Disabled {
                    reason: "Format does not support font size".into(),
                }
            } else {
                Enablement::Disabled {
                    reason: "Select text first".into(),
                }
            }
        } else {
            Enablement::Disabled {
                reason: "No handler for this document type".into(),
            }
        }
    }

    pub fn execute(
        &self,
        command_id: &str,
        doc: &mut DocumentSession,
    ) -> Result<String, String> {
        let backend_id = doc.backend.backend_id().to_string();
        let handler = self
            .resolve_handler(command_id, &backend_id)
            .ok_or_else(|| "No handler for active document".to_string())?;
        let en = (handler.enable)(doc);
        if !en.is_enabled() {
            return Err(en
                .disabled_reason()
                .unwrap_or("Command disabled")
                .to_string());
        }
        (handler.execute)(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

    fn sample_doc(backend_id: &str, has_text: bool) -> DocumentSession {
        struct StubBackend {
            id: String,
            has_text: bool,
        }
        impl crate::document::DocumentBackend for StubBackend {
            fn backend_id(&self) -> &str {
                &self.id
            }
            fn doc_type(&self) -> &str {
                "text"
            }
            fn display_name(&self) -> &str {
                "stub"
            }
            fn mode(&self) -> Mode {
                Mode::Text
            }
            fn capabilities(&self) -> Capabilities {
                Capabilities {
                    font_size: self.has_text,
                }
            }
            fn selection(&self) -> SelectionSnapshot {
                SelectionSnapshot {
                    has_text_selection: self.has_text,
                    text_length: if self.has_text { 3 } else { 0 },
                    font_size_pt: Some(12.0),
                    domains: vec![],
                }
            }
            fn increase_font_size(&mut self) -> Result<f32, String> {
                Ok(14.0)
            }
            fn decrease_font_size(&mut self) -> Result<f32, String> {
                Ok(10.0)
            }
            fn status_line(&self) -> String {
                String::new()
            }
            fn set_demo_text_selection(&mut self, selected: bool) {
                let _ = selected;
            }
        }
        DocumentSession::new(
            "stub.txt",
            Box::new(StubBackend {
                id: backend_id.to_string(),
                has_text,
            }),
        )
    }

    #[test]
    fn builtin_commands_registered() {
        let reg = CommandRegistry::with_builtins();
        assert!(reg.command_meta(commands::FONT_SIZE_INCREASE).is_some());
    }

    #[test]
    fn missing_handler_disables() {
        let reg = CommandRegistry::with_builtins();
        let doc = sample_doc("unknown", false);
        let en = reg.enablement(commands::FONT_SIZE_INCREASE, &doc);
        assert!(!en.is_enabled());
    }
}
