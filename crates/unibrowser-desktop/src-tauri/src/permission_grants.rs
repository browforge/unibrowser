//! Process-wide ADR 0029 grant store (see `permission-grants.json`).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use unibrowser_mod_api::{
    GrantChoice, ModPermissionsBlock, PermissionGrantStore,
};

static STORE: OnceLock<Mutex<PermissionGrantStore>> = OnceLock::new();

fn store_path() -> Option<PathBuf> {
    crate::config::permission_grants_file_path()
}

pub fn grant_store() -> &'static Mutex<PermissionGrantStore> {
    STORE.get_or_init(|| {
        let store = match store_path() {
            Some(path) => PermissionGrantStore::load(path).unwrap_or_else(|_| {
                PermissionGrantStore::in_memory()
            }),
            None => PermissionGrantStore::in_memory(),
        };
        Mutex::new(store)
    })
}

pub fn sync_plugin_permissions(
    plugin_id: &str,
    permissions: Option<&ModPermissionsBlock>,
) -> Result<(), String> {
    grant_store()
        .lock()
        .map_err(|_| "grant store lock poisoned".to_string())?
        .sync_plugin_permissions(plugin_id, permissions)
}

pub fn record_grant(
    plugin_id: &str,
    capability: &str,
    host: &str,
    choice: GrantChoice,
    permissions_fingerprint: &str,
) -> Result<(), String> {
    grant_store()
        .lock()
        .map_err(|_| "grant store lock poisoned".to_string())?
        .record_grant(plugin_id, capability, host, choice, permissions_fingerprint)
}

pub fn revoke_always(plugin_id: &str, capability: &str, host: &str) -> Result<(), String> {
    grant_store()
        .lock()
        .map_err(|_| "grant store lock poisoned".to_string())?
        .revoke_always(plugin_id, capability, host)
}

pub fn is_network_allowed(plugin_id: &str, host: &str) -> bool {
    grant_store()
        .lock()
        .ok()
        .map(|s| s.is_allowed(plugin_id, "network", host))
        .unwrap_or(false)
}
