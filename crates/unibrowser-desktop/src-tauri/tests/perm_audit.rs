//! PERM-02..07 audit probes for desktop host (ADR 0029).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use unibrowser_mod_api::{
    permissions_fingerprint, GrantChoice, ModPermissionsBlock, NetworkPermissions,
    PermissionGrantStore, PERMISSION_SCHEMA_VERSION,
};

#[test]
fn perm02_permission_grants_persistence_path() {
    let config = include_str!("../src/config.rs");
    assert!(
        config.contains("permission-grants"),
        "PERM-02 grant store file must be wired in config"
    );
}

#[test]
fn perm06_no_network_activity_log_path() {
    let config = include_str!("../src/config.rs");
    assert!(
        !config.contains("network-activity"),
        "PERM-06 activity log path not wired in config"
    );
}

#[test]
fn perm07_no_plugin_secrets_host_commands() {
    let lib = include_str!("../src/lib.rs");
    assert!(
        !lib.contains("secrets_put") && !lib.contains("plugin_secrets"),
        "PERM-07 keychain secrets API not registered"
    );
}

#[test]
fn perm10_vsix_trust_still_separate_from_grant_store() {
    let prefs = include_str!("../src/preferences.rs");
    assert!(
        prefs.contains("vsix_trusted_extensions"),
        "VSIX trust list should exist until PERM-10 aligns with grants"
    );
    let config = include_str!("../src/config.rs");
    assert!(config.contains("permission-grants"));
}

#[test]
fn perm02_revoke_always_grant_without_restart() {
    let path = std::env::temp_dir().join(format!(
        "ub-perm02-revoke-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_file(&path);

    static TEST_STORE: OnceLock<Mutex<PermissionGrantStore>> = OnceLock::new();
    let store = TEST_STORE.get_or_init(|| {
        Mutex::new(PermissionGrantStore::load(path.clone()).expect("open grant store"))
    });

    let perms = ModPermissionsBlock {
        permission_schema: PERMISSION_SCHEMA_VERSION.into(),
        network: Some(NetworkPermissions {
            domains: vec!["api.example.com".into()],
            reason: "fixture".into(),
            disclosure: vec!["bytes".into()],
        }),
        ..Default::default()
    };
    let fp = permissions_fingerprint(&perms);

    {
        let mut guard = store.lock().expect("lock");
        guard
            .record_grant("fixture-mod", "network", "api.example.com", GrantChoice::Always, &fp)
            .expect("grant");
        assert!(guard.is_allowed("fixture-mod", "network", "api.example.com"));
        guard
            .revoke_always("fixture-mod", "network", "api.example.com")
            .expect("revoke");
        assert!(!guard.is_allowed("fixture-mod", "network", "api.example.com"));
    }

    {
        let guard = store.lock().expect("lock");
        assert!(!guard.is_allowed("fixture-mod", "network", "api.example.com"));
    }

    let _ = std::fs::remove_file(&path);
}

#[test]
#[ignore = "PERM-06: each gated fetch must append one JSONL log row"]
fn perm06_gated_fetch_appends_activity_log_row() {
    let _ = ();
}
