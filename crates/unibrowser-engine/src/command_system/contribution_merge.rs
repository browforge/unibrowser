//! Merge declarative mod contributions into resolved families (CMD-09).

use unibrowser_mod_api::{
    CapabilityState, ContributedControl, ControlType, DensityVariant, ModContributes,
    OverflowPriority, ResolvedControl, ResolvedFamily, ResolvedGroup, SelectionState,
};

use crate::ui_command::UiCommandSlot;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Debug, Clone)]
pub struct LoadedModContribution {
    pub mod_name: String,
    pub contributes: ModContributes,
}

#[derive(Debug, Clone, Default)]
pub struct ContributionMergeDiagnostics {
    pub messages: Vec<String>,
}

pub fn diagnose_contributions(
    contributions: &[LoadedModContribution],
) -> ContributionMergeDiagnostics {
    let mut diagnostics = ContributionMergeDiagnostics::default();
    let _ = insertion_cycle_ids(contributions, &mut diagnostics);
    let mut controls = BTreeSet::new();
    let mut toolbars = BTreeSet::new();
    let mut ordered: Vec<_> = contributions.iter().collect();
    ordered.sort_by(|a, b| a.mod_name.cmp(&b.mod_name));
    for loaded in ordered {
        for control in &loaded.contributes.controls {
            if !controls.insert(control.id.clone()) {
                diagnostics.messages.push(format!(
                    "duplicate control id {} from mod {} (skipped)",
                    control.id, loaded.mod_name
                ));
            }
        }
        for toolbar in &loaded.contributes.toolbars {
            if !toolbars.insert(toolbar.id.clone()) {
                diagnostics.messages.push(format!(
                    "duplicate toolbar id {} from mod {} (skipped)",
                    toolbar.id, loaded.mod_name
                ));
            }
        }
    }
    diagnostics.messages.sort();
    diagnostics.messages.dedup();
    diagnostics
}

pub fn merge_mod_compact_contributions(
    families: &mut [ResolvedFamily],
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
) {
    merge_mod_contributions_filtered(families, contributions, diagnostics, true);
}

pub fn merge_mod_contributions(
    families: &mut [ResolvedFamily],
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
) {
    merge_mod_contributions_filtered(families, contributions, diagnostics, false);
}

