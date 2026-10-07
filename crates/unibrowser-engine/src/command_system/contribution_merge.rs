//! Merge declarative mod contributions into resolved families (CMD-09).

use unibrowser_mod_api::{
    CapabilityState, ContributedControl, ControlType, DensityVariant, ModContributes,
    OverflowPriority, ResolvedControl, ResolvedFamily, ResolvedGroup, SelectionState,
};

use crate::ui_command::UiCommandSlot;

#[derive(Debug, Clone)]
pub struct LoadedModContribution {
    pub mod_name: String,
    pub contributes: ModContributes,
}

#[derive(Debug, Clone, Default)]
pub struct ContributionMergeDiagnostics {
    pub messages: Vec<String>,
}

pub fn merge_mod_contributions(
    families: &mut [ResolvedFamily],
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
) {
    let mut seen_control_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for family in families.iter() {
        for group in &family.groups {
            for control in &group.controls {
                seen_control_ids.insert(control.control_id.clone());
            }
        }
    }

    for loaded in contributions {
        let c = &loaded.contributes;
        for control in &c.controls {
            if seen_control_ids.contains(&control.id) {
                diagnostics.messages.push(format!(
                    "duplicate control id {} from mod {} (skipped)",
                    control.id,
                    loaded.mod_name
                ));
                continue;
            }
            insert_control(families, control, &loaded.mod_name, diagnostics);
            seen_control_ids.insert(control.id.clone());
        }
        for family in &c.families {
            if !family.extend {
                diagnostics.messages.push(format!(
                    "mod {} family {} must set extend:true",
                    loaded.mod_name,
                    family.id
                ));
                continue;
            }
            let target = families.iter_mut().find(|f| f.family_id == family.id);
            if target.is_none() {
                diagnostics.messages.push(format!(
                    "mod {} extends unknown family {}",
                    loaded.mod_name,
                    family.id
                ));
                continue;
            }
            let target = target.unwrap();
            for control in &family.controls {
                if seen_control_ids.contains(&control.id) {
                    diagnostics.messages.push(format!(
                        "duplicate control id {} from mod {} (skipped)",
                        control.id,
                        loaded.mod_name
                    ));
                    continue;
                }
                insert_control_in_family(target, control, &loaded.mod_name);
                seen_control_ids.insert(control.id.clone());
            }
        }
    }
}

fn insert_control(
    families: &mut [ResolvedFamily],
    control: &ContributedControl,
    mod_name: &str,
    diagnostics: &mut ContributionMergeDiagnostics,
) {
    let family_id = "unibrowser.family.text-formatting";
    let Some(family) = families.iter_mut().find(|f| f.family_id == family_id) else {
        diagnostics.messages.push(format!(
            "mod {} control {} has no target family",
            mod_name,
            control.id
        ));
        return;
    };
    insert_control_in_family(family, control, mod_name);
}

fn insert_control_in_family(
    family: &mut ResolvedFamily,
    control: &ContributedControl,
    mod_name: &str,
) {
    let resolved = contributed_to_resolved(control, mod_name);
    let group_id = format!("{}.group", mod_name);
    for group in &mut family.groups {
        if group.controls.iter().any(|c| {
            control
                .insert
                .as_ref()
                .and_then(|i| i.after.as_ref())
                .map(|after| c.control_id == *after)
                .unwrap_or(false)
        }) {
            insert_at_point(&mut group.controls, control, resolved);
            return;
        }
    }
    if let Some(group) = family.groups.iter_mut().find(|g| g.group_id == group_id) {
        group.controls.push(resolved);
        return;
    }
    family.groups.push(ResolvedGroup {
        group_id,
        label: format!("{} mod", mod_name),
        overflow_priority: OverflowPriority::Low,
        controls: vec![resolved],
    });
}

fn insert_at_point(
    controls: &mut Vec<ResolvedControl>,
    spec: &ContributedControl,
    resolved: ResolvedControl,
) {
    if let Some(insert) = &spec.insert {
        if let Some(after) = &insert.after {
            if let Some(idx) = controls.iter().position(|c| c.control_id == *after) {
                controls.insert(idx + 1, resolved);
                return;
            }
        }
        if let Some(before) = &insert.before {
            if let Some(idx) = controls.iter().position(|c| c.control_id == *before) {
                controls.insert(idx, resolved);
                return;
            }
        }
        if insert.end {
            controls.push(resolved);
            return;
        }
    }
    controls.push(resolved);
}

