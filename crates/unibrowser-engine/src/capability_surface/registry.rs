use unibrowser_mod_api::{
    CapabilityProvideEntry, ContentKindRegistry, ModManifest, validate_capability_provide,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceProviderRecord {
    pub mod_name: String,
    pub mod_title: String,
    pub capability_id: String,
    pub default: bool,
    pub priority: i32,
    pub content_kinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDefaultConflict {
    pub capability_id: String,
    pub mod_names: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurfaceProviderRegistry {
    pub providers: Vec<SurfaceProviderRecord>,
    pub diagnostics: Vec<String>,
    pub default_conflicts: Vec<CapabilityDefaultConflict>,
}

pub fn build_surface_provider_registry(mods: &[ModManifest]) -> SurfaceProviderRegistry {
    let kinds = ContentKindRegistry::with_core_kinds();
    let mut reg = SurfaceProviderRegistry::default();
    for manifest in mods {
        let entries = manifest
            .capability_provides
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let entries = if entries.is_empty() {
            synthetic_provides(&manifest.name)
        } else {
            entries
        };
        for entry in entries {
            if let Err(msg) = validate_capability_provide(&manifest.name, &entry, &kinds) {
                reg.diagnostics.push(format!("mod {}: {}", manifest.name, msg));
                continue;
            }
            reg.providers.push(SurfaceProviderRecord {
                mod_name: manifest.name.clone(),
                mod_title: manifest.title.clone(),
                capability_id: entry.capability.clone(),
                default: entry.default,
                priority: entry.priority,
                content_kinds: entry.content_kinds.clone(),
            });
        }
    }
    reg.providers.sort_by(|a, b| {
        a.capability_id
            .cmp(&b.capability_id)
            .then(b.priority.cmp(&a.priority))
            .then(a.mod_name.cmp(&b.mod_name))
    });
    detect_default_conflicts(&mut reg);
    reg
}

fn detect_default_conflicts(reg: &mut SurfaceProviderRegistry) {
    use std::collections::BTreeMap;
    let mut defaults: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in &reg.providers {
        if p.default {
            defaults
                .entry(p.capability_id.clone())
                .or_default()
                .push(p.mod_name.clone());
        }
    }
    for (capability_id, mods) in defaults {
        if mods.len() < 2 {
            continue;
        }
        reg.diagnostics.push(format!(
            "Capability {} has multiple default providers: {} (user must choose)",
            capability_id,
            mods.join(", ")
        ));
        reg.default_conflicts.push(CapabilityDefaultConflict {
            capability_id,
            mod_names: mods,
        });
    }
}

fn synthetic_provides(mod_name: &str) -> Vec<CapabilityProvideEntry> {
    use unibrowser_mod_api::capability_surface::ids;
    use unibrowser_mod_api::content_kind::ids as ck;
    let mk = |capability: &str, kinds: &[&str]| CapabilityProvideEntry {
        capability: capability.to_string(),
        default: true,
        priority: 100,
        content_kinds: kinds.iter().map(|k| (*k).to_string()).collect(),
    };
    match mod_name {
        "base-text" => vec![mk(ids::OPEN_TEXT, &[ck::TEXT_PLAIN, ck::TEXT_MARKDOWN])],
        "toy-markdown" => vec![CapabilityProvideEntry {
            capability: ids::OPEN_TEXT.into(),
            default: false,
            priority: 50,
            content_kinds: vec![ck::TEXT_MARKDOWN.into()],
        }],
        "base-hex" => vec![CapabilityProvideEntry {
            capability: ids::OPEN_TEXT.into(),
            default: false,
            priority: 10,
            content_kinds: vec![ck::TEXT_PLAIN.into()],
        }],
        "base-pdf" => vec![mk(ids::RENDER_PDF, &[ck::PDF_PAGE])],
        "base-png" => vec![mk(ids::RENDER_IMAGE, &[ck::IMAGE])],
        "base-svg" => vec![mk(ids::RENDER_SVG, &[ck::IMAGE])],
        "base-docx" => vec![mk(ids::OPEN_DOCX, &[ck::TEXT_RICH])],
        "base-dxf" => vec![mk(ids::OPEN_DXF, &[ck::VECTOR_CAD])],
        "base-video" => vec![mk(ids::OPEN_VIDEO, &[ck::VIDEO_CLIP])],
        "base-audio" => vec![mk(ids::OPEN_AUDIO, &[ck::AUDIO_CLIP])],
        "base-epub" => vec![mk(ids::OPEN_EPUB, &[ck::TEXT_RICH])],
        "base-csv" => vec![mk(ids::OPEN_CSV, &[ck::TABULAR])],
        "base-js" => vec![mk(ids::OPEN_SCRIPT, &[ck::TEXT_PLAIN])],
        "base-files" => vec![mk(ids::BROWSE_FILES, &[])],
        _ => Vec::new(),
    }
}
