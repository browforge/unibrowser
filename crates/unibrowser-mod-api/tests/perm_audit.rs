//! PERM-01 audit: ADR 0029 `permissions` manifest block (see docs/status/perm-audit.md).

use unibrowser_mod_api::{
    validate_permissions, ModManifest, ModPermissionsBlock, NetworkPermissions,
    PERMISSION_SCHEMA_VERSION,
};

#[test]
fn perm01_mod_manifest_has_permissions_field_and_validation() {
    let src = include_str!("../src/manifest.rs");
    assert!(
        src.contains("permissions") && src.contains("ModPermissionsBlock"),
        "ModManifest should declare ADR 0029 permissions"
    );
    let perm_src = include_str!("../src/permissions.rs");
    assert!(perm_src.contains("permission_schema"));
    assert!(perm_src.contains("validate_network_host_entry"));
}

#[test]
fn perm01_loader_validates_permissions_json() {
    let json = r#"{
        "name": "perm-fixture",
        "version": "1.0.0",
        "title": "Fixture",
        "unibrowser_api": "0.1.0",
        "permissions": {
            "permission_schema": "1",
            "network": {
                "domains": ["api.example.com"],
                "reason": "test",
                "disclosure": ["payload"]
            }
        }
    }"#;
    let manifest: ModManifest = serde_json::from_str(json).expect("parse");
    validate_permissions(&manifest.name, manifest.permissions.as_ref().unwrap())
        .expect("valid fixture");
}

#[test]
fn perm01_invalid_wildcard_domain_fails_load() {
    let block = ModPermissionsBlock {
        permission_schema: PERMISSION_SCHEMA_VERSION.into(),
        network: Some(NetworkPermissions {
            domains: vec!["*".into()],
            reason: "test".into(),
            disclosure: vec!["payload".into()],
        }),
        ..Default::default()
    };
    let err = validate_permissions("perm-fixture", &block).unwrap_err();
    assert!(
        err.contains("broad") || err.contains("wildcard") || err.contains("invalid"),
        "unexpected error: {}",
        err
    );
}
