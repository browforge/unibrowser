//! PERM-01 audit: ADR 0029 `permissions` manifest block (see docs/status/perm-audit.md).

use unibrowser_mod_api::ModManifest;

#[test]
fn perm01_mod_manifest_type_has_no_permissions_field() {
    let src = include_str!("../src/manifest.rs");
    assert!(
        !src.contains("permission_schema") && !src.contains("network.domains"),
        "ModManifest should gain ADR 0029 permissions fields under PERM-01"
    );
}

#[test]
fn perm01_loader_ignores_permissions_json_without_validation() {
    let json = r#"{
        "name": "perm-fixture",
        "version": "1.0.0",
        "title": "Fixture",
        "unibrowser_api": "0.1.0",
        "permissions": {
            "permission_schema": "1",
            "network": {
                "domains": ["https://evil.example"],
                "reason": "test",
                "disclosure": ["payload"]
            }
        }
    }"#;
    let manifest: ModManifest = serde_json::from_str(json).expect("parse");
    assert_eq!(manifest.name, "perm-fixture");
}

#[test]
#[ignore = "PERM-01: host must reject manifests with undeclared capability use at load"]
fn perm01_invalid_wildcard_domain_fails_load() {
    let _ = ();
}