fn contributed_to_resolved(control: &ContributedControl, mod_name: &str) -> ResolvedControl {
    ResolvedControl {
        control_id: control.id.clone(),
        control_type: ControlType::Button,
        command_ids: vec![control.command.clone()],
        capability: CapabilityState::Supported,
        selection: SelectionState::Off,
        value: None,
        label: control.id.split('.').next_back().unwrap_or("Mod").into(),
        short_label: None,
        icon: "extension".into(),
        shortcut: None,
        density_variant: DensityVariant::Compact,
        provider_id: Some(format!("{}::native", mod_name)),
        context_revision: 0,
        group_id: format!("{}.group", mod_name),
        family_id: "unibrowser.family.text-formatting".into(),
    }
}

pub fn merge_toolbar_layouts(
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
) -> Vec<crate::ui_command::FloatingToolbarLayout> {
    use crate::ui_command::{FloatingToolbarLayout, ToolbarAnchor, ToolbarCoordSpace, ToolbarPresentation};
    let mut out = Vec::new();
    for loaded in contributions {
        for bar in &loaded.contributes.toolbars {
            if out.iter().any(|b: &FloatingToolbarLayout| b.id == bar.id) {
                diagnostics.messages.push(format!(
                    "duplicate toolbar id {} from mod {}",
                    bar.id,
                    loaded.mod_name
                ));
                continue;
            }
            let commands = toolbar_command_slots(&loaded.contributes, &bar.id);
            out.push(FloatingToolbarLayout {
                id: bar.id.clone(),
                label: bar.label.clone(),
                anchor: ToolbarAnchor::Floating,
                x: 48.0,
                y: 120.0,
                coord_space: ToolbarCoordSpace::Screen,
                presentation: if bar.detachable {
                    ToolbarPresentation::DetachedWindow
                } else {
                    ToolbarPresentation::InWindow
                },
                commands,
            });
        }
    }
    out
}

fn toolbar_command_slots(contributes: &ModContributes, bar_id: &str) -> Vec<UiCommandSlot> {
    let bar = contributes.toolbars.iter().find(|t| t.id == bar_id);
    let Some(bar) = bar else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for group_id in &bar.groups {
        let group = contributes.groups.iter().find(|g| g.id == *group_id);
        if let Some(group) = group {
            for control_id in &group.controls {
                let control = contributes.controls.iter().find(|c| c.id == *control_id);
                if let Some(control) = control {
                    out.push(UiCommandSlot {
                        command_id: control.command.clone(),
                    });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_mod_api::{ContributedFamily, InsertPoint, family_ids};
    use crate::command_system::resolver::CapabilityResolver;
    use crate::command_system::context::ResolutionContext;
    use crate::command_system::core_tab_layout;
    use unibrowser_mod_api::{CommandContext, HostContext, Mode};

    fn sample_families() -> Vec<ResolvedFamily> {
        let host = HostContext {
            mode: Mode::Text,
            doc_type: String::new(),
            backend_id: String::new(),
            selection: Default::default(),
            capabilities: Default::default(),
        };
        let ctx = ResolutionContext::new(
            CommandContext::Text,
            host,
            Some("readme.md".into()),
            1,
        );
        let resolver = CapabilityResolver::default();
        core_tab_layout(&ctx, &resolver)
    }

    #[test]
    fn extends_text_family_with_mod_control() {
        let mut families = sample_families();
        let mut diag = ContributionMergeDiagnostics::default();
        let contributions = vec![LoadedModContribution {
            mod_name: "cmd-proving".into(),
            contributes: ModContributes {
                families: vec![ContributedFamily {
                    id: family_ids::TEXT_FORMATTING.into(),
                    extend: true,
                    controls: vec![ContributedControl {
                        id: "cmd-proving.control.chip".into(),
                        control_type: "button".into(),
                        command: "cmd-proving.subscript.apply".into(),
                        insert: Some(InsertPoint {
                            after: Some("unibrowser.control.text.subscript".into()),
                            before: None,
                            end: false,
                        }),
                    }],
                }],
                ..ModContributes::default()
            },
        }];
        merge_mod_contributions(&mut families, &contributions, &mut diag);
        let all_controls = families
            .iter()
            .flat_map(|f| f.groups.iter())
            .flat_map(|g| g.controls.iter())
            .map(|c| c.control_id.as_str())
            .collect::<Vec<_>>();
        assert!(all_controls.iter().any(|id| *id == "unibrowser.control.text.subscript"));
        assert!(all_controls.iter().any(|id| *id == "cmd-proving.control.chip"));
    }
}
