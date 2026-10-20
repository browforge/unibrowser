//! ADR 0029 permission manifest block (`permission_schema`).

use serde::{Deserialize, Serialize};

use crate::content_kind::validate_content_kind_id;

pub const PERMISSION_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModPermissionsBlock {
    pub permission_schema: String,
    #[serde(default)]
    pub network: Option<NetworkPermissions>,
    #[serde(default)]
    pub filesystem: Option<FilesystemPermissions>,
    #[serde(default)]
    pub clipboard: Option<ClipboardPermission>,
    #[serde(default)]
    pub process: Option<ProcessPermissions>,
    #[serde(default)]
    pub content: Option<ContentPermissions>,
    #[serde(default)]
    pub secrets: Vec<String>,
    #[serde(default)]
    pub background: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkPermissions {
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub disclosure: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesystemPermissions {
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardPermission {
    None,
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessPermissions {
    #[serde(default, rename = "launch")]
    pub launch: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentPermissions {
    #[serde(default, rename = "read")]
    pub read: Vec<String>,
    #[serde(default, rename = "write")]
    pub write: Vec<String>,
}

const FS_SCOPES: &[&str] = &["plugin_data", "user_pick", "workspace_roots", "downloads"];

/// Stable fingerprint for grant reset when declared permissions change.
pub fn permissions_fingerprint(block: &ModPermissionsBlock) -> String {
    serde_json::to_string(block).unwrap_or_default()
}

pub fn validate_permissions(mod_name: &str, block: &ModPermissionsBlock) -> Result<(), String> {
    if block.permission_schema != PERMISSION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported permission_schema `{}` (host supports {})",
            block.permission_schema,
            PERMISSION_SCHEMA_VERSION
        ));
    }

    if let Some(net) = &block.network {
        let has_domains = !net.domains.is_empty();
        if has_domains {
            if net.reason.trim().is_empty() {
                return Err("permissions.network.reason is required when domains are declared".into());
            }
            if net.disclosure.is_empty() {
                return Err(
                    "permissions.network.disclosure must list at least one bullet when domains are declared"
                        .into(),
                );
            }
        }
        for domain in &net.domains {
            validate_network_host_entry(domain)?;
        }
    }

    if let Some(fs) = &block.filesystem {
        for scope in &fs.scopes {
            if !FS_SCOPES.contains(&scope.as_str()) {
                return Err(format!(
                    "permissions.filesystem.scopes contains unknown scope `{}`",
                    scope
                ));
            }
        }
    }

    if let Some(content) = &block.content {
        for kind in content.read.iter().chain(content.write.iter()) {
            validate_content_kind_id(kind, Some(mod_name))?;
        }
    }

    for slot in &block.secrets {
        if slot.trim().is_empty() {
            return Err("permissions.secrets entries must be non-empty names".into());
        }
        if slot.contains('/') || slot.contains('\\') {
            return Err(format!("permissions.secrets slot `{}` must be a simple name", slot));
        }
    }

    if let Some(proc) = &block.process {
        for entry in &proc.launch {
            if entry.trim().is_empty() {
                return Err("permissions.process.launch entries must be non-empty".into());
            }
        }
    }

    Ok(())
}

/// Accepts `host`, `host:port`, `https://host/path`, or `*.example.com` wildcards.
pub fn validate_network_host_entry(entry: &str) -> Result<(), String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err("network domain entry must not be empty".into());
    }
    if trimmed == "*" || trimmed == "*.*" || trimmed == "https://*" || trimmed == "http://*" {
        return Err(format!("network domain `{}` is too broad", entry));
    }

    let host = extract_host_for_validation(trimmed)?;
    validate_host_pattern(&host, entry)?;
    Ok(())
}

