//! VSIX command enablement and Node host execution glue.

use std::path::Path;

use base_text::{try_text_backend, TextBackend};
use tauri::{AppHandle, State};
use unibrowser_extension_host::{EditorSnapshot, NodeExecuteResult, VsixInvokeContext};

use crate::commands::execute_builtin_from_extension_host;
use crate::state::AppState;
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

pub fn active_editor_snapshot(
    state: &AppState,
    window: &tauri::Window,
) -> Option<EditorSnapshot> {
    let sess = state.session(window.label()).ok()?;
    let mut tabs = sess.tabs.lock().ok()?;
    let tab = tabs.active_tab_mut()?;
    let doc = tab.document.as_mut()?;
    let backend = try_text_backend(&mut *doc.backend)?;
    Some(EditorSnapshot {
        language_id: TextBackend::language_id(&backend.path).to_string(),
        text: backend.content.clone(),
    })
}

pub fn run_vsix_node_command(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
    ctx: unibrowser_vsix::VsixCommandContext,
    main_rel: String,
    command_id: String,
) -> Result<NodeExecuteResult, String> {
    let configuration =
        crate::vsix_extension_config::load_extension_configuration(&ctx.extension_id);
    let editor = active_editor_snapshot(state.inner(), &window);
    let invoke = VsixInvokeContext {
        install_dir: ctx.install_dir.clone(),
        extension_root: ctx.install_dir.join("extension"),
        main_rel,
        extension_id: ctx.extension_id.clone(),
        command_id,
        editor,
        configuration,
    };
    unibrowser_extension_host::execute_extension_command(invoke, |builtin_id| {
        execute_builtin_from_extension_host(
            window.clone(),
            builtin_id.to_string(),
            app.clone(),
            state.clone(),
        )
    })
}
