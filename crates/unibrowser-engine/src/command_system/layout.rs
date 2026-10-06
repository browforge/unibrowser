use std::collections::BTreeMap;
use std::path::Path;

use unibrowser_mod_api::{
    CommandLayoutFileV2, ContextLayout, HiddenControlEntry, LAYOUT_VERSION_V2, MonitorAnchor,
};

use crate::ui_command::CommandManagerFile;

#[derive(Debug, Clone, Default)]
pub struct LayoutStore {
    pub v2: CommandLayoutFileV2,
}

impl LayoutStore {
    pub fn load(path: Option<&Path>, legacy: &CommandManagerFile) -> Self {
        let mut store = LayoutStore {
            v2: CommandLayoutFileV2 {
                layout_version: LAYOUT_VERSION_V2,
                contexts: BTreeMap::new(),
                orphaned_contributions: vec![],
            },
        };
        if let Some(path) = path {
            let v2_path = path.with_extension("layout.json");
            if let Ok(data) = std::fs::read_to_string(&v2_path) {
                if let Ok(file) = serde_json::from_str::<CommandLayoutFileV2>(&data) {
                    store.v2 = file;
                }
            }
        }
        if store.v2.contexts.is_empty() {
            store.v2 = migrate_toolbars_json(legacy);
        }
        store
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let v2_path = path.with_extension("layout.json");
        if let Some(parent) = v2_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(&self.v2).map_err(|e| e.to_string())?;
        std::fs::write(v2_path, data).map_err(|e| e.to_string())
    }

    pub fn hide_control(&mut self, context_id: &str, control_id: &str, all_contexts: bool) {
        let ctx = self
            .v2
            .contexts
            .entry(context_id.to_string())
            .or_insert_with(ContextLayout::default);
        ctx.hidden_controls.push(HiddenControlEntry {
            control_id: control_id.to_string(),
            all_contexts,
        });
    }

    pub fn set_provider_override(&mut self, context_id: &str, control_id: &str, provider: &str) {
        let ctx = self
            .v2
            .contexts
            .entry(context_id.to_string())
            .or_insert_with(ContextLayout::default);
        ctx.provider_overrides
            .insert(control_id.to_string(), provider.to_string());
    }

    pub fn hidden_for_context(&self, context_id: &str) -> Vec<String> {
        let mut hidden = Vec::new();
        for (ctx_id, layout) in &self.v2.contexts {
            for entry in &layout.hidden_controls {
                if entry.all_contexts || ctx_id == context_id {
                    hidden.push(entry.control_id.clone());
                }
            }
        }
        hidden
    }

    pub fn provider_overrides_for(&self, context_id: &str) -> BTreeMap<String, String> {
        self.v2
            .contexts
            .get(context_id)
            .map(|c| c.provider_overrides.clone())
            .unwrap_or_default()
    }
}

pub fn migrate_toolbars_json(legacy: &CommandManagerFile) -> CommandLayoutFileV2 {
    CommandLayoutFileV2 {
        layout_version: LAYOUT_VERSION_V2,
        contexts: BTreeMap::from([(
            legacy.selected_tab.clone(),
            ContextLayout {
                family_order: vec![],
                hidden_controls: vec![],
                provider_overrides: BTreeMap::new(),
            },
        )]),
        orphaned_contributions: vec![],
    }
}

pub fn nearest_monitor_anchor(
    anchors: &[MonitorAnchor],
    work_x: f64,
    work_y: f64,
    work_w: f64,
    work_h: f64,
) -> MonitorAnchor {
    if let Some(first) = anchors.first() {
        return first.clone();
    }
    MonitorAnchor {
        monitor_name: "primary".into(),
        monitor_edid: None,
        x: work_x,
        y: work_y,
        width: work_w,
        height: work_h,
        scale: 1.0,
    }
}
