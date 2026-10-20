//! PERM-03 / PERM-10 audit: WASM host surface and mod network mediation.

#[test]
fn perm03_wasm_host_linker_has_no_network_import() {
    let src = include_str!("../src/wasm.rs");
    for needle in [
        "ub_host_fetch",
        "ub_host_http",
        "ub_host_network",
        "ub_host_websocket",
    ] {
        assert!(
            !src.contains(needle),
            "WASM linker must not expose {} until PERM-03 gates network",
            needle
        );
    }
    assert!(
        src.contains("ub_host_register_command"),
        "expected existing WASM host imports"
    );
}

#[test]
fn perm03_no_central_mod_network_shim_module() {
    let lib = include_str!("../src/lib.rs");
    assert!(
        !lib.contains("mod network") && !lib.contains("network_gate"),
        "PERM-03 expects a host-mediated mod fetch API module"
    );
}

#[test]
#[ignore = "PERM-03: undeclared origin must fail closed via host fetch shim"]
fn perm03_undeclared_host_blocked_at_fetch_gate() {
    let _ = ();
}

#[test]
#[ignore = "PERM-03: WASM guest cannot reach network without declaration and grant"]
fn perm03_wasm_cannot_fetch_without_declaration_and_grant() {
    let _ = ();
}
