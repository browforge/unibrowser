use std::collections::HashMap;

use unibrowser_mod_api::{control_ids, CapabilityState, WorkContext};

use super::context::ResolutionContext;
use super::kind_host::KindHostCapabilityMatrix;
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
    pub kind_host: KindHostCapabilityMatrix,
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
        // These commands are handled by the document/workshop webview rather
        // than the Rust document backend. Registry absence must not overwrite
        // their declared capability with "No handler for this document type".
        if frontend_owned_command(command_id) {
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
            // A missing Rust backend registration is not a runtime disabled
            // state for structured controls. Many are intentionally executed
            // by the webview command router. Preserve the declared capability;
            // concrete backend enablement reasons may still disable controls.
            if reason == "No handler for this document type" {
                return cap;
            }
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

fn frontend_owned_command(command_id: &str) -> bool {
    command_id.starts_with("unibrowser.project.")
        || command_id.starts_with("unibrowser.cad.")
        || command_id.starts_with("unibrowser.image.")
        || command_id.starts_with("unibrowser.pdf.")
        || command_id.starts_with("unibrowser.browse.")
        || command_id.starts_with("unibrowser.settings.")
        || command_id.starts_with("unibrowser.mods.")
        || command_id.starts_with("unibrowser.semantic.")
        || command_id.starts_with("unibrowser.text.")
        || command_id.starts_with("unibrowser.list.")
}

fn default_capability(control_id: &str, ctx: &ResolutionContext) -> CapabilityState {
    match ctx.work {
        WorkContext::PlainText => plain_text_capability(control_id),
        WorkContext::Markdown => markdown_capability(control_id, ctx.markdown_inline_html),
        WorkContext::RichDocument => {
            if control_id == control_ids::SEMANTIC_BLOCK_STYLE {
                return CapabilityState::Emulated {
                    provider_id: "unibrowser.docx.named-style".into(),
                    note: "Applies Word named styles (OOXML), not literal point sizes.".into(),
                };
            }
            CapabilityState::Supported
        }
        WorkContext::CodeEditor => code_capability(control_id),
        WorkContext::Pdf => pdf_capability(control_id),
        WorkContext::Image => image_capability(control_id, ctx),
        WorkContext::Audio => media_capability(control_id),
        WorkContext::Cad3d => cad_capability(control_id),
        WorkContext::Project => project_capability(control_id),
        WorkContext::Start => start_capability(control_id),
        WorkContext::Browse => browse_capability(control_id),
        WorkContext::Settings => settings_capability(control_id),
        WorkContext::Mods => mods_capability(control_id),
        WorkContext::Web => CapabilityState::Unsupported {
            reason: "Open a document to use formatting commands.".into(),
        },
    }
}

fn media_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.media.") {
        return CapabilityState::Supported;
    }
    CapabilityState::Unsupported {
        reason: "This command is not available in the media viewer.".into(),
    }
}

