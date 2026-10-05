//! VSIX command enablement and Node host execution glue.

use std::path::Path;

use base_text::{try_text_backend, TextBackend};
use tauri::{AppHandle, State};
use unibrowser_extension_host::{
    apply_text_edits, format_document_via_node, EditorSnapshot, NodeExecuteResult, TextEditDto,
    VsixInvokeContext,
};

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
    let sess = match state.session(window.label()) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let mut tabs = sess.tabs.lock().ok()?;
    let tab = tabs.active_tab_mut()?;
    let doc = tab.document.as_mut()?;
    let backend = try_text_backend(&mut *doc.backend)?;
    Some(EditorSnapshot {
        language_id: TextBackend::language_id(&backend.path).to_string(),
        text: backend.content.clone(),
    })
}

pub fn apply_editor_text_edits(
    window: &tauri::Window,
    state: &AppState,
    edits: &[TextEditDto],
) -> Result<String, String> {
    if edits.is_empty() {
        return Ok("no edits".into());
    }
    let sess = state.session(window.label())?;
    let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let backend = try_text_backend(&mut *doc.backend).ok_or("not a text document")?;
    let path = backend.path.clone();
    let before = backend.content.clone();
    let after = apply_text_edits(backend.content.clone(), edits)?;
    backend.set_content(after);
    let mut stacks = sess.undo_stacks.lock().map_err(|e| e.to_string())?;
    stacks.record_change(&path, before, &backend.content);
    if backend.dirty {
        crate::crash_recovery::maybe_backup_dirty(&path, base_text::BACKEND_ID, &backend.content);
    }
    Ok("applied".into())
}

pub fn format_active_document(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<String, String> {
    if !node_extension_host_enabled() {
        return Err(unibrowser_extension_host::VSIX_NODE_HOST_PREF_OFF.into());
    }
    if !unibrowser_extension_host::node_on_path() {
        return Err(unibrowser_extension_host::VSIX_NODE_HOST_NO_NODE.into());
    }
    let editor = active_editor_snapshot(state.inner(), &window).ok_or("no text editor")?;
    let edits = format_document_via_node(editor)?;
    if edits.is_empty() {
        return Ok("Document already formatted".into());
    }
    apply_editor_text_edits(&window, state.inner(), &edits)?;
    Ok("Document formatted".into())
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
    let window_for_edits = window.clone();
    let state_for_edits = state.clone();
    unibrowser_extension_host::execute_extension_command(invoke, move |call| {
        if call.kind == "workspace_edit" {
            return apply_editor_text_edits(&window_for_edits, state_for_edits.inner(), &call.edits);
        }
        let builtin_id = call
            .command_id
            .as_deref()
            .ok_or("missing builtin command id")?;
        execute_builtin_from_extension_host(
            window.clone(),
            builtin_id.to_string(),
            app.clone(),
            state.clone(),
        )
    })
}
