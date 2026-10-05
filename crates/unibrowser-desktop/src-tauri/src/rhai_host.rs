//! Rhai script evaluation with live `CommandRegistry` (ADR 0017).

use std::cell::RefCell;

use tauri::State;
use unibrowser_engine::keybinding::COMMAND_PALETTE_TOGGLE;
use unibrowser_script_host::{eval_with_host, with_execute_hook, HostApi};

use crate::state::AppState;

thread_local! {
    static RHAI_STATE: RefCell<Option<*const AppState>> = const { RefCell::new(None) };
}

pub fn execute_command_for_script(state: &AppState, command_id: &str) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Err("command blocked in scripts".into());
    }
    if state.registry.command_meta(command_id).is_none() {
        return Err(format!("unknown command: {}", command_id));
    }
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    let doc = tab
        .document
        .as_mut()
        .ok_or("execute_command needs an open document tab")?;
    state.registry.execute(command_id, doc)
}

fn rhai_execute(command_id: String) -> Result<String, String> {
    let ptr = RHAI_STATE
        .with(|s| *s.borrow())
        .ok_or("script host not active")?;
    let state = unsafe { &*ptr };
    execute_command_for_script(state, &command_id)
}

#[tauri::command]
pub fn run_rhai_script(source: String, state: State<AppState>) -> Result<String, String> {
    let ids = state.registry.sorted_command_ids();
    let api = HostApi::from_registry_ids(ids, &[COMMAND_PALETTE_TOGGLE]);
    let inner: &AppState = state.inner();
    RHAI_STATE.with(|slot| {
        *slot.borrow_mut() = Some(inner);
        let result = with_execute_hook(Box::new(rhai_execute), || eval_with_host(&source, &api));
        *slot.borrow_mut() = None;
        result
    })
}
