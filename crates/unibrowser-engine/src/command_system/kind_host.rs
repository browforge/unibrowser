//! Per content-kind × host-format capability matrix (CMD-14).

use std::collections::HashMap;

use unibrowser_mod_api::{
    CapabilityState, ContributedCapabilityProvider, ContributedHostFormatCapability,
};

use super::context::ResolutionContext;
use crate::command_system::contribution_merge::LoadedModContribution;

#[derive(Debug, Clone)]
struct ProviderMatrix {
    provider_id: String,
    families: Vec<String>,
    content_kinds: Vec<String>,
    host_formats: Vec<String>,
    format_states: HashMap<String, ContributedHostFormatCapability>,
}

#[derive(Debug, Clone, Default)]
pub struct KindHostCapabilityMatrix {
    by_provider: HashMap<String, ProviderMatrix>,
    control_families: HashMap<String, String>,
}

impl KindHostCapabilityMatrix {
    pub fn from_contributions(contributions: &[LoadedModContribution]) -> Self {
        let mut by_provider = HashMap::new();
        let mut control_families = HashMap::new();
        for loaded in contributions {
            for provider in &loaded.contributes.capability_providers {
                let format_states = provider
                    .format_states
                    .iter()
                    .map(|s| (s.host_format.to_ascii_lowercase(), s.clone()))
                    .collect();
                by_provider.insert(
                    provider.id.clone(),
                    ProviderMatrix {
                        provider_id: provider.id.clone(),
                        families: provider.families.clone(),
                        content_kinds: provider.content_kinds.clone(),
                        host_formats: provider
                            .host_formats
                            .iter()
                            .map(|h| h.to_ascii_lowercase())
                            .collect(),
                        format_states,
                    },
                );
            }
            for family in &loaded.contributes.families {
                for control in &family.controls {
                    control_families.insert(control.id.clone(), family.id.clone());
                }
            }
            for control in &loaded.contributes.controls {
                control_families.insert(
                    control.id.clone(),
                    "unibrowser.family.text-formatting".into(),
                );
            }
        }
        Self {
            by_provider,
            control_families,
        }
    }

    pub fn resolve_control(
        &self,
        control_id: &str,
        provider_id: Option<&str>,
        family_id: &str,
        ctx: &ResolutionContext,
    ) -> Option<CapabilityState> {
        let provider_id = provider_id?;
        let matrix = self.by_provider.get(provider_id)?;
        if !matrix.families.is_empty() && !matrix.families.iter().any(|f| f == family_id) {
            return None;
        }
        if !matrix.content_kinds.is_empty()
            && !matrix
                .content_kinds
                .iter()
                .any(|k| ctx.selection_content_kinds.iter().any(|s| s == k))
        {
            return Some(CapabilityState::Unsupported {
                reason: "Select content that matches this tool's kind.".into(),
            });
        }
        let host = ctx.host_format_id.to_ascii_lowercase();
        if let Some(rule) = matrix.format_states.get(&host) {
            return Some(capability_from_rule(rule, &matrix.provider_id));
        }
        if !matrix.host_formats.is_empty() && !matrix.host_formats.iter().any(|h| h == &host) {
            return Some(CapabilityState::Unsupported {
                reason: format!(
                    "Not available for .{} host documents",
                    host
                ),
            });
        }
        None
    }
}

fn capability_from_rule(
    rule: &ContributedHostFormatCapability,
    provider_id: &str,
) -> CapabilityState {
    match rule.state.as_str() {
        "supported" => CapabilityState::Supported,
        "emulated" => CapabilityState::Emulated {
            provider_id: provider_id.to_string(),
            note: rule
                .note
                .clone()
                .unwrap_or_else(|| "Approximated for this host format.".into()),
        },
        "unsupported" => CapabilityState::Unsupported {
            reason: rule
                .reason
                .clone()
                .unwrap_or_else(|| "Not supported for this host format.".into()),
        },
        other => CapabilityState::Unsupported {
            reason: format!("Unknown capability state {}", other),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_mod_api::{ContributedControl, ContributedFamily, ModContributes};
    use unibrowser_mod_api::content_kind::ids as ck;

    fn sample_ctx(host: &str, kinds: Vec<&str>) -> ResolutionContext {
        ResolutionContext {
            work: unibrowser_mod_api::WorkContext::Image,
            command_context: unibrowser_mod_api::CommandContext::Image,
            host: unibrowser_mod_api::HostContext {
                mode: unibrowser_mod_api::Mode::Image,
                doc_type: "image".into(),
                backend_id: "base-png".into(),
                selection: Default::default(),
                capabilities: Default::default(),
            },
            doc_path: Some("photo.png".into()),
            revision: 1,
            markdown_inline_html: true,
            host_format_id: host.into(),
            selection_content_kinds: kinds.into_iter().map(String::from).collect(),
            selection_node_paths: vec!["embedded-image-1".into()],
            content_tree_revision: 0,
        }
    }

    #[test]
    fn png_supported_markdown_emulated() {
        let contributions = vec![LoadedModContribution {
            mod_name: "image-grayscale-preview".into(),
            contributes: ModContributes {
                families: vec![ContributedFamily {
                    id: "unibrowser.family.image-adjust".into(),
                    extend: true,
                    controls: vec![ContributedControl {
                        id: "image-grayscale-preview.control.grayscale".into(),
                        control_type: "toggle".into(),
                        command: "image-grayscale-preview.grayscale".into(),
                        label: Some("Grayscale preview".into()),
                        icon: None,
                        insert: None,
                    }],
                }],
                capability_providers: vec![ContributedCapabilityProvider {
                    id: "image-grayscale-preview::native".into(),
                    families: vec!["unibrowser.family.image-adjust".into()],
                    contexts: vec![],
                    content_kinds: vec![ck::IMAGE.into()],
                    host_formats: vec!["png".into(), "docx".into(), "md".into()],
                    format_states: vec![
                        ContributedHostFormatCapability {
                            host_format: "png".into(),
                            state: "supported".into(),
                            note: None,
                            reason: None,
                        },
                        ContributedHostFormatCapability {
                            host_format: "docx".into(),
                            state: "emulated".into(),
                            note: Some("Preview only; DOCX stores rich text.".into()),
                            reason: None,
                        },
                    ],
                    runtime: "native".into(),
                    priority: 10,
                }],
                ..ModContributes::default()
            },
        }];
        let matrix = KindHostCapabilityMatrix::from_contributions(&contributions);
        let ctx = sample_ctx("png", vec![ck::IMAGE]);
        let cap = matrix
            .resolve_control(
                "image-grayscale-preview.control.grayscale",
                Some("image-grayscale-preview::native"),
                "unibrowser.family.image-adjust",
                &ctx,
            )
            .expect("cap");
        assert!(matches!(cap, CapabilityState::Supported));
        let ctx_docx = sample_ctx("docx", vec![ck::IMAGE]);
        let cap_docx = matrix
            .resolve_control(
                "image-grayscale-preview.control.grayscale",
                Some("image-grayscale-preview::native"),
                "unibrowser.family.image-adjust",
                &ctx_docx,
            )
            .expect("cap");
        assert!(matches!(cap_docx, CapabilityState::Emulated { .. }));
    }
}
