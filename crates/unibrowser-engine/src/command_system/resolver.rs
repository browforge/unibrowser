use std::collections::HashMap;

use unibrowser_mod_api::{control_ids, CapabilityState, WorkContext};

use super::context::ResolutionContext;
use super::provider::ProviderRegistry;
use super::save_guard::save_commands_always_supported;
use crate::command::CommandRegistry;
use crate::document::DocumentSession;

#[derive(Debug, Clone, Default)]
pub struct CapabilityResolver {
    pub providers: ProviderRegistry,
    pub user_provider_overrides: HashMap<String, String>,
    pub hidden_controls: HashMap<String, bool>,
    pub layout_revision: u64,
}

impl CapabilityResolver {
    pub fn resolve_control(
        &self,
        control_id: &str,
        command_id: &str,
        ctx: &ResolutionContext,
    ) -> CapabilityState {
        if self
            .hidden_controls
            .get(control_id)
            .copied()
            .unwrap_or(false)
        {
            return CapabilityState::HiddenByUser;
        }
        if save_commands_always_supported(command_id) {
            return CapabilityState::Supported;
        }
        let base = default_capability(control_id, ctx);
        let with_provider = self.providers.apply_override(
            control_id,
            command_id,
            base,
            ctx,
            &self.user_provider_overrides,
        );
        self.apply_hidden_policy(control_id, with_provider)
    }

    fn apply_hidden_policy(&self, control_id: &str, cap: CapabilityState) -> CapabilityState {
        if matches!(cap, CapabilityState::HiddenByUser) {
            return cap;
        }
        if self
            .hidden_controls
            .get(control_id)
            .copied()
            .unwrap_or(false)
        {
            CapabilityState::HiddenByUser
        } else {
            cap
        }
    }

    pub fn resolve_with_registry(
        &self,
        registry: &CommandRegistry,
        doc: Option<&DocumentSession>,
        host: &unibrowser_mod_api::HostContext,
        control_id: &str,
        command_id: &str,
        ctx: &ResolutionContext,
    ) -> CapabilityState {
        let cap = self.resolve_control(control_id, command_id, ctx);
        if !cap.is_actionable() {
            return cap;
        }
        let en = if let Some(doc) = doc {
            registry.enablement(command_id, doc)
        } else {
            registry.enablement_ctx(command_id, host)
        };
        if en.is_enabled() {
            return cap;
        }
        if let Some(reason) = en.disabled_reason() {
            return CapabilityState::Unsupported {
                reason: reason.to_string(),
            };
        }
        cap
    }

    pub fn accept_provider_response(
        &mut self,
        context_revision: u64,
        expected_revision: u64,
        cap: CapabilityState,
    ) -> Option<CapabilityState> {
        if context_revision != expected_revision {
            return None;
        }
        Some(cap)
    }
}

fn default_capability(control_id: &str, ctx: &ResolutionContext) -> CapabilityState {
    match ctx.work {
        WorkContext::PlainText => plain_text_capability(control_id),
        WorkContext::Markdown => markdown_capability(control_id, ctx.markdown_inline_html),
        WorkContext::RichDocument => CapabilityState::Supported,
        WorkContext::CodeEditor => code_capability(control_id),
        WorkContext::Pdf => pdf_capability(control_id),
        WorkContext::Image => image_capability(control_id, ctx),
        WorkContext::Cad3d => cad_capability(control_id),
        WorkContext::Start => start_capability(control_id),
        WorkContext::Browse => browse_capability(control_id),
        WorkContext::Settings => settings_capability(control_id),
        WorkContext::Web => CapabilityState::Unsupported {
            reason: "Open a document to use formatting commands.".into(),
        },
    }
}

fn settings_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.settings.") {
        return CapabilityState::Supported;
    }
    CapabilityState::Unsupported {
        reason: "This command is not available in Settings.".into(),
    }
}

fn browse_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.browse.") {
        return CapabilityState::Supported;
    }
    CapabilityState::Unsupported {
        reason: "This command is not available in the file browser.".into(),
    }
}

fn start_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.start.") {
        CapabilityState::Supported
    } else {
        CapabilityState::Unsupported {
            reason: "Start commands are available on the new tab page.".into(),
        }
    }
}

fn plain_text_capability(control_id: &str) -> CapabilityState {
    match control_id {
        control_ids::TEXT_FONT | control_ids::TEXT_COLOUR => CapabilityState::Unsupported {
            reason: "Plain text has no font or colour semantics. Save as Markdown or use a rich document to preserve formatting.".into(),
        },
        control_ids::TEXT_STRIKETHROUGH | control_ids::TEXT_UNDERLINE | control_ids::TEXT_SUBSCRIPT
        | control_ids::TEXT_SUPERSCRIPT => CapabilityState::Unsupported {
            reason: "Plain text has no rich formatting. Save as Markdown or use a rich document to preserve it.".into(),
        },
        control_ids::TEXT_BOLD | control_ids::TEXT_ITALIC => CapabilityState::Unsupported {
            reason: "Plain text has no emphasis styles. Save as Markdown or use a rich document.".into(),
        },
        control_ids::TEXT_SIZE => CapabilityState::Supported,
        _ => CapabilityState::Supported,
    }
}

