//! Rhai script evaluation with live `CommandRegistry` (ADR 0017).

use std::cell::RefCell;

use tauri::State;
use unibrowser_engine::keybinding::COMMAND_PALETTE_TOGGLE;
use unibrowser_script_host::allowlist::{filter_registry_ids, parse_allowlist_from_source};
use unibrowser_script_host::{eval_with_host, with_execute_hook, HostApi};

use crate::state::AppState;

thread_local! {
    static RHAI_STATE: RefCell<Option<*const AppState>> = const { RefCell::new(None) };
}

fn browse_script_command(command_id: &str) -> Option<String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Some("palette".into());
    }
    None
}

pub fn execute_command_for_script(state: &AppState, command_id: &str) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Err("command blocked in scripts when a document is open".into());
    }
    if command_id != COMMAND_PALETTE_TOGGLE
        && state.registry.command_meta(command_id).is_none()
    {
        return Err(format!("unknown command: {}", command_id));
    }
    let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no tab")?;
    if tab.document.is_none() {
        if let Some(msg) = browse_script_command(command_id) {
            return Ok(msg);
        }
        return Err(
            "execute_command needs an open document (browse-only: unibrowser.palette.toggle)"
                .into(),
        );
    }
    let doc = tab.document.as_mut().unwrap();
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
    let registry_ids = state.registry.sorted_command_ids();
    let allowlist = parse_allowlist_from_source(&source);
    let mut command_ids = filter_registry_ids(&registry_ids, allowlist.as_deref());
    if !command_ids.iter().any(|id| id == COMMAND_PALETTE_TOGGLE) {
        command_ids.push(COMMAND_PALETTE_TOGGLE.to_string());
    }
    command_ids.sort();
    let api = HostApi::from_registry_ids(command_ids, &[]);
    let inner: &AppState = state.inner();
    RHAI_STATE.with(|slot| {
        *slot.borrow_mut() = Some(inner);
        let result = with_execute_hook(Box::new(rhai_execute), || eval_with_host(&source, &api));
        *slot.borrow_mut() = None;
        result
    })
}
