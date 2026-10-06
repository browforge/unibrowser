use unibrowser_mod_api::{
    CapabilityState, ControlType, DensityVariant, OverflowPriority, SelectionState, WorkContext,
    commands,
    control_ids,
    family_ids,
    group_ids,
    ResolvedControl,
    ResolvedFamily,
    ResolvedGroup,
};

use super::context::ResolutionContext;
use super::resolver::CapabilityResolver;

pub fn core_tab_layout(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> Vec<ResolvedFamily> {
    vec![
        history_family(ctx, resolver),
        text_or_editor_family(ctx, resolver),
        find_family(ctx, resolver),
        proofing_family(ctx, resolver),
        export_family(ctx, resolver),
        pdf_annotation_family(ctx, resolver),
        image_family(ctx, resolver),
    ]
    .into_iter()
    .filter(|f| !f.groups.is_empty())
    .collect()
}

fn history_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let controls = vec![
        control_spec(
            "unibrowser.control.history.undo",
            ControlType::Button,
            commands::DOCUMENT_UNDO,
            "Undo",
            "undo",
            group_ids::TEXT_EMPHASIS,
            family_ids::HISTORY,
            ctx,
            resolver,
        ),
        control_spec(
            "unibrowser.control.history.redo",
            ControlType::Button,
            commands::DOCUMENT_REDO,
            "Redo",
            "redo",
            group_ids::TEXT_EMPHASIS,
            family_ids::HISTORY,
            ctx,
            resolver,
        ),
    ];
    ResolvedFamily {
        family_id: family_ids::HISTORY.into(),
        label: "History".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![ResolvedGroup {
            group_id: "unibrowser.group.history".into(),
            label: "History".into(),
            overflow_priority: OverflowPriority::Primary,
            controls,
        }],
    }
}

fn text_or_editor_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    if ctx.work == WorkContext::CodeEditor {
        return editor_appearance_family(ctx, resolver);
    }
    text_formatting_family(ctx, resolver)
}

fn editor_appearance_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let type_group = ResolvedGroup {
        group_id: group_ids::TEXT_TYPE.into(),
        label: "Type".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![
            control_spec(
                control_ids::EDITOR_FONT,
                ControlType::Dropdown,
                commands::EDITOR_FONT_FAMILY,
                "Font",
                "type",
                group_ids::TEXT_TYPE,
                family_ids::EDITOR_APPEARANCE,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::EDITOR_SIZE,
                ControlType::ValueStepper,
                commands::EDITOR_FONT_SIZE,
                "Size",
                "type",
                group_ids::TEXT_TYPE,
                family_ids::EDITOR_APPEARANCE,
                ctx,
                resolver,
            ),
        ],
    };
    ResolvedFamily {
        family_id: family_ids::EDITOR_APPEARANCE.into(),
        label: "Editor Appearance".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![type_group],
    }
}

fn text_formatting_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let type_group = ResolvedGroup {
        group_id: group_ids::TEXT_TYPE.into(),
        label: "Type".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![
            control_spec(
                control_ids::TEXT_FONT,
                ControlType::Dropdown,
                commands::TEXT_FONT_FAMILY,
                "Font",
                "type",
                group_ids::TEXT_TYPE,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_SIZE,
                ControlType::ValueStepper,
                commands::FONT_SIZE_INCREASE,
                "Size",
                "type",
                group_ids::TEXT_TYPE,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
        ],
    };
    let emphasis = ResolvedGroup {
        group_id: group_ids::TEXT_EMPHASIS.into(),
        label: "Emphasis".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![
            control_spec(
                control_ids::TEXT_BOLD,
                ControlType::Toggle,
                commands::TEXT_BOLD,
                "Bold",
                "bold",
                group_ids::TEXT_EMPHASIS,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_ITALIC,
                ControlType::Toggle,
                commands::TEXT_ITALIC,
                "Italic",
                "italic",
                group_ids::TEXT_EMPHASIS,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_UNDERLINE,
                ControlType::Toggle,
                commands::TEXT_UNDERLINE,
                "Underline",
                "underline",
                group_ids::TEXT_EMPHASIS,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_STRIKETHROUGH,
                ControlType::Toggle,
                commands::TEXT_STRIKETHROUGH,
                "Strikethrough",
                "strikethrough",
                group_ids::TEXT_EMPHASIS,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
        ],
    };
    let baseline = ResolvedGroup {
        group_id: group_ids::TEXT_BASELINE.into(),
        label: "Baseline".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![
            control_spec(
                control_ids::TEXT_SUBSCRIPT,
                ControlType::Toggle,
                commands::TEXT_SUBSCRIPT,
                "Subscript",
                "subscript",
                group_ids::TEXT_BASELINE,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_SUPERSCRIPT,
                ControlType::Toggle,
                commands::TEXT_SUPERSCRIPT,
                "Superscript",
                "superscript",
                group_ids::TEXT_BASELINE,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_COLOUR,
                ControlType::Colour,
                commands::TEXT_COLOUR,
                "Colour",
                "palette",
                group_ids::TEXT_BASELINE,
                family_ids::TEXT_FORMATTING,
                ctx,
                resolver,
            ),
        ],
    };
    ResolvedFamily {
        family_id: family_ids::TEXT_FORMATTING.into(),
        label: "Text Formatting".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![type_group, emphasis, baseline],
    }
}

fn find_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    ResolvedFamily {
        family_id: family_ids::FIND.into(),
        label: "Find".into(),
        overflow_priority: OverflowPriority::Normal,
        groups: vec![ResolvedGroup {
            group_id: "unibrowser.group.find".into(),
            label: "Find".into(),
            overflow_priority: OverflowPriority::Normal,
            controls: vec![control_spec(
                "unibrowser.control.find",
                ControlType::Button,
                commands::FIND_IN_DOCUMENT,
                "Find",
                "search",
                "unibrowser.group.find",
                family_ids::FIND,
                ctx,
                resolver,
            )],
        }],
    }
}

