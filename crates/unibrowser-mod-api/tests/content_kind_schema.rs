use unibrowser_mod_api::{
    ContentKindRegistry, CONTENT_KIND_SCHEMA_VERSION, validate_content_kind_id,
    capability_surface::{validate_capability_id, CAPABILITY_SCHEMA_VERSION},
};

#[test]
fn canonical_core_kind_ids_are_registered() {
    let reg = ContentKindRegistry::with_core_kinds();
    assert!(reg.get("unibrowser.content.text_plain").is_some());
    assert!(reg.get("unibrowser.content.image").is_some());
}

#[test]
fn rejects_plugin_namespace_on_core_capability_ids() {
    assert!(validate_capability_id("acme.capability.open_text").is_err());
}

#[test]
fn schema_versions_are_non_empty() {
    assert!(!CONTENT_KIND_SCHEMA_VERSION.is_empty());
    assert!(!CAPABILITY_SCHEMA_VERSION.is_empty());
}

#[test]
fn unknown_kind_on_extension_fails_validation() {
    let err = validate_content_kind_id("unknown.kind", Some("acme")).unwrap_err();
    assert!(err.contains("namespace"));
}
