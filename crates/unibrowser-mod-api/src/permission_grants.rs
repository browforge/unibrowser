//! ADR 0029 grant store (`permission-grants.json`).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::permissions::ModPermissionsBlock;

pub const GRANT_STORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantChoice {
    Once,
    Always,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrantRecord {
    pub host: String,
    pub choice: GrantChoice,
    pub granted_at_ms: u64,
    pub permissions_fingerprint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginGrantState {
    #[serde(default)]
    pub capabilities: BTreeMap<String, Vec<GrantRecord>>,
    #[serde(default)]
    pub permissions_fingerprint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrantStoreFile {
    pub version: u32,
    #[serde(default)]
    pub plugins: BTreeMap<String, PluginGrantState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GrantKey {
    pub plugin_id: String,
    pub capability: String,
    pub host: String,
}

#[derive(Debug, Default)]
pub struct PermissionGrantStore {
    path: Option<PathBuf>,
    file: GrantStoreFile,
    session_once: HashMap<GrantKey, GrantRecord>,
}

impl PermissionGrantStore {
    pub fn in_memory() -> Self {
        Self {
            path: None,
            file: GrantStoreFile {
                version: GRANT_STORE_VERSION,
                plugins: BTreeMap::new(),
            },
            session_once: HashMap::new(),
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, String> {
        let file = if path.is_file() {
            let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&data).map_err(|e| e.to_string())?
        } else {
            GrantStoreFile {
                version: GRANT_STORE_VERSION,
                plugins: BTreeMap::new(),
            }
        };
        Ok(Self {
            path: Some(path),
            file,
            session_once: HashMap::new(),
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn sync_plugin_permissions(
        &mut self,
        plugin_id: &str,
        permissions: Option<&ModPermissionsBlock>,
    ) -> Result<(), String> {
        let fingerprint = permissions
            .map(crate::permissions::permissions_fingerprint)
            .unwrap_or_default();
        let entry = self.file.plugins.entry(plugin_id.to_string()).or_default();
        if entry.permissions_fingerprint != fingerprint {
            entry.capabilities.clear();
            entry.permissions_fingerprint = fingerprint;
            self.session_once
                .retain(|k, _| k.plugin_id != plugin_id);
            self.persist()?;
        }
        Ok(())
    }

    pub fn record_grant(
        &mut self,
        plugin_id: &str,
        capability: &str,
        host: &str,
        choice: GrantChoice,
        permissions_fingerprint: &str,
    ) -> Result<(), String> {
        let key = GrantKey {
            plugin_id: plugin_id.to_string(),
            capability: capability.to_string(),
            host: host.to_string(),
        };
        let record = GrantRecord {
            host: host.to_string(),
            choice,
            granted_at_ms: now_ms(),
            permissions_fingerprint: permissions_fingerprint.to_string(),
        };

        match choice {
            GrantChoice::Once => {
                self.session_once.insert(key, record);
            }
            GrantChoice::Always | GrantChoice::Deny => {
                self.session_once.remove(&key);
                let plugin = self
                    .file
                    .plugins
                    .entry(plugin_id.to_string())
                    .or_default();
                if plugin.permissions_fingerprint.is_empty() {
                    plugin.permissions_fingerprint = permissions_fingerprint.to_string();
                }
                let list = plugin
                    .capabilities
                    .entry(capability.to_string())
                    .or_default();
                if let Some(idx) = list.iter().position(|r| r.host == host) {
                    list[idx] = record;
                } else {
                    list.push(record);
                }
                self.persist()?;
            }
        }
        Ok(())
    }

    pub fn revoke_always(
        &mut self,
        plugin_id: &str,
        capability: &str,
        host: &str,
    ) -> Result<(), String> {
        let key = GrantKey {
            plugin_id: plugin_id.to_string(),
            capability: capability.to_string(),
            host: host.to_string(),
        };
        self.session_once.remove(&key);
        if let Some(plugin) = self.file.plugins.get_mut(plugin_id) {
            if let Some(list) = plugin.capabilities.get_mut(capability) {
                list.retain(|r| r.host != host);
                if list.is_empty() {
                    plugin.capabilities.remove(capability);
                }
            }
        }
        self.persist()?;
        Ok(())
    }

    pub fn effective_choice(
        &self,
        plugin_id: &str,
        capability: &str,
        host: &str,
    ) -> Option<GrantChoice> {
        let key = GrantKey {
            plugin_id: plugin_id.to_string(),
            capability: capability.to_string(),
            host: host.to_string(),
        };
        if let Some(rec) = self.session_once.get(&key) {
            return Some(rec.choice);
        }
        self.file
            .plugins
            .get(plugin_id)
            .and_then(|p| p.capabilities.get(capability))
            .and_then(|list| list.iter().find(|r| r.host == host))
            .map(|r| r.choice)
    }

    pub fn is_allowed(&self, plugin_id: &str, capability: &str, host: &str) -> bool {
        matches!(
            self.effective_choice(plugin_id, capability, host),
            Some(GrantChoice::Once) | Some(GrantChoice::Always)
        )
    }

    fn persist(&mut self) -> Result<(), String> {
        let path = self.path.as_ref().ok_or("grant store has no path")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(&self.file).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        {
            let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
            f.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
        }
        fs::rename(&tmp, path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{
        permissions_fingerprint, ModPermissionsBlock, NetworkPermissions, PERMISSION_SCHEMA_VERSION,
    };

    fn temp_store() -> (std::path::PathBuf, PermissionGrantStore) {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ub-perm-grants-{}-{}.json",
            std::process::id(),
            n
        ));
        let store = PermissionGrantStore::load(path.clone()).expect("load");
        (path, store)
    }

    fn sample_permissions(domains: &[&str]) -> ModPermissionsBlock {
        ModPermissionsBlock {
            permission_schema: PERMISSION_SCHEMA_VERSION.into(),
            network: Some(NetworkPermissions {
                domains: domains.iter().map(|d| (*d).to_string()).collect(),
                reason: "test".into(),
                disclosure: vec!["payload".into()],
            }),
            ..Default::default()
        }
    }

    #[test]
    fn always_grant_survives_reload() {
        let (path, mut store) = temp_store();
        let fp = permissions_fingerprint(&sample_permissions(&["api.example.com"]));
        store
            .record_grant("acme", "network", "api.example.com", GrantChoice::Always, &fp)
            .expect("grant");
        assert!(store.is_allowed("acme", "network", "api.example.com"));

        let reloaded = PermissionGrantStore::load(path.clone()).expect("reload");
        assert!(reloaded.is_allowed("acme", "network", "api.example.com"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn once_grant_does_not_persist() {
        let (path, mut store) = temp_store();
        let fp = permissions_fingerprint(&sample_permissions(&["api.example.com"]));
        store
            .record_grant("acme", "network", "api.example.com", GrantChoice::Once, &fp)
            .expect("grant");
        assert!(store.is_allowed("acme", "network", "api.example.com"));

        let reloaded = PermissionGrantStore::load(path.clone()).expect("reload");
        assert!(!reloaded.is_allowed("acme", "network", "api.example.com"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn fingerprint_change_clears_persistent_grants() {
        let (path, mut store) = temp_store();
        let v1 = sample_permissions(&["api.example.com"]);
        let fp1 = permissions_fingerprint(&v1);
        store.sync_plugin_permissions("acme", Some(&v1)).expect("sync");
        store
            .record_grant("acme", "network", "api.example.com", GrantChoice::Always, &fp1)
            .expect("grant");

        let v2 = sample_permissions(&["api.example.com", "cdn.example.com"]);
        store.sync_plugin_permissions("acme", Some(&v2)).expect("sync v2");
        assert!(!store.is_allowed("acme", "network", "api.example.com"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn revoke_always_takes_effect_immediately() {
        let (_path, mut store) = temp_store();
        let fp = permissions_fingerprint(&sample_permissions(&["api.example.com"]));
        store
            .record_grant("acme", "network", "api.example.com", GrantChoice::Always, &fp)
            .expect("grant");
        assert!(store.is_allowed("acme", "network", "api.example.com"));
        store
            .revoke_always("acme", "network", "api.example.com")
            .expect("revoke");
        assert!(!store.is_allowed("acme", "network", "api.example.com"));
    }
}
