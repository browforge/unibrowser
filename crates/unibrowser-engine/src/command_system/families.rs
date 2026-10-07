use unibrowser_mod_api::{
    commands, control_ids, family_ids, group_ids, ControlType, DensityVariant, OverflowPriority,
    ResolvedControl, ResolvedFamily, ResolvedGroup, SelectionState, WorkContext,
};

use super::context::ResolutionContext;
use super::resolver::CapabilityResolver;

pub fn core_tab_layout(
    ctx: &ResolutionContext,
    resolver: &CapabilityResolver,
) -> Vec<ResolvedFamily> {
    if ctx.work == WorkContext::Start {
        return vec![start_family(ctx, resolver)];
    }
    if ctx.work == WorkContext::Browse {
        return vec![browse_family(ctx, resolver)];
    }
    if ctx.work == WorkContext::Settings {
        return vec![settings_family(ctx, resolver)];
    }
    if ctx.work == WorkContext::Mods {
        return vec![mods_family(ctx, resolver)];
    }
    if ctx.work == WorkContext::Cad3d {
        return vec![cad_family(ctx, resolver)];
    }
    if ctx.work == WorkContext::Project {
        return vec![project_family(ctx, resolver)];
    }
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

fn project_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let family_id = family_ids::PROJECT;
    let control = |id: &str, command: &str, label: &str, icon: &str, group: &str| {
        control_spec(
            id,
            ControlType::Button,
            command,
            label,
            icon,
            group,
            family_id,
            ctx,
            resolver,
        )
    };
    let group = |id: &str,
                 label: &str,
                 priority: OverflowPriority,
                 controls: Vec<ResolvedControl>| ResolvedGroup {
        group_id: id.into(),
        label: label.into(),
        overflow_priority: priority,
        controls,
    };
    ResolvedFamily {
        family_id: family_id.into(),
        label: "Project".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![
            group(
                "unibrowser.group.project.add",
                "Add",
                OverflowPriority::Primary,
                vec![
                    control(
                        "unibrowser.control.project.addAsset",
                        "unibrowser.project.addAsset",
                        "Add asset",
                        "plus",
                        "unibrowser.group.project.add",
                    ),
                    control(
                        "unibrowser.control.project.addTitle",
                        "unibrowser.project.addTitle",
                        "Add title",
                        "type",
                        "unibrowser.group.project.add",
                    ),
                ],
            ),
            group(
                "unibrowser.group.project.edit",
                "Edit",
                OverflowPriority::Primary,
                vec![
                    control(
                        "unibrowser.control.project.split",
                        "unibrowser.project.split",
                        "Split",
                        "copy",
                        "unibrowser.group.project.edit",
                    ),
                    control(
                        "unibrowser.control.project.deleteClip",
                        "unibrowser.project.deleteClip",
                        "Delete clip",
                        "trash",
                        "unibrowser.group.project.edit",
                    ),
                ],
            ),
            group(
                "unibrowser.group.project.timeline",
                "Timeline",
                OverflowPriority::Normal,
                vec![
                    control(
                        "unibrowser.control.project.snap",
                        "unibrowser.project.snap",
                        "Snap",
                        "pin",
                        "unibrowser.group.project.timeline",
                    ),
                    control(
                        "unibrowser.control.project.marker",
                        "unibrowser.project.marker",
                        "Marker",
                        "bookmark",
                        "unibrowser.group.project.timeline",
                    ),
                    control(
                        "unibrowser.control.project.zoomTimeline",
                        "unibrowser.project.zoomTimeline",
                        "Timeline zoom",
                        "zoom-in",
                        "unibrowser.group.project.timeline",
                    ),
                ],
            ),
            group(
                "unibrowser.group.project.mode",
                "Mode",
                OverflowPriority::Primary,
                vec![
                    control(
                        "unibrowser.control.project.preview",
                        "unibrowser.project.preview",
                        "Preview",
                        "play",
                        "unibrowser.group.project.mode",
                    ),
                    control(
                        "unibrowser.control.project.live",
                        "unibrowser.project.live",
                        "LIVE",
                        "globe",
                        "unibrowser.group.project.mode",
                    ),
                    control(
                        "unibrowser.control.project.record",
                        "unibrowser.project.record",
                        "Record",
                        "circle-dot",
                        "unibrowser.group.project.mode",
                    ),
                ],
            ),
            group(
                "unibrowser.group.project.output",
                "Output",
                OverflowPriority::Primary,
                vec![control(
                    "unibrowser.control.project.export",
                    "unibrowser.project.export",
                    "Export",
                    "download",
                    "unibrowser.group.project.output",
                )],
            ),
        ],
    }
}

