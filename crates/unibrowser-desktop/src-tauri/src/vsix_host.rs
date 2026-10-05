//! VSIX command enablement and Node host execution glue.

use std::path::Path;

use crate::preferences::load_preferences;

pub fn node_extension_host_enabled() -> bool {
    load_preferences().advanced_node_extension_host
}

pub fn vsix_command_palette_state(root: &Path, command_id: &str) -> (bool, Option<String>) {
    let pref = node_extension_host_enabled();
    let node = unibrowser_extension_host::node_on_path();
    let (ok, reason) = unibrowser_extension_host::vsix_palette_enablement(pref, node);
    if !ok {
        return (ok, reason);
    }
    let ctx = unibrowser_vsix::lookup_vsix_command_context(root, command_id);
    if ctx.is_none() {
        return (false, Some("VSIX command not installed".into()));
    }
    let ctx = ctx.unwrap();
    if unibrowser_vsix::read_extension_main(&ctx.install_dir).is_none() {
        return (
            false,
            Some(unibrowser_extension_host::VSIX_NODE_HOST_NO_MAIN.into()),
        );
    }
    (true, None)
}