fn proofing_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    ResolvedFamily {
        family_id: family_ids::PROOFING.into(),
        label: "Proofing".into(),
        overflow_priority: OverflowPriority::Low,
        groups: vec![ResolvedGroup {
            group_id: "unibrowser.group.proofing".into(),
            label: "Proofing".into(),
            overflow_priority: OverflowPriority::Low,
            controls: vec![control_spec(
                "unibrowser.control.proofing",
                ControlType::Toggle,
                commands::PROOFING_TOGGLE,
                "Check",
                "spell-check",
                "unibrowser.group.proofing",
                family_ids::PROOFING,
                ctx,
                resolver,
            )],
        }],
    }
}

fn export_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    ResolvedFamily {
        family_id: family_ids::EXPORT.into(),
        label: "Export".into(),
        overflow_priority: OverflowPriority::Low,
        groups: vec![ResolvedGroup {
            group_id: "unibrowser.group.export".into(),
            label: "Export".into(),
            overflow_priority: OverflowPriority::Low,
            controls: vec![control_spec(
                "unibrowser.control.export",
                ControlType::SplitButton,
                commands::EXPORT_PRINT_PDF,
                "Export",
                "download",
                "unibrowser.group.export",
                family_ids::EXPORT,
                ctx,
                resolver,
            )],
        }],
    }
}

fn pdf_annotation_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    if ctx.work != WorkContext::Pdf {
        return ResolvedFamily {
            family_id: family_ids::PDF_ANNOTATION.into(),
            label: "Annotation".into(),
            overflow_priority: OverflowPriority::Normal,
            groups: vec![],
        };
    }
    ResolvedFamily {
        family_id: family_ids::PDF_ANNOTATION.into(),
        label: "Annotation".into(),
        overflow_priority: OverflowPriority::Normal,
        groups: vec![ResolvedGroup {
            group_id: "unibrowser.group.pdf.annotate".into(),
            label: "Annotate".into(),
            overflow_priority: OverflowPriority::Normal,
            controls: vec![
                control_spec(
                    "unibrowser.control.pdf.highlight",
                    ControlType::Toggle,
                    commands::PDF_ANNOTATE_HIGHLIGHT,
                    "Highlight",
                    "highlighter",
                    "unibrowser.group.pdf.annotate",
                    family_ids::PDF_ANNOTATION,
                    ctx,
                    resolver,
                ),
                control_spec(
                    "unibrowser.control.pdf.stamp",
                    ControlType::Button,
                    commands::PDF_ANNOTATE_STAMP,
                    "Stamp",
                    "stamp",
                    "unibrowser.group.pdf.annotate",
                    family_ids::PDF_ANNOTATION,
                    ctx,
                    resolver,
                ),
                control_spec(
                    "unibrowser.control.pdf.ink",
                    ControlType::Button,
                    commands::PDF_ANNOTATE_INK,
                    "Ink",
                    "pen-line",
                    "unibrowser.group.pdf.annotate",
                    family_ids::PDF_ANNOTATION,
                    ctx,
                    resolver,
                ),
            ],
        }],
    }
}

fn image_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    if ctx.work != WorkContext::Image {
        return ResolvedFamily {
            family_id: family_ids::IMAGE_ADJUST.into(),
            label: "Adjust".into(),
            overflow_priority: OverflowPriority::Normal,
            groups: vec![],
        };
    }
    ResolvedFamily {
        family_id: family_ids::IMAGE_ADJUST.into(),
        label: "Adjust".into(),
        overflow_priority: OverflowPriority::Normal,
        groups: vec![ResolvedGroup {
            group_id: "unibrowser.group.image".into(),
            label: "Adjust".into(),
            overflow_priority: OverflowPriority::Normal,
            controls: vec![control_spec(
                "unibrowser.control.image.curves",
                ControlType::Button,
                commands::IMAGE_CURVES,
                "Curves",
                "image",
                "unibrowser.group.image",
                family_ids::IMAGE_ADJUST,
                ctx,
                resolver,
            )],
        }],
    }
}

fn control_spec(
    control_id: &str,
    control_type: ControlType,
    command_id: &str,
    label: &str,
    icon: &str,
    group_id: &str,
    family_id: &str,
    ctx: &ResolutionContext,
    resolver: &CapabilityResolver,
) -> ResolvedControl {
    let capability = resolver.resolve_control(control_id, command_id, ctx);
    let value = if control_id == control_ids::TEXT_SIZE || control_id == control_ids::EDITOR_SIZE {
        ctx.host
            .selection
            .font_size_pt
            .map(|pt| format!("{:.0} pt", pt))
    } else {
        None
    };
    ResolvedControl {
        control_id: control_id.into(),
        control_type,
        command_ids: vec![command_id.into()],
        capability,
        selection: SelectionState::Off,
        value,
        label: label.into(),
        short_label: None,
        icon: icon.into(),
        shortcut: None,
        density_variant: DensityVariant::Compact,
        provider_id: None,
        context_revision: ctx.revision,
        group_id: group_id.into(),
        family_id: family_id.into(),
    }
}
