//! PERM-01: invalid permissions skip mod at discover time.

use std::collections::HashMap;
use std::fs;
use unibrowser_engine::CommandRegistry;
use unibrowser_mod_host::{ModHost, ModRegisterFn};

#[test]
fn perm01_invalid_permissions_skips_mod_folder() {
    let root = std::env::temp_dir().join(format!(
        "ub-perm01-loader-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let bad = root.join("evil-net-mod");
    fs::create_dir_all(&bad).expect("dir");
    fs::write(
        bad.join("info.json"),
        r#"{
            "name": "evil-net-mod",
            "version": "1.0.0",
            "title": "Evil",
            "unibrowser_api": "0.1.0",
            "permissions": {
                "permission_schema": "1",
                "network": {
                    "domains": ["*"],
                    "reason": "exfil",
                    "disclosure": ["everything"]
                }
            }
        }"#,
    )
    .expect("write");

    let mut registry = CommandRegistry::with_builtins();
    let bundled: HashMap<&str, ModRegisterFn> = HashMap::new();
    let host = ModHost::discover_and_load(&[root.clone()], &mut registry, &bundled)
    .expect("discover");

    assert!(
        !host.mods.iter().any(|m| m.manifest.name == "evil-net-mod"),
        "invalid permissions mod must not load"
    );
    assert!(
        host.resolve_notes
            .iter()
            .any(|n| n.contains("evil-net-mod") && n.contains("permissions")),
        "notes: {:?}",
        host.resolve_notes
    );

    let _ = fs::remove_dir_all(&root);
}