fn extract_host_for_validation(entry: &str) -> Result<String, String> {
    if entry.contains("://") {
        let after_scheme = entry
            .split("://")
            .nth(1)
            .ok_or_else(|| format!("invalid network origin `{}`", entry))?;
        let authority = after_scheme.split('/').next().unwrap_or(after_scheme);
        if authority.is_empty() {
            return Err(format!("invalid network origin `{}`", entry));
        }
        let host_port = authority.split('@').next_back().unwrap_or(authority);
        let host = host_port
            .rsplit_once(':')
            .filter(|(_, port)| port.chars().all(|c| c.is_ascii_digit()))
            .map(|(h, _)| h)
            .unwrap_or(host_port);
        if host.starts_with('[') && host.ends_with(']') {
            return Ok(host[1..host.len() - 1].to_string());
        }
        return Ok(host.to_string());
    }
    let host_port = entry.split('/').next().unwrap_or(entry);
    let host = host_port
        .rsplit_once(':')
        .filter(|(_, port)| port.chars().all(|c| c.is_ascii_digit()))
        .map(|(h, _)| h)
        .unwrap_or(host_port);
    Ok(host.to_string())
}

fn validate_host_pattern(host: &str, original: &str) -> Result<(), String> {
    if host.is_empty() {
        return Err(format!("invalid network host in `{}`", original));
    }
    if host == "*" {
        return Err(format!("network domain `{}` is too broad", original));
    }

    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() == 1 && !host.eq_ignore_ascii_case("localhost") {
        if host.starts_with("*.") || host == "*" {
            return Err(format!("network domain `{}` is too broad", original));
        }
        return Err(format!(
            "network host `{}` looks like a bare TLD or label; use a fully qualified host or *.domain wildcard",
            host
        ));
    }

    if host.contains('*') {
        if !host.starts_with("*.") {
            return Err(format!(
                "network wildcard `{}` must be a single `*.domain` pattern",
                original
            ));
        }
        let rest = host.strip_prefix("*.").unwrap_or("");
        if rest.is_empty() || rest.contains('*') {
            return Err(format!("network wildcard `{}` is invalid", original));
        }
        let rest_labels: Vec<&str> = rest.split('.').collect();
        if rest_labels.len() < 2 {
            return Err(format!(
                "network wildcard `{}` must include a registrable domain (e.g. *.cdn.example.com)",
                original
            ));
        }
        for label in rest_labels {
            if label.is_empty() {
                return Err(format!("network wildcard `{}` has empty label", original));
            }
        }
    } else {
        for label in labels {
            if label.is_empty() {
                return Err(format!("network host `{}` has empty label", host));
            }
            if label == "*" {
                return Err(format!("network domain `{}` is too broad", original));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_with_domains(domains: Vec<&str>) -> ModPermissionsBlock {
        ModPermissionsBlock {
            permission_schema: PERMISSION_SCHEMA_VERSION.into(),
            network: Some(NetworkPermissions {
                domains: domains.into_iter().map(String::from).collect(),
                reason: "fixture".into(),
                disclosure: vec!["test payload".into()],
            }),
            ..Default::default()
        }
    }

    #[test]
    fn accepts_exact_host_and_subdomain_wildcard() {
        validate_network_host_entry("api.example.com").expect("host");
        validate_network_host_entry("https://api.example.com:443/path").expect("origin");
        validate_network_host_entry("*.cdn.example.com").expect("wildcard");
    }

    #[test]
    fn rejects_star_and_bare_tld() {
        assert!(validate_network_host_entry("*").is_err());
        assert!(validate_network_host_entry("com").is_err());
        assert!(validate_network_host_entry("https://*").is_err());
    }

    #[test]
    fn invalid_wildcard_domain_fails_validation() {
        let block = block_with_domains(vec!["*"]);
        let err = validate_permissions("fixture", &block).unwrap_err();
        assert!(err.contains("broad") || err.contains("wildcard"), "{}", err);
    }

    #[test]
    fn requires_network_reason_when_domains_present() {
        let block = ModPermissionsBlock {
            permission_schema: PERMISSION_SCHEMA_VERSION.into(),
            network: Some(NetworkPermissions {
                domains: vec!["api.example.com".into()],
                reason: String::new(),
                disclosure: vec!["x".into()],
            }),
            ..Default::default()
        };
        assert!(validate_permissions("fixture", &block).is_err());
    }
}