fn cad_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let family_id = "unibrowser.family.cad";
    let make = |control_id: &str,
                command_id: &str,
                label: &str,
                icon: &str,
                group_id: &str,
                priority: OverflowPriority| ResolvedGroup {
        group_id: group_id.into(),
        label: label.into(),
        overflow_priority: priority,
        controls: vec![control_spec(
            control_id,
            ControlType::Button,
            command_id,
            label,
            icon,
            group_id,
            family_id,
            ctx,
            resolver,
        )],
    };
    ResolvedFamily {
        family_id: family_id.into(),
        label: "CAD / 3D".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![
            make(
                "unibrowser.control.cad.find",
                "unibrowser.cad.find",
                "Find entity",
                "search",
                "unibrowser.group.cad.find",
                OverflowPriority::Primary,
            ),
            make(
                "unibrowser.control.cad.fitView",
                "unibrowser.cad.fitView",
                "Fit view",
                "maximize",
                "unibrowser.group.cad.view",
                OverflowPriority::Primary,
            ),
            ResolvedGroup {
                group_id: "unibrowser.group.cad.zoom".into(),
                label: "Zoom".into(),
                overflow_priority: OverflowPriority::Primary,
                controls: vec![
                    control_spec(
                        "unibrowser.control.cad.zoomOut",
                        ControlType::Button,
                        "unibrowser.cad.zoomOut",
                        "Zoom out",
                        "zoom-out",
                        "unibrowser.group.cad.zoom",
                        family_id,
                        ctx,
                        resolver,
                    ),
                    control_spec(
                        "unibrowser.control.cad.zoomIn",
                        ControlType::Button,
                        "unibrowser.cad.zoomIn",
                        "Zoom in",
                        "zoom-in",
                        "unibrowser.group.cad.zoom",
                        family_id,
                        ctx,
                        resolver,
                    ),
                ],
            },
            make(
                "unibrowser.control.cad.measure",
                "unibrowser.cad.measure",
                "Measure",
                "ruler",
                "unibrowser.group.cad.measure",
                OverflowPriority::Normal,
            ),
            make(
                "unibrowser.control.cad.exportStep",
                "unibrowser.cad.exportStep",
                "Export STEP",
                "download",
                "unibrowser.group.cad.export",
                OverflowPriority::Low,
            ),
        ],
    }
}

fn settings_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let group_id = "unibrowser.group.settings";
    let controls = vec![
        control_spec(
            control_ids::SETTINGS_SEARCH,
            ControlType::Button,
            commands::SETTINGS_SEARCH,
            "Search settings",
            "search",
            group_id,
            family_ids::SETTINGS,
            ctx,
            resolver,
        ),
        control_spec(
            control_ids::SETTINGS_KEYBINDINGS,
            ControlType::Button,
            commands::SETTINGS_KEYBINDINGS,
            "Keybindings",
            "type",
            group_id,
            family_ids::SETTINGS,
            ctx,
            resolver,
        ),
        control_spec(
            control_ids::SETTINGS_DIAGNOSTICS,
            ControlType::Button,
            commands::SETTINGS_DIAGNOSTICS,
            "Diagnostics",
            "info",
            group_id,
            family_ids::SETTINGS,
            ctx,
            resolver,
        ),
    ];
    ResolvedFamily {
        family_id: family_ids::SETTINGS.into(),
        label: "Settings".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![ResolvedGroup {
            group_id: group_id.into(),
            label: "Settings".into(),
            overflow_priority: OverflowPriority::Primary,
            controls,
        }],
    }
}