fn markdown_capability(control_id: &str, inline_html: bool) -> CapabilityState {
    match control_id {
        control_ids::TEXT_FONT => CapabilityState::Unsupported {
            reason: "Portable Markdown has no font family semantic.".into(),
        },
        control_ids::TEXT_BOLD | control_ids::TEXT_ITALIC | control_ids::TEXT_STRIKETHROUGH => {
            CapabilityState::Supported
        }
        control_ids::TEXT_UNDERLINE => {
            if inline_html {
                CapabilityState::Emulated {
                    provider_id: "unibrowser.markdown.inline-html".into(),
                    note: "Approximated with inline HTML when the document allows it.".into(),
                }
            } else {
                CapabilityState::Unsupported {
                    reason: "Underline is not part of portable Markdown.".into(),
                }
            }
        }
        control_ids::TEXT_SUBSCRIPT => CapabilityState::Emulated {
            provider_id: "unibrowser.markdown.sub".into(),
            note: "Approximated with <sub> or a font-scaling fallback.".into(),
        },
        control_ids::TEXT_SUPERSCRIPT => CapabilityState::Emulated {
            provider_id: "unibrowser.markdown.sup".into(),
            note: "Approximated with <sup> or a font-scaling fallback.".into(),
        },
        control_ids::TEXT_COLOUR => {
            if inline_html {
                CapabilityState::Emulated {
                    provider_id: "unibrowser.markdown.inline-html".into(),
                    note: "Approximated with inline HTML colour spans.".into(),
                }
            } else {
                CapabilityState::Unsupported {
                    reason: "Text colour is not part of portable Markdown.".into(),
                }
            }
        }
        control_ids::TEXT_SIZE => CapabilityState::Supported,
        _ => CapabilityState::Supported,
    }
}

fn code_capability(control_id: &str) -> CapabilityState {
    match control_id {
        control_ids::EDITOR_FONT | control_ids::EDITOR_SIZE => CapabilityState::Supported,
        id if id.starts_with("unibrowser.control.text.") => CapabilityState::Unsupported {
            reason: "Code files use editor appearance settings, not document formatting.".into(),
        },
        _ => CapabilityState::Supported,
    }
}

fn pdf_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.pdf.") {
        CapabilityState::Supported
    } else if control_id.starts_with("unibrowser.control.text.") {
        CapabilityState::Unsupported {
            reason: "PDF pages are not rich-text documents.".into(),
        }
    } else {
        CapabilityState::Supported
    }
}

fn image_capability(control_id: &str, ctx: &ResolutionContext) -> CapabilityState {
    let vector_image = ctx
        .doc_path
        .as_deref()
        .map(|path| path.rsplit('.').next().is_some_and(|ext| ext.eq_ignore_ascii_case("svg")))
        .unwrap_or(false);
    match control_id {
        "unibrowser.control.find" => CapabilityState::Unsupported {
            reason: "Find is not available for images.".into(),
        },
        "unibrowser.control.proofing" => CapabilityState::Unsupported {
            reason: "Spell check applies to text, not images.".into(),
        },
        "unibrowser.control.image.curves" if vector_image => CapabilityState::Unsupported {
            reason: "Curves adjusts raster pixels, not SVG vector paths.".into(),
        },
        id if id.starts_with("unibrowser.control.image.") => CapabilityState::Supported,
        id if id.starts_with("unibrowser.control.text.") => CapabilityState::Unsupported {
            reason: "Adjust image pixels with the image tools, not text formatting.".into(),
        },
        _ => CapabilityState::Supported,
    }
}

fn cad_capability(control_id: &str) -> CapabilityState {
    match control_id {
        "unibrowser.control.cad.fitView"
        | "unibrowser.control.cad.zoomOut"
        | "unibrowser.control.cad.zoomIn" => CapabilityState::Supported,
        "unibrowser.control.cad.find" => CapabilityState::Unsupported {
            reason: "Entity search needs a CAD indexing provider.".into(),
        },
        "unibrowser.control.cad.measure" => CapabilityState::Unsupported {
            reason: "Measure needs a full CAD geometry engine.".into(),
        },
        "unibrowser.control.cad.exportStep" => CapabilityState::Unsupported {
            reason: "STEP export needs a full CAD geometry engine.".into(),
        },
        _ => CapabilityState::Unsupported {
            reason: "This command is not available for CAD documents.".into(),
        },
    }
}