fn merge_mod_contributions_filtered(
    families: &mut [ResolvedFamily],
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
    compact_only: bool,
) {
    let mut seen_control_ids: HashSet<String> = HashSet::new();
    for family in families.iter() {
        for group in &family.groups {
            for control in &group.controls {
                seen_control_ids.insert(control.control_id.clone());
            }
        }
    }

    let cyclic = insertion_cycle_ids(contributions, diagnostics);
    let mut ordered: Vec<_> = contributions.iter().collect();
    ordered.sort_by(|a, b| a.mod_name.cmp(&b.mod_name));
    for loaded in ordered {
        let c = &loaded.contributes;
        let mut controls: Vec<_> = c.controls.iter().collect();
        controls.sort_by(|a, b| a.id.cmp(&b.id));
        for control in controls {
            let on_compact_surface = control
                .surfaces
                .iter()
                .any(|surface| surface == "text_compact_toolbar");
            if compact_only && !on_compact_surface {
                continue;
            }
            if !compact_only && on_compact_surface {
                continue;
            }
            if seen_control_ids.contains(&control.id) {
                diagnostics.messages.push(format!(
                    "duplicate control id {} from mod {} (skipped)",
                    control.id,
                    loaded.mod_name
                ));
                continue;
            }
            let mut effective = control.clone();
            if cyclic.contains(&effective.id) {
                effective.insert = Some(unibrowser_mod_api::InsertPoint {
                    after: None,
                    before: None,
                    end: true,
                });
            }
            insert_control(families, &effective, &loaded.mod_name, diagnostics);
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

fn insertion_cycle_ids(
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
) -> BTreeSet<String> {
    let controls: BTreeMap<String, ContributedControl> = contributions
        .iter()
        .flat_map(|loaded| loaded.contributes.controls.iter().cloned())
        .map(|control| (control.id.clone(), control))
        .collect();
    let mut edges: BTreeMap<String, BTreeSet<String>> = controls
        .keys()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect();
    let mut indegree: BTreeMap<String, usize> =
        controls.keys().map(|id| (id.clone(), 0)).collect();
    for control in controls.values() {
        let Some(insert) = &control.insert else {
            continue;
        };
        let edge = if let Some(after) = &insert.after {
            Some((after, &control.id))
        } else if let Some(before) = &insert.before {
            Some((&control.id, before))
        } else {
            None
        };
        let Some((from, to)) = edge else {
            continue;
        };
        if !controls.contains_key(from) || !controls.contains_key(to) {
            continue;
        }
        if edges.get_mut(from).expect("edge node").insert(to.clone()) {
            *indegree.get_mut(to).expect("indegree node") += 1;
        }
    }
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect();
    while let Some(id) = ready.pop_first() {
        for to in edges.get(&id).into_iter().flatten() {
            let degree = indegree.get_mut(to).expect("indegree edge");
            *degree -= 1;
            if *degree == 0 {
                ready.insert(to.clone());
            }
        }
    }
    let cyclic: BTreeSet<String> = indegree
        .into_iter()
        .filter(|(_, degree)| *degree > 0)
        .map(|(id, _)| id)
        .collect();
    if !cyclic.is_empty() {
        diagnostics.messages.push(format!(
            "order cycle [{}] moved to extension-point end",
            cyclic.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    cyclic
}

fn insert_control(
    families: &mut [ResolvedFamily],
    control: &ContributedControl,
    mod_name: &str,
    diagnostics: &mut ContributionMergeDiagnostics,
) {
    let targets_compact = control
        .surfaces
        .iter()
        .any(|surface| surface == "text_compact_toolbar");
    let family_id = if targets_compact {
        unibrowser_mod_api::family_ids::TEXT_COMPACT_TOOLBAR
    } else {
        unibrowser_mod_api::family_ids::TEXT_FORMATTING
    };
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
    let resolved = contributed_to_resolved(control, mod_name, &family.family_id);
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

fn contributed_control_type(raw: &str) -> ControlType {
    match raw {
        "toggle" => ControlType::Toggle,
        "split_button" => ControlType::SplitButton,
        "dropdown" => ControlType::Dropdown,
        "value_stepper" => ControlType::ValueStepper,
        "colour" => ControlType::Colour,
        "gallery" => ControlType::Gallery,
        "segmented" => ControlType::Segmented,
        "menu_button" => ControlType::MenuButton,
        "indicator_action" => ControlType::IndicatorAction,
        _ => ControlType::Button,
    }
}

pub fn proving_toolbar_in_default_layout() -> bool {
    std::env::var("UNIBROWSER_DEMO_STATE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("cmd-proving")
}

fn contributed_to_resolved(
    control: &ContributedControl,
    mod_name: &str,
    family_id: &str,
) -> ResolvedControl {
    let fallback_label = control
        .id
        .split('.')
        .next_back()
        .unwrap_or("Mod")
        .into();
    let label = control
        .label
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback_label);
    let icon = control
        .icon
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "extension".into());
    ResolvedControl {
        control_id: control.id.clone(),
        control_type: contributed_control_type(&control.control_type),
        command_ids: vec![control.command.clone()],
        capability: CapabilityState::Supported,
        selection: SelectionState::Off,
        value: None,
        label,
        short_label: None,
        icon,
        shortcut: None,
        density_variant: DensityVariant::Compact,
        provider_id: Some(format!("{}::native", mod_name)),
        context_revision: 0,
        group_id: format!("{}.group", mod_name),
        family_id: family_id.into(),
    }
}

pub fn merge_toolbar_layouts(
    contributions: &[LoadedModContribution],
    diagnostics: &mut ContributionMergeDiagnostics,
) -> Vec<crate::ui_command::FloatingToolbarLayout> {
    use crate::ui_command::{FloatingToolbarLayout, ToolbarAnchor, ToolbarCoordSpace, ToolbarPresentation};
    let mut out = Vec::new();
    let include_proving = proving_toolbar_in_default_layout();
    for loaded in contributions {
        for bar in &loaded.contributes.toolbars {
            if !bar.default_visible && !include_proving {
                continue;
            }
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
                anchor: ToolbarAnchor::Docked,
                x: 48.0,
                y: 120.0,
                coord_space: ToolbarCoordSpace::Screen,
                presentation: ToolbarPresentation::InWindow,
                monitor_name: None,
                monitor_edid: None,
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
            Some("brief.docx".into()),
            1,
        );
        let resolver = CapabilityResolver::default();
        core_tab_layout(&ctx, &resolver)
    }

    #[test]
    fn extends_text_family_with_mod_control() {
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
        let mut families = super::super::compact_toolbar::compact_text_toolbar_families(&ctx, &resolver);
        let mut diag = ContributionMergeDiagnostics::default();
        let contributions = vec![LoadedModContribution {
            mod_name: "cmd-proving".into(),
            contributes: ModContributes {
                controls: vec![ContributedControl {
                    id: "cmd-proving.control.chip".into(),
                    control_type: "button".into(),
                    command: "cmd-proving.subscript.apply".into(),
                    label: Some("Proving chip".into()),
                    icon: Some("extension".into()),
                    insert: Some(InsertPoint {
                        after: Some("unibrowser.control.text.link".into()),
                        before: None,
                        end: false,
                    }),
                    surfaces: vec!["text_compact_toolbar".into()],
                }],
                ..ModContributes::default()
            },
        }];
        merge_mod_compact_contributions(&mut families, &contributions, &mut diag);
        let all_controls = families
            .iter()
            .flat_map(|f| f.groups.iter())
            .flat_map(|g| g.controls.iter())
            .map(|c| c.control_id.as_str())
            .collect::<Vec<_>>();
        assert!(all_controls.iter().any(|id| *id == "unibrowser.control.text.link"));
        assert!(all_controls.iter().any(|id| *id == "cmd-proving.control.chip"));
    }

    #[test]
    fn order_cycle_has_sorted_diagnostic() {
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
        let mut families =
            super::super::compact_toolbar::compact_text_toolbar_families(&ctx, &resolver);
        let mut diag = ContributionMergeDiagnostics::default();
        let controls = vec![
            ContributedControl {
                id: "zmod.a".into(),
                control_type: "button".into(),
                command: "zmod.a".into(),
                label: None,
                icon: None,
                insert: Some(InsertPoint {
                    after: Some("zmod.b".into()),
                    before: None,
                    end: false,
                }),
                surfaces: vec!["text_compact_toolbar".into()],
            },
            ContributedControl {
                id: "zmod.b".into(),
                control_type: "button".into(),
                command: "zmod.b".into(),
                label: None,
                icon: None,
                insert: Some(InsertPoint {
                    after: Some("zmod.a".into()),
                    before: None,
                    end: false,
                }),
                surfaces: vec!["text_compact_toolbar".into()],
            },
        ];
        merge_mod_compact_contributions(
            &mut families,
            &[LoadedModContribution {
                mod_name: "zmod".into(),
                contributes: ModContributes {
                    controls,
                    ..Default::default()
                },
            }],
            &mut diag,
        );
        assert_eq!(
            diag.messages,
            vec!["order cycle [zmod.a, zmod.b] moved to extension-point end"]
        );
    }

    #[test]
    fn detachable_toolbar_defaults_to_in_window() {
        let mut diag = ContributionMergeDiagnostics::default();
        let bars = merge_toolbar_layouts(
            &[LoadedModContribution {
                mod_name: "cmd-proving".into(),
                contributes: ModContributes {
                    toolbars: vec![unibrowser_mod_api::ContributedToolbar {
                        id: "cmd-proving.detached-markdown".into(),
                        label: "Markdown proving".into(),
                        groups: vec![],
                        floatable: true,
                        detachable: true,
                        default_visible: true,
                        placement: None,
                    }],
                    ..ModContributes::default()
                },
            }],
            &mut diag,
        );
        assert_eq!(bars.len(), 1);
        assert_eq!(
            bars[0].presentation,
            crate::ui_command::ToolbarPresentation::InWindow
        );
        assert_eq!(bars[0].anchor, crate::ui_command::ToolbarAnchor::Docked);
    }

    #[test]
    fn proving_toolbar_hidden_on_fresh_profile() {
        std::env::remove_var("UNIBROWSER_DEMO_STATE");
        let mut diag = ContributionMergeDiagnostics::default();
        let bars = merge_toolbar_layouts(
            &[LoadedModContribution {
                mod_name: "cmd-proving".into(),
                contributes: ModContributes {
                    toolbars: vec![unibrowser_mod_api::ContributedToolbar {
                        id: "cmd-proving.detached-markdown".into(),
                        label: "Markdown proving".into(),
                        groups: vec!["cmd-proving.group.proving".into()],
                        floatable: true,
                        detachable: true,
                        default_visible: false,
                        placement: None,
                    }],
                    ..ModContributes::default()
                },
            }],
            &mut diag,
        );
        assert!(bars.is_empty());
    }
}
