use crate::history_file::{default_header, HistoryLogEntry};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PinHeader {
    pub format: u32,
    /// Key: `plugin_id@version`, value: op_type -> highest schema version seen.
    pub plugins: BTreeMap<String, BTreeMap<String, u32>>,
}

pub fn parse_pin_header(header: &str) -> PinHeader {
    let mut pin = PinHeader {
        format: 1,
        plugins: BTreeMap::new(),
    };
    let mut current_plugin: Option<String> = None;
    let mut in_plugins = false;
    let mut in_op_types = false;
    for line in header.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("historyFormat:") {
            pin.format = rest.trim().parse().unwrap_or(1);
            continue;
        }
        if trimmed == "plugins:" {
            in_plugins = true;
            in_op_types = false;
            current_plugin = None;
            continue;
        }
        if !in_plugins {
            continue;
        }
        if trimmed.ends_with(':') && !trimmed.contains(' ') {
            let key = trimmed.trim_end_matches(':').to_string();
            if key == "op_types" {
                in_op_types = true;
            } else {
                current_plugin = Some(key.clone());
                pin.plugins.entry(key).or_default();
                in_op_types = false;
            }
            continue;
        }
        if in_op_types {
            if let Some((op_type, ver)) = trimmed.split_once(':') {
                if let Some(plugin_key) = &current_plugin {
                    let schema = ver.trim().parse().unwrap_or(1);
                    pin.plugins
                        .entry(plugin_key.clone())
                        .or_default()
                        .insert(op_type.trim().to_string(), schema);
                }
            }
        }
    }
    pin
}

pub fn format_pin_header(pin: &PinHeader) -> String {
    let mut out = format!("historyFormat: {}\nplugins:\n", pin.format);
    for (plugin_key, op_types) in &pin.plugins {
        out.push_str(&format!("  {}:\n    op_types:\n", plugin_key));
        for (op_type, schema) in op_types {
            out.push_str(&format!("      {}: {}\n", op_type, schema));
        }
    }
    out
}

pub fn upsert_pin_from_entry(header: &str, entry: &HistoryLogEntry) -> String {
    let mut pin = parse_pin_header(header);
    if pin.plugins.is_empty() && header.trim().is_empty() {
        pin = parse_pin_header(&default_header());
    }
    let plugin_key = format!("{}@{}", entry.plugin_id, entry.plugin_version);
    let ops = pin.plugins.entry(plugin_key).or_default();
    let prior = ops.get(&entry.op_type).copied().unwrap_or(0);
    if entry.op_schema_version > prior {
        ops.insert(entry.op_type.clone(), entry.op_schema_version);
    } else if !ops.contains_key(&entry.op_type) {
        ops.insert(entry.op_type.clone(), entry.op_schema_version);
    }
    format_pin_header(&pin)
}

pub fn pinned_plugin_versions(header: &str) -> Vec<(String, String)> {
    parse_pin_header(header)
        .plugins
        .keys()
        .filter_map(|key| {
            let (id, ver) = key.split_once('@')?;
            Some((id.to_string(), ver.to_string()))
        })
        .collect()
}