fn mods_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let make = |control_id, command_id, label, icon, group_id| {
        control_spec(
            control_id,
            ControlType::Button,
            command_id,
            label,
            icon,
            group_id,
            family_ids::MODS,
            ctx,
            resolver,
        )
    };
    ResolvedFamily {
        family_id: family_ids::MODS.into(),
        label: "Mods".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![
            ResolvedGroup {
                group_id: "unibrowser.group.mods.discover".into(),
                label: "Discover".into(),
                overflow_priority: OverflowPriority::Primary,
                controls: vec![
                    make(
                        control_ids::MODS_SEARCH,
                        commands::MODS_SEARCH,
                        "Search",
                        "search",
                        "unibrowser.group.mods.discover",
                    ),
                    make(
                        control_ids::MODS_INSTALL,
                        commands::MODS_INSTALL,
                        "Install",
                        "download",
                        "unibrowser.group.mods.discover",
                    ),
                ],
            },
            ResolvedGroup {
                group_id: "unibrowser.group.mods.manage".into(),
                label: "Manage".into(),
                overflow_priority: OverflowPriority::Normal,
                controls: vec![
                    make(
                        control_ids::MODS_UPDATES,
                        commands::MODS_UPDATES,
                        "Updates",
                        "refresh-cw",
                        "unibrowser.group.mods.manage",
                    ),
                    make(
                        control_ids::MODS_PERMISSIONS,
                        commands::MODS_PERMISSIONS,
                        "Permissions",
                        "shield",
                        "unibrowser.group.mods.manage",
                    ),
                    make(
                        control_ids::MODS_OPEN_VSX,
                        commands::MODS_OPEN_VSX,
                        "Open VSX network",
                        "globe",
                        "unibrowser.group.mods.manage",
                    ),
                ],
            },
        ],
    }
}

fn browse_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let make = |control_id, command_id, label, icon, group_id| {
        control_spec(
            control_id,
            ControlType::Button,
            command_id,
            label,
            icon,
            group_id,
            family_ids::BROWSE,
            ctx,
            resolver,
        )
    };
    let groups = vec![
        ResolvedGroup {
            group_id: "unibrowser.group.browse.create".into(),
            label: "Create".into(),
            overflow_priority: OverflowPriority::Primary,
            controls: vec![
                make(
                    control_ids::BROWSE_NEW_FOLDER,
                    commands::BROWSE_NEW_FOLDER,
                    "New folder",
                    "folder-plus",
                    "unibrowser.group.browse.create",
                ),
                make(
                    control_ids::BROWSE_NEW_TEXT_FILE,
                    commands::BROWSE_NEW_TEXT_FILE,
                    "New text file",
                    "file-plus",
                    "unibrowser.group.browse.create",
                ),
            ],
        },
        ResolvedGroup {
            group_id: "unibrowser.group.browse.open".into(),
            label: "Open".into(),
            overflow_priority: OverflowPriority::Primary,
            controls: vec![
                make(
                    control_ids::BROWSE_OPEN,
                    commands::BROWSE_OPEN,
                    "Open",
                    "folder-open",
                    "unibrowser.group.browse.open",
                ),
                make(
                    control_ids::BROWSE_COPY_PATHS,
                    commands::BROWSE_COPY_PATHS,
                    "Copy paths",
                    "copy",
                    "unibrowser.group.browse.open",
                ),
            ],
        },
        ResolvedGroup {
            group_id: "unibrowser.group.browse.transfer".into(),
            label: "Transfer".into(),
            overflow_priority: OverflowPriority::Normal,
            controls: vec![
                make(
                    control_ids::BROWSE_COPY_TO,
                    commands::BROWSE_COPY_TO,
                    "Copy to",
                    "copy",
                    "unibrowser.group.browse.transfer",
                ),
                make(
                    control_ids::BROWSE_MOVE_TO,
                    commands::BROWSE_MOVE_TO,
                    "Move to",
                    "move",
                    "unibrowser.group.browse.transfer",
                ),
            ],
        },
        ResolvedGroup {
            group_id: "unibrowser.group.browse.manage".into(),
            label: "Manage".into(),
            overflow_priority: OverflowPriority::Low,
            controls: vec![
                make(
                    control_ids::BROWSE_RENAME,
                    commands::BROWSE_RENAME,
                    "Rename",
                    "pencil",
                    "unibrowser.group.browse.manage",
                ),
                make(
                    control_ids::BROWSE_COMPRESS,
                    commands::BROWSE_COMPRESS,
                    "Compress",
                    "archive",
                    "unibrowser.group.browse.manage",
                ),
                make(
                    control_ids::BROWSE_PROPERTIES,
                    commands::BROWSE_PROPERTIES,
                    "Properties",
                    "info",
                    "unibrowser.group.browse.manage",
                ),
                make(
                    control_ids::BROWSE_DELETE,
                    commands::BROWSE_DELETE,
                    "Delete",
                    "trash",
                    "unibrowser.group.browse.manage",
                ),
            ],
        },
        ResolvedGroup {
            group_id: "unibrowser.group.browse.view".into(),
            label: "View".into(),
            overflow_priority: OverflowPriority::Low,
            controls: vec![
                make(
                    control_ids::BROWSE_SORT,
                    commands::BROWSE_SORT,
                    "Sort",
                    "arrow-up-down",
                    "unibrowser.group.browse.view",
                ),
                make(
                    control_ids::BROWSE_SORT_DIRECTION,
                    commands::BROWSE_SORT_DIRECTION,
                    "Direction",
                    "arrow-down-up",
                    "unibrowser.group.browse.view",
                ),
                make(
                    control_ids::BROWSE_SHOW_HIDDEN,
                    commands::BROWSE_SHOW_HIDDEN,
                    "Hidden files",
                    "eye",
                    "unibrowser.group.browse.view",
                ),
                make(
                    control_ids::BROWSE_SET_WORKSPACE,
                    commands::BROWSE_SET_WORKSPACE,
                    "Set workspace",
                    "folder",
                    "unibrowser.group.browse.view",
                ),
                make(
                    control_ids::BROWSE_CLEAR_WORKSPACE,
                    commands::BROWSE_CLEAR_WORKSPACE,
                    "Clear workspace",
                    "x",
                    "unibrowser.group.browse.view",
                ),
            ],
        },
    ];
    ResolvedFamily {
        family_id: family_ids::BROWSE.into(),
        label: "Browse".into(),
        overflow_priority: OverflowPriority::Primary,
        groups,
    }
}

