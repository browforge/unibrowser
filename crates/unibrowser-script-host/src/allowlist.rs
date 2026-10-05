//! Per-script command allowlist (ADR 0017).

/// Parse `// allow-commands: id1, id2` from the first lines of a script.
pub fn parse_allowlist_from_source(source: &str) -> Option<Vec<String>> {
    for line in source.lines().take(32) {
        let trimmed = line.trim();
        let rest = trimmed
            .strip_prefix("// allow-commands:")
            .or_else(|| trimmed.strip_prefix("//allow-commands:"));
        if let Some(rest) = rest {
            let ids: Vec<String> = rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !ids.is_empty() {
                return Some(ids);
            }
        }
    }
    None
}

pub fn filter_registry_ids(registry_ids: &[String], allowlist: Option<&[String]>) -> Vec<String> {
    match allowlist {
        None => registry_ids.to_vec(),
        Some(allowed) => registry_ids
            .iter()
            .filter(|id| allowed.iter().any(|a| a == *id))
            .cloned()
            .collect(),
    }
}
