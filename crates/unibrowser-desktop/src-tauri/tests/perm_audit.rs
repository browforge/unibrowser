//! PERM-02..07 audit probes for desktop host (ADR 0029).

#[test]
fn perm02_no_permission_grants_persistence_path() {
    let config = include_str!("../src/config.rs");
    assert!(
        !config.contains("permission-grants"),
        "PERM-02 grant store file not wired in config"
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
    assert!(!config.contains("permission-grants"));
}

#[test]
#[ignore = "PERM-02: revoke Always grant must take effect without restart"]
fn perm02_revoke_always_grant_without_restart() {
    let _ = ();
}

#[test]
#[ignore = "PERM-06: each gated fetch must append one JSONL log row"]
fn perm06_gated_fetch_appends_activity_log_row() {
    let _ = ();
}