fn start_family(ctx: &ResolutionContext, resolver: &CapabilityResolver) -> ResolvedFamily {
    let open = ResolvedGroup {
        group_id: "unibrowser.group.start.open".into(),
        label: "Open".into(),
        overflow_priority: OverflowPriority::Normal,
        controls: vec![
            control_spec(
                control_ids::START_OPEN_FILE,
                ControlType::Button,
                commands::START_OPEN_FILE,
                "File",
                "file-text",
                "unibrowser.group.start.open",
                family_ids::START,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::START_OPEN_FOLDER,
                ControlType::Button,
                commands::START_OPEN_FOLDER,
                "Folder",
                "folder",
                "unibrowser.group.start.open",
                family_ids::START,
                ctx,
                resolver,
            ),
        ],
    };
    let create = ResolvedGroup {
        group_id: "unibrowser.group.start.create".into(),
        label: "Create".into(),
        overflow_priority: OverflowPriority::Normal,
        controls: vec![control_spec(
            control_ids::START_NEW_PROJECT,
            ControlType::Button,
            commands::START_NEW_PROJECT,
            "Project",
            "layout-grid",
            "unibrowser.group.start.create",
            family_ids::START,
            ctx,
            resolver,
        )],
    };
    let recent = ResolvedGroup {
        group_id: "unibrowser.group.start.recent".into(),
        label: "Recent".into(),
        overflow_priority: OverflowPriority::Low,
        controls: vec![control_spec(
            control_ids::START_RECENT,
            ControlType::Button,
            commands::START_RECENT,
            "Recent",
            "clock",
            "unibrowser.group.start.recent",
            family_ids::START,
            ctx,
            resolver,
        )],
    };
    ResolvedFamily {
        family_id: family_ids::START.into(),
        label: "Start".into(),
        overflow_priority: OverflowPriority::Primary,
        groups: vec![open, create, recent],
    }
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
    if matches!(ctx.work, WorkContext::Pdf | WorkContext::Image) {
        return ResolvedFamily {
            family_id: family_ids::TEXT_FORMATTING.into(),
            label: "Formatting".into(),
            overflow_priority: OverflowPriority::Primary,
            groups: vec![],
        };
    }
    if ctx.work == WorkContext::CodeEditor {
        return editor_appearance_family(ctx, resolver);
    }
    text_formatting_family(ctx, resolver)
}

fn editor_appearance_family(
    ctx: &ResolutionContext,
    resolver: &CapabilityResolver,
) -> ResolvedFamily {
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

fn text_size_label(ctx: &ResolutionContext) -> &'static str {
    match ctx.work {
        WorkContext::PlainText | WorkContext::Markdown => "Editor text size",
        _ => "Size",
    }
}

