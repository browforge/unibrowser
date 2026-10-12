//! Fixed-slot panel, view, status, and service contributions (CMD-15).

use serde::{Deserialize, Serialize};

use unibrowser_mod_api::{
    ContributedPanel, ContributedService, ContributedStatusItem, ContributedView,
};

use super::contribution_merge::LoadedModContribution;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedPanelContribution {
    pub mod_name: String,
    pub panel: ContributedPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedViewContribution {
    pub mod_name: String,
    pub view: ContributedView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedStatusContribution {
    pub mod_name: String,
    pub item: ContributedStatusItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedServiceContribution {
    pub mod_name: String,
    pub service: ContributedService,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellContributionSnapshot {
    pub panels: Vec<ResolvedPanelContribution>,
    pub views: Vec<ResolvedViewContribution>,
    pub status_items: Vec<ResolvedStatusContribution>,
    pub services: Vec<ResolvedServiceContribution>,
}

pub fn merge_shell_contributions(
    contributions: &[LoadedModContribution],
) -> ShellContributionSnapshot {
    let mut snapshot = ShellContributionSnapshot::default();
    let mut ordered: Vec<_> = contributions.iter().collect();
    ordered.sort_by(|a, b| a.mod_name.cmp(&b.mod_name));
    for loaded in ordered {
        for panel in &loaded.contributes.panels {
            if snapshot.panels.iter().any(|p| p.panel.id == panel.id) {
                continue;
            }
            snapshot.panels.push(ResolvedPanelContribution {
                mod_name: loaded.mod_name.clone(),
                panel: panel.clone(),
            });
        }
        for view in &loaded.contributes.views {
            if snapshot.views.iter().any(|v| v.view.id == view.id) {
                continue;
            }
            snapshot.views.push(ResolvedViewContribution {
                mod_name: loaded.mod_name.clone(),
                view: view.clone(),
            });
        }
        for item in &loaded.contributes.status_items {
            if snapshot.status_items.iter().any(|s| s.item.id == item.id) {
                continue;
            }
            snapshot.status_items.push(ResolvedStatusContribution {
                mod_name: loaded.mod_name.clone(),
                item: item.clone(),
            });
        }
        for service in &loaded.contributes.services {
            if snapshot.services.iter().any(|s| s.service.id == service.id) {
                continue;
            }
            snapshot.services.push(ResolvedServiceContribution {
                mod_name: loaded.mod_name.clone(),
                service: service.clone(),
            });
        }
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_mod_api::ModContributes;

    #[test]
    fn proving_mod_panel_and_status_merge() {
        let contributions = vec![LoadedModContribution {
            mod_name: "acme-tools".into(),
            contributes: ModContributes {
                panels: vec![ContributedPanel {
                    id: "acme-tools.panel.hint".into(),
                    region: "left".into(),
                    title: "Tools".into(),
                    content_kinds: vec![],
                }],
                status_items: vec![ContributedStatusItem {
                    id: "acme-tools.status.ready".into(),
                    slot: "right".into(),
                    priority: 5,
                    label: "Ready".into(),
                    content_kinds: vec![],
                }],
                ..ModContributes::default()
            },
        }];
        let snap = merge_shell_contributions(&contributions);
        assert_eq!(snap.panels.len(), 1);
        assert_eq!(snap.status_items.len(), 1);
    }
}