fn project_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.project.") {
        return CapabilityState::Supported;
    }
    CapabilityState::Unsupported {
        reason: "This command is not available in the project workshop.".into(),
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

fn mods_capability(control_id: &str) -> CapabilityState {
    if control_id.starts_with("unibrowser.control.mods.") {
        return CapabilityState::Supported;
    }
    CapabilityState::Unsupported {
        reason: "This command is not available in the Mod portal.".into(),
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

fn block_heading_capability(ctx: &ResolutionContext, level: u8) -> CapabilityState {
    match ctx.work {
        WorkContext::PlainText => CapabilityState::Unsupported {
            reason: "Plain text has no heading structure. Save as Markdown or use a rich document.".into(),
        },
        WorkContext::Markdown => CapabilityState::Supported,
        WorkContext::RichDocument => CapabilityState::Emulated {
            provider_id: "unibrowser.docx.named-style".into(),
            note: format!(
                "Applies Word named style Heading {} (OOXML), not literal point size.",
                level
            ),
        },
        _ => CapabilityState::Unsupported {
            reason: "Block headings are not available in this context.".into(),
        },
    }
}

fn plain_text_capability(control_id: &str) -> CapabilityState {
    match control_id {
        control_ids::SEMANTIC_BLOCK_STYLE => CapabilityState::Supported,
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
        control_ids::SEMANTIC_BLOCK_STYLE
        | control_ids::TEXT_CODE_INLINE
        | control_ids::TEXT_LINK
        | control_ids::TEXT_LIST
        | control_ids::TEXT_LIST_NUMBERED
        | control_ids::TEXT_INSERT_IMAGE
        | control_ids::TEXT_CLEAR
        | control_ids::TEXT_COMPACT_OVERFLOW => CapabilityState::Supported,
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
            note: "Approximated with inline formatting or reduced text size.".into(),
        },
        control_ids::TEXT_SUPERSCRIPT => CapabilityState::Emulated {
            provider_id: "unibrowser.markdown.sup".into(),
            note: "Approximated with inline formatting or raised text size.".into(),
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
    match control_id {
        control_ids::PDF_HIGHLIGHT
        | control_ids::PDF_PAGE
        | control_ids::PDF_VIEW_ZOOM
        | control_ids::PDF_EXPORT => CapabilityState::Supported,
        control_ids::PDF_SELECT => CapabilityState::Unsupported {
            reason: "Annotation selection needs a PDF annotation picking provider.".into(),
        },
        control_ids::PDF_HAND => CapabilityState::Unsupported {
            reason: "Hand tool needs the PDF pan provider.".into(),
        },
        control_ids::PDF_UNDERLINE | control_ids::PDF_STRIKETHROUGH => {
            CapabilityState::Unsupported {
                reason: "Text markup needs PDF text-range coordinate support.".into(),
            }
        }
        control_ids::PDF_INK => CapabilityState::Unsupported {
            reason: "Ink needs a PDF stroke annotation provider.".into(),
        },
        control_ids::PDF_NOTE => CapabilityState::Unsupported {
            reason: "Notes need a PDF point annotation provider.".into(),
        },
        control_ids::PDF_AREA => CapabilityState::Unsupported {
            reason: "Area markup needs a PDF shape annotation provider.".into(),
        },
        control_ids::PDF_TEXT_BOX => CapabilityState::Unsupported {
            reason: "Text box needs a PDF free-text annotation provider.".into(),
        },
        control_ids::PDF_STAMP => CapabilityState::Unsupported {
            reason: "Stamp needs a PDF stamp annotation provider.".into(),
        },
        control_ids::PDF_ANNOTATION_PREVIOUS | control_ids::PDF_ANNOTATION_NEXT => {
            CapabilityState::Unsupported {
                reason: "Annotation navigation needs selectable annotation identities.".into(),
            }
        }
        control_ids::PDF_STROKE_COLOUR
        | control_ids::PDF_FILL_COLOUR
        | control_ids::PDF_STROKE_WIDTH => CapabilityState::Unsupported {
            reason: "Select an editable annotation to change its appearance.".into(),
        },
        id if id.starts_with("unibrowser.control.text.") => CapabilityState::Unsupported {
            reason: "PDF pages are not rich-text documents.".into(),
        },
        "unibrowser.control.proofing" => CapabilityState::Unsupported {
            reason: "Spell check applies to editable text, not PDF page content.".into(),
        },
        _ => CapabilityState::Supported,
    }
}

fn image_capability(control_id: &str, ctx: &ResolutionContext) -> CapabilityState {
    let vector_image = ctx
        .doc_path
        .as_deref()
        .map(|path| {
            path.rsplit('.')
                .next()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
        })
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
        "unibrowser.control.image.zoomOut" | "unibrowser.control.image.zoomIn" => {
            CapabilityState::Supported
        }
        "unibrowser.control.image.fit"
        | "unibrowser.control.image.actualSize"
        | "unibrowser.control.image.crop"
        | "unibrowser.control.image.rotate"
        | "unibrowser.control.image.flipHorizontal"
        | "unibrowser.control.image.flipVertical"
        | "unibrowser.control.image.histogram"
        | "unibrowser.control.image.eyedropper"
        | "unibrowser.control.image.copy"
        | "unibrowser.control.image.ocr"
            if !vector_image =>
        {
            CapabilityState::Supported
        }
        "unibrowser.control.image.ocr" => CapabilityState::Unsupported {
            reason: "OCR applies to raster images, not SVG.".into(),
        },
        "unibrowser.control.image.fit" => CapabilityState::Unsupported {
            reason: "Fit-to-window needs an SVG view provider.".into(),
        },
        "unibrowser.control.image.actualSize" => CapabilityState::Unsupported {
            reason: "Pixel zoom needs an SVG view provider.".into(),
        },
        "unibrowser.control.image.crop" => CapabilityState::Unsupported {
            reason: "Crop needs a raster image edit provider.".into(),
        },
        "unibrowser.control.image.rotate" => CapabilityState::Unsupported {
            reason: "Rotate needs an SVG transform provider.".into(),
        },
        "unibrowser.control.image.resize" => CapabilityState::Unsupported {
            reason: "Resize needs an image resampling provider.".into(),
        },
        "unibrowser.control.image.exposure"
        | "unibrowser.control.image.contrast"
        | "unibrowser.control.image.colour"
        | "unibrowser.control.image.curves" => CapabilityState::Unsupported {
            reason: "Image adjustments need a non-destructive raster adjustment provider.".into(),
        },
        id if id.starts_with("unibrowser.control.image.") => CapabilityState::Unsupported {
            reason: "No image provider currently implements this control.".into(),
        },
        id if id.starts_with("unibrowser.control.text.") => CapabilityState::Unsupported {
            reason: "Adjust image pixels with the image tools, not text formatting.".into(),
        },
        _ => CapabilityState::Supported,
    }
}

fn cad_capability(control_id: &str) -> CapabilityState {
    match control_id {
        "unibrowser.control.cad.select"
        | "unibrowser.control.cad.orientation"
        | "unibrowser.control.cad.pan"
        | "unibrowser.control.cad.orbit"
        | "unibrowser.control.cad.fitView"
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
        "unibrowser.control.cad.section" => CapabilityState::Unsupported {
            reason: "Section view needs a full CAD geometry engine.".into(),
        },
        "unibrowser.control.cad.appearance" => CapabilityState::Unsupported {
            reason: "Appearance controls need a CAD material provider.".into(),
        },
        _ => CapabilityState::Unsupported {
            reason: "This command is not available for CAD documents.".into(),
        },
    }
}