fn text_formatting_family(
    ctx: &ResolutionContext,
    resolver: &CapabilityResolver,
) -> ResolvedFamily {
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
                text_size_label(ctx),
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
            label: "PDF".into(),
            overflow_priority: OverflowPriority::Normal,
            groups: vec![],
        };
    }
    let tools = ResolvedGroup {
        group_id: "unibrowser.group.pdf.tools".into(),
        label: "Tools".into(),
        overflow_priority: OverflowPriority::Low,
        controls: vec![control_spec(
            control_ids::PDF_SELECT,
            ControlType::SplitButton,
            commands::PDF_TOOL_SELECT,
            "Select",
            "lasso",
            "unibrowser.group.pdf.tools",
            family_ids::PDF_ANNOTATION,
            ctx,
            resolver,
        )],
    };
    let annotate = ResolvedGroup {
        group_id: "unibrowser.group.pdf.annotate".into(),
        label: "Annotate".into(),
        overflow_priority: OverflowPriority::Low,
        controls: vec![
            control_spec(
                "unibrowser.control.pdf.highlight",
                ControlType::SplitButton,
                commands::PDF_ANNOTATE_HIGHLIGHT,
                "Highlight",
                "highlighter",
                "unibrowser.group.pdf.annotate",
                family_ids::PDF_ANNOTATION,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::PDF_NOTE,
                ControlType::Button,
                commands::PDF_ANNOTATE_NOTE,
                "Note",
                "message-square",
                "unibrowser.group.pdf.annotate",
                family_ids::PDF_ANNOTATION,
                ctx,
                resolver,
            ),
            control_spec(
                control_ids::PDF_AREA,
                ControlType::Button,
                commands::PDF_ANNOTATE_AREA,
                "Area",
                "square-dashed",
                "unibrowser.group.pdf.annotate",
                family_ids::PDF_ANNOTATION,
                ctx,
                resolver,
            ),
        ],
    };
    let page = ResolvedGroup {
        group_id: "unibrowser.group.pdf.page".into(),
        label: "Page".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![control_spec(
            control_ids::PDF_PAGE,
            ControlType::SplitButton,
            commands::PDF_PAGE_NAV,
            "1 / 1",
            "file",
            "unibrowser.group.pdf.page",
            family_ids::PDF_ANNOTATION,
            ctx,
            resolver,
        )],
    };
    let view = ResolvedGroup {
        group_id: "unibrowser.group.pdf.view".into(),
        label: "View".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![control_spec(
            control_ids::PDF_VIEW_ZOOM,
            ControlType::SplitButton,
            commands::PDF_VIEW_ZOOM,
            "100%",
            "zoom-in",
            "unibrowser.group.pdf.view",
            family_ids::PDF_ANNOTATION,
            ctx,
            resolver,
        )],
    };
    let export = ResolvedGroup {
        group_id: "unibrowser.group.pdf.export".into(),
        label: "Export".into(),
        overflow_priority: OverflowPriority::Primary,
        controls: vec![control_spec(
            control_ids::PDF_EXPORT,
            ControlType::Button,
            commands::EXPORT_PRINT_PDF,
            "Export PDF",
            "share",
            "unibrowser.group.pdf.export",
            family_ids::PDF_ANNOTATION,
            ctx,
            resolver,
        )],
    };
    ResolvedFamily {
        family_id: family_ids::PDF_ANNOTATION.into(),
        label: "PDF".into(),
        overflow_priority: OverflowPriority::Normal,
        groups: vec![tools, annotate, page, view, export],
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
        groups: vec![
            ResolvedGroup {
                group_id: "unibrowser.group.image.view".into(),
                label: "View".into(),
                overflow_priority: OverflowPriority::Primary,
                controls: vec![
                    control_spec(
                        "unibrowser.control.image.zoomOut",
                        ControlType::Button,
                        "unibrowser.image.zoomOut",
                        "Zoom out",
                        "zoom-out",
                        "unibrowser.group.image.view",
                        family_ids::IMAGE_ADJUST,
                        ctx,
                        resolver,
                    ),
                    control_spec(
                        "unibrowser.control.image.zoomIn",
                        ControlType::Button,
                        "unibrowser.image.zoomIn",
                        "Zoom in",
                        "zoom-in",
                        "unibrowser.group.image.view",
                        family_ids::IMAGE_ADJUST,
                        ctx,
                        resolver,
                    ),
                ],
            },
            ResolvedGroup {
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
            },
        ],
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
