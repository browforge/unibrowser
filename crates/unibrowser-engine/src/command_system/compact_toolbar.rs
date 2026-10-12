//! 32px compact text toolbar families (ADR 0034).

use unibrowser_mod_api::{
    commands, control_ids, family_ids, group_ids, ControlType, DensityVariant, OverflowPriority,
    ResolvedControl, ResolvedFamily, ResolvedGroup, SelectionState, WorkContext,
};

use super::context::ResolutionContext;
use super::families::control_spec;
use super::resolver::CapabilityResolver;

pub fn uses_compact_text_toolbar(work: WorkContext) -> bool {
    matches!(
        work,
        WorkContext::Markdown | WorkContext::PlainText | WorkContext::RichDocument
    )
}

pub fn editor_display_size_cm_family(
    ctx: &ResolutionContext,
    resolver: &CapabilityResolver,
) -> Option<ResolvedFamily> {
    if !uses_compact_text_toolbar(ctx.work) {
        return None;
    }
    let label = match ctx.work {
        WorkContext::PlainText | WorkContext::Markdown => "Editor text size",
        _ => "Size",
    };
    Some(ResolvedFamily {
        family_id: family_ids::EDITOR_APPEARANCE.into(),
        label: "Editor".into(),
        overflow_priority: OverflowPriority::Low,
        groups: vec![ResolvedGroup {
            group_id: group_ids::TEXT_TYPE.into(),
            label: "Editor".into(),
            overflow_priority: OverflowPriority::Low,
            controls: vec![control_spec(
                control_ids::TEXT_SIZE,
                ControlType::ValueStepper,
                commands::FONT_SIZE_INCREASE,
                label,
                "type",
                group_ids::TEXT_TYPE,
                family_ids::EDITOR_APPEARANCE,
                ctx,
                resolver,
            )],
        }],
    })
}

pub fn compact_text_toolbar_families(
    ctx: &ResolutionContext,
    resolver: &CapabilityResolver,
) -> Vec<ResolvedFamily> {
    if !uses_compact_text_toolbar(ctx.work) {
        return Vec::new();
    }
    let block = ResolvedGroup {
        group_id: "unibrowser.group.text.compact.block".into(),
        label: "Block".into(),
        overflow_priority: OverflowPriority::Critical,
        controls: vec![block_style_control(ctx, resolver)],
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
                family_ids::TEXT_COMPACT_TOOLBAR,
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
                family_ids::TEXT_COMPACT_TOOLBAR,
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
                family_ids::TEXT_COMPACT_TOOLBAR,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_CODE_INLINE,
                ControlType::Toggle,
                commands::TEXT_CODE_INLINE,
                "Inline code",
                "code",
                group_ids::TEXT_EMPHASIS,
                family_ids::TEXT_COMPACT_TOOLBAR,
                ctx,
                resolver,
            ),
        ],
    };
    let links = ResolvedGroup {
        group_id: "unibrowser.group.text.compact.links".into(),
        label: "Links".into(),
        overflow_priority: OverflowPriority::Normal,
        controls: vec![
            control_spec(
                control_ids::TEXT_LINK,
                ControlType::Button,
                commands::TEXT_LINK,
                "Link",
                "link",
                "unibrowser.group.text.compact.links",
                family_ids::TEXT_COMPACT_TOOLBAR,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_LIST,
                ControlType::SplitButton,
                commands::TEXT_LIST_BULLET,
                "List",
                "list",
                "unibrowser.group.text.compact.links",
                family_ids::TEXT_COMPACT_TOOLBAR,
                ctx,
                resolver,
            ),
        ],
    };
    let utility = ResolvedGroup {
        group_id: "unibrowser.group.text.compact.utility".into(),
        label: "Utility".into(),
        overflow_priority: OverflowPriority::Low,
        controls: vec![
            control_spec(
                control_ids::TEXT_CLEAR,
                ControlType::Button,
                commands::TEXT_CLEAR,
                "Clear formatting",
                "remove-formatting",
                "unibrowser.group.text.compact.utility",
                family_ids::TEXT_COMPACT_TOOLBAR,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::TEXT_COMPACT_OVERFLOW,
                ControlType::MenuButton,
                commands::TEXT_COMPACT_OVERFLOW,
                "More formatting",
                "ellipsis",
                "unibrowser.group.text.compact.utility",
                family_ids::TEXT_COMPACT_TOOLBAR,
                ctx,
                resolver,
            ),
        ],
    };
    vec![ResolvedFamily {
        family_id: family_ids::TEXT_COMPACT_TOOLBAR.into(),
        label: "Compact text".into(),
        overflow_priority: OverflowPriority::Critical,
        groups: vec![block, emphasis, links, utility],
    }]
}

fn block_style_control(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedControl {
    let (default_cmd, label) = match ctx.work {
        WorkContext::PlainText => (commands::SEMANTIC_BLOCK_PARAGRAPH, "Paragraph"),
        WorkContext::Markdown => (commands::SEMANTIC_BLOCK_HEADING_2, "Heading 2"),
        WorkContext::RichDocument => (commands::SEMANTIC_BLOCK_HEADING_1, "Heading 1"),
        _ => (commands::SEMANTIC_BLOCK_PARAGRAPH, "Paragraph"),
    };
    let capability = resolver.resolve_control(
        control_ids::SEMANTIC_BLOCK_STYLE,
        default_cmd,
        ctx,
    );
    ResolvedControl {
        control_id: control_ids::SEMANTIC_BLOCK_STYLE.into(),
        control_type: ControlType::Dropdown,
        command_ids: vec![
            commands::SEMANTIC_BLOCK_PARAGRAPH.into(),
            commands::SEMANTIC_BLOCK_HEADING_1.into(),
            commands::SEMANTIC_BLOCK_HEADING_2.into(),
            commands::SEMANTIC_BLOCK_HEADING_3.into(),
            commands::SEMANTIC_BLOCK_HEADING_4.into(),
            commands::SEMANTIC_BLOCK_HEADING_5.into(),
            commands::SEMANTIC_BLOCK_HEADING_6.into(),
            commands::SEMANTIC_BLOCK_BLOCKQUOTE.into(),
            commands::SEMANTIC_BLOCK_CODE.into(),
        ],
        capability,
        selection: SelectionState::Value,
        value: Some(label.into()),
        label: label.into(),
        short_label: Some("¶".into()),
        icon: "pilcrow".into(),
        shortcut: None,
        density_variant: DensityVariant::Compact,
        provider_id: None,
        context_revision: ctx.revision,
        group_id: "unibrowser.group.text.compact.block".into(),
        family_id: family_ids::TEXT_COMPACT_TOOLBAR.into(),
    }
}
