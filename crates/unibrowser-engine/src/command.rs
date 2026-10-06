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
        reg.register_command(CommandMeta {
            id: commands::SCRIPT_DEMO_HELLO,
            title: "Run demo Rhai script (hello)",
            icon: "extension",
            default_keybinding: None,
            aliases: &["script hello", "rhai demo"],
        });
        reg.register_command(CommandMeta {
            id: commands::PAGE_ZOOM_INCREASE,
            title: "Increase page zoom",
            icon: "zoom-in",
            default_keybinding: Some("Ctrl+Shift+="),
            aliases: &["zoom in", "bigger page"],
        });
        reg.register_command(CommandMeta {
            id: commands::PAGE_ZOOM_DECREASE,
            title: "Decrease page zoom",
            icon: "zoom-out",
            default_keybinding: Some("Ctrl+Shift+-"),
            aliases: &["zoom out", "smaller page"],
        });
        reg.register_command(CommandMeta {
            id: commands::PAGE_ZOOM_RESET_ALL,
            title: "Reset zoom for all sites",
            icon: "zoom-reset",
            default_keybinding: None,
            aliases: &["zoom reset all", "clear site zoom"],
        });
        reg.register_command(CommandMeta {
            id: commands::PAGE_ZOOM_FIT,
            title: "Zoom to fit",
            icon: "zoom-fit",
            default_keybinding: Some("Ctrl+0"),
            aliases: &["fit page", "fit view", "zoom fit"],
        });
        reg.register_command(CommandMeta {
            id: commands::FIND_IN_DOCUMENT,
            title: "Find in document",
            icon: "search",
            default_keybinding: Some("Ctrl+F"),
            aliases: &["find", "search document"],
        });
        reg.register_command(CommandMeta {
            id: commands::PROOFING_TOGGLE,
            title: "Check",
            icon: "spellcheck",
            default_keybinding: None,
            aliases: &["spellcheck", "proofing", "check spelling"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_SAVE,
            title: "Save",
            icon: "save",
            default_keybinding: Some("Ctrl+S"),
            aliases: &["save file", "write file"],
        });
        reg.register_command(CommandMeta {
            id: commands::FORMAT_DOCUMENT,
            title: "Format",
            icon: "format",
            default_keybinding: None,
            aliases: &["format document", "prettify"],
        });
        reg.register_command(CommandMeta {
            id: commands::PROOFING_NEXT,
            title: "Next proofing problem",
            icon: "arrow-down",
            default_keybinding: Some("F8"),
            aliases: &["next spelling error"],
        });
        reg.register_command(CommandMeta {
            id: commands::PROOFING_PREVIOUS,
            title: "Previous proofing problem",
            icon: "arrow-up",
            default_keybinding: Some("Shift+F8"),
            aliases: &["previous spelling error"],
        });
        reg.register_command(CommandMeta {
            id: commands::RESTORE_AUTOSAVE,
            title: "Restore from autosave…",
            icon: "history",
            default_keybinding: None,
            aliases: &["recover autosave", "autosave slot"],
        });
        reg.register_command(CommandMeta {
            id: commands::TOGGLE_PROBLEMS_PANEL,
            title: "Toggle problems panel",
            icon: "warning",
            default_keybinding: None,
            aliases: &["diagnostics panel", "problems list"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_BOLD,
            title: "Bold",
            icon: "bold",
            default_keybinding: Some("Ctrl+B"),
            aliases: &["strong", "emphasis bold"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_ITALIC,
            title: "Italic",
            icon: "italic",
            default_keybinding: Some("Ctrl+I"),
            aliases: &["emphasis italic"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_UNDERLINE,
            title: "Underline",
            icon: "underline",
            default_keybinding: Some("Ctrl+U"),
            aliases: &["underline text"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_STRIKETHROUGH,
            title: "Strikethrough",
            icon: "strikethrough",
            default_keybinding: None,
            aliases: &["strike"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_SUBSCRIPT,
            title: "Subscript",
            icon: "subscript",
            default_keybinding: None,
            aliases: &["sub"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_SUPERSCRIPT,
            title: "Superscript",
            icon: "superscript",
            default_keybinding: None,
            aliases: &["sup"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_COLOUR,
            title: "Text colour",
            icon: "palette",
            default_keybinding: None,
            aliases: &["text color", "font colour"],
        });
        reg.register_command(CommandMeta {
            id: commands::TEXT_FONT_FAMILY,
            title: "Font",
            icon: "type",
            default_keybinding: None,
            aliases: &["font family", "typeface"],
        });
        reg.register_command(CommandMeta {
            id: commands::EDITOR_FONT_FAMILY,
            title: "Editor font",
            icon: "type",
            default_keybinding: None,
            aliases: &["monospace font", "code font"],
        });
        reg.register_command(CommandMeta {
            id: commands::EDITOR_FONT_SIZE,
            title: "Editor font size",
            icon: "type",
            default_keybinding: None,
            aliases: &["code font size"],
        });
        reg.register_command(CommandMeta {
            id: commands::DOCUMENT_UNDO,
            title: "Undo",
            icon: "undo",
            default_keybinding: Some("Ctrl+Alt+Z"),
            aliases: &["revert change"],
        });
        reg.register_command(CommandMeta {
            id: commands::DOCUMENT_REDO,
            title: "Redo",
            icon: "redo",
            default_keybinding: Some("Ctrl+Alt+Y"),
            aliases: &["repeat change"],
        });
        reg.register_command(CommandMeta {
            id: commands::EXPORT_PRINT_PDF,
            title: "Export",
            icon: "export",
            default_keybinding: None,
            aliases: &["print pdf", "save as pdf", "export pdf"],
        });
        reg.register_command(CommandMeta {
            id: commands::PRINT_SELECTION,
            title: "Print selection",
            icon: "export",
            default_keybinding: None,
            aliases: &["print selected text", "print highlight"],
        });
        reg.register_command(CommandMeta {
            id: commands::TOGGLE_WEB_DEVTOOLS,
            title: "Toggle developer tools",
            icon: "tools",
            default_keybinding: Some("F12"),
            aliases: &["devtools", "web inspector", "inspect element"],
        });
        reg.register_command(CommandMeta {
            id: commands::TOGGLE_TERMINAL_PANEL,
            title: "Toggle terminal panel",
            icon: "terminal",
            default_keybinding: Some("Ctrl+`"),
            aliases: &["integrated terminal", "shell", "pty"],
        });
        reg.register_command(CommandMeta {
            id: commands::COPY_TEXT_FROM_IMAGE,
            title: "Copy text from image (OCR)",
            icon: "ocr",
            default_keybinding: None,
            aliases: &["ocr", "extract text", "tesseract"],
        });
        reg.register_command(CommandMeta {
            id: commands::NEW_WINDOW,
            title: "New window",
            icon: "window",
            default_keybinding: None,
            aliases: &["another window", "second window"],
        });
        reg.register_command(CommandMeta {
            id: commands::SHOW_KEYBOARD_SHORTCUTS,
            title: "Show keyboard shortcuts",
            icon: "keyboard",
            default_keybinding: None,
            aliases: &["key bindings", "cheat sheet", "shortcuts"],
        });
        reg.register_command(CommandMeta {
            id: commands::TOGGLE_READER_MODE,
            title: "Toggle reader mode",
            icon: "reader",
            default_keybinding: None,
            aliases: &["readability", "declutter", "article mode"],
        });
        reg.register_command(CommandMeta {
            id: commands::CAPTURE_VISIBLE_TAB,
            title: "Capture visible tab",
            icon: "camera",
            default_keybinding: None,
            aliases: &["screenshot", "snapshot", "save png"],
        });
        reg.register_command(CommandMeta {
            id: commands::COMPARE_IMAGES,
            title: "Compare images (experimental)",
            icon: "compare",
            default_keybinding: None,
            aliases: &["image diff", "side by side images"],
        });
        reg.register_command(CommandMeta {
            id: commands::COMPARE_TEXT,
            title: "Compare text files (experimental)",
            icon: "compare",
            default_keybinding: None,
            aliases: &["text diff", "diff two files", "unified diff"],
        });
        reg.register_command(CommandMeta {
            id: commands::GIT_COMMIT,
            title: "Git commit (not available)",
            icon: "git",
            default_keybinding: None,
            aliases: &["scm commit"],
        });
        reg.register_command(CommandMeta {
            id: commands::GIT_PUSH,
            title: "Git push (not available)",
            icon: "git",
            default_keybinding: None,
            aliases: &["scm push"],
        });
        reg.register_command(CommandMeta {
            id: commands::GIT_SYNC,
            title: "Git sync (not available)",
            icon: "git",
            default_keybinding: None,
            aliases: &["pull", "fetch"],
        });
        reg.register_command(CommandMeta {
            id: commands::GO_TO_TAB,
            title: "Go to tab",
            icon: "tabs",
            default_keybinding: Some("Ctrl+Shift+A"),
            aliases: &["tab search", "find tab", "switch tab"],
        });
        reg.register_command(CommandMeta {
            id: commands::DUPLICATE_TAB,
            title: "Duplicate tab",
            icon: "tabs",
            default_keybinding: Some("Ctrl+Shift+K"),
            aliases: &["clone tab", "copy tab"],
        });
        reg.register_command(CommandMeta {
            id: commands::CLOSE_OTHER_TABS,
            title: "Close other tabs",
            icon: "tabs",
            default_keybinding: None,
            aliases: &["close others"],
        });
        reg.register_command(CommandMeta {
            id: commands::CLOSE_TABS_TO_RIGHT,
            title: "Close tabs to the right",
            icon: "tabs",
            default_keybinding: None,
            aliases: &["close right tabs"],
        });
        reg.register_command(CommandMeta {
            id: commands::REOPEN_CLOSED_TAB,
            title: "Reopen closed tab",
            icon: "tabs",
            default_keybinding: Some("Ctrl+Shift+T"),
            aliases: &["restore tab", "undo close tab"],
        });
        reg.register_command(CommandMeta {
            id: commands::TOGGLE_TAB_MUTE,
            title: "Mute tab",
            icon: "media",
            default_keybinding: None,
            aliases: &["mute audio", "silence tab"],
        });
        reg.register_command(CommandMeta {
            id: commands::PICTURE_IN_PICTURE,
            title: "Picture-in-picture",
            icon: "pip",
            default_keybinding: None,
            aliases: &["pip", "pop out video", "pop out player"],
        });
        reg.register_command(CommandMeta {
            id: commands::CONTEXT_HELP,
            title: "What's this (context help)",
            icon: "help",
            default_keybinding: Some("F1"),
            aliases: &["help", "context help", "documentation"],
        });
        reg.register_command(CommandMeta {
            id: commands::WORKSPACE_QUICK_OPEN,
            title: "Go to file in workspace",
            icon: "search",
            default_keybinding: Some("Ctrl+P"),
            aliases: &["quick open", "workspace file", "ctrl p"],
        });
        reg.register_command(CommandMeta {
            id: commands::FIND_FILES_DEEP,
            title: "Find files by name (deep search)",
            icon: "search",
            default_keybinding: Some("Ctrl+Shift+F"),
            aliases: &["global find", "filename search"],
        });
        reg.register_command(CommandMeta {
            id: commands::MIDI_RENDER_AUDIO,
            title: "Render MIDI to audio (stub)",
            icon: "export",
            default_keybinding: None,
            aliases: &["midi wav", "fluidsynth", "synthesize midi"],
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

    pub fn sorted_command_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.commands.keys().cloned().collect();
        ids.sort();
        ids
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
        if command_id == commands::FORMAT_DOCUMENT {
            return Enablement::Disabled {
                reason: "Format document is unavailable for plain text.".into(),
            };
        }
        let backend_id = doc.backend.backend_id();
        match self.resolve_handler(command_id, backend_id) {
            Some(h) => (h.enable)(doc),
            None => Enablement::Disabled {
                reason: "No handler for this document type".into(),
            },
        }
    }

    pub fn enablement_ctx(&self, command_id: &str, ctx: &HostContext) -> Enablement {
        if command_id == commands::GO_TO_TAB
            || command_id == commands::WORKSPACE_QUICK_OPEN
            || command_id == commands::FIND_FILES_DEEP
            || command_id == commands::DUPLICATE_TAB
            || command_id == commands::CLOSE_OTHER_TABS
            || command_id == commands::CLOSE_TABS_TO_RIGHT
            || command_id == commands::REOPEN_CLOSED_TAB
            || command_id == commands::TOGGLE_TAB_MUTE
            || command_id == commands::PAGE_ZOOM_RESET_ALL
        {
            return Enablement::Enabled;
        }
        if self.resolve_handler(command_id, &ctx.backend_id).is_none() {
            return Enablement::Disabled {
                reason: "No handler for this document type".into(),
            };
        }
        if command_id == commands::TEXT_SAVE {
            return Enablement::Enabled;
        }
        if command_id == commands::FIND_IN_DOCUMENT {
            if self.resolve_handler(command_id, &ctx.backend_id).is_some() {
                return Enablement::Enabled;
            }
            return Enablement::Disabled {
                reason: "Find is not available in this view".into(),
            };
        }
        if command_id == commands::PAGE_ZOOM_INCREASE
            || command_id == commands::PAGE_ZOOM_DECREASE
            || command_id == commands::PAGE_ZOOM_FIT
        {
            if self.resolve_handler(command_id, &ctx.backend_id).is_some() {
                return Enablement::Enabled;
            }
            return Enablement::Disabled {
                reason: "Page zoom is not available in this view".into(),
            };
        }
        if command_id == commands::DOCUMENT_UNDO || command_id == commands::DOCUMENT_REDO {
            if self.resolve_handler(command_id, &ctx.backend_id).is_some() {
                return Enablement::Enabled;
            }
            return Enablement::Disabled {
                reason: "Undo/redo is not available for this document type".into(),
            };
        }
        if command_id == commands::FORMAT_DOCUMENT {
            return Enablement::Disabled {
                reason: "Format document is unavailable for plain text.".into(),
            };
        }
        if command_id == commands::PROOFING_TOGGLE {
            return Enablement::Enabled;
        }
        if command_id == commands::EXPORT_PRINT_PDF {
            if self.resolve_handler(command_id, &ctx.backend_id).is_some() {
                return Enablement::Enabled;
            }
            return Enablement::Disabled {
                reason: "PDF export needs printable HTML (Markdown or DOCX preview)".into(),
            };
        }
        if command_id == commands::PRINT_SELECTION {
            if self.resolve_handler(command_id, &ctx.backend_id).is_some() {
                return Enablement::Enabled;
            }
            return Enablement::Disabled {
                reason: "Print selection needs Monaco or DOCX preview".into(),
            };
        }
        // Handlers need full session for precise enable; palette uses snapshot approximation.
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
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
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
