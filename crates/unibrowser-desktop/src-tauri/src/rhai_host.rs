//! Rhai script evaluation with live `CommandRegistry` (ADR 0017).

use std::cell::RefCell;

use serde::Serialize;
use tauri::State;
use unibrowser_engine::keybinding::COMMAND_PALETTE_TOGGLE;
use unibrowser_script_host::allowlist::{filter_registry_ids, parse_allowlist_from_source};
use unibrowser_script_host::{eval_with_host, with_execute_hook, HostApi};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct RhaiScriptInfoDto {
    pub allowlist: Option<Vec<String>>,
    pub effective_commands: Vec<String>,
    pub document_commands_blocked: bool,
}

thread_local! {
    static RHAI_STATE: RefCell<Option<*const AppState>> = const { RefCell::new(None) };
}

fn browse_script_command(command_id: &str) -> Option<String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Some("palette".into());
    }
    None
}

pub fn execute_command_for_script(
    state: &AppState,
    command_id: &str,
    allow_document_commands: bool,
) -> Result<String, String> {
    if command_id == COMMAND_PALETTE_TOGGLE {
        return Err("command blocked in scripts when a document is open".into());
    }
    if command_id != COMMAND_PALETTE_TOGGLE
        && state
            .registry
            .lock()
            .map_err(|e| e.to_string())?
            .command_meta(command_id)
            .is_none()
    {
        return Err(format!("unknown command: {}", command_id));
    }
    let sess = state.session("main")?;
    let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
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
    if !allow_document_commands {
        return Err(
            "execute_command on open document blocked (enable in script panel)".into(),
        );
    }
    let doc = tab.document.as_mut().unwrap();
    state
        .registry
        .lock()
        .map_err(|e| e.to_string())?
        .execute(command_id, doc)
}

thread_local! {
    static RHAI_ALLOW_DOC: RefCell<bool> = const { RefCell::new(false) };
}

fn rhai_execute(command_id: String) -> Result<String, String> {
    let ptr = RHAI_STATE
        .with(|s| *s.borrow())
        .ok_or("script host not active")?;
    let allow_doc = RHAI_ALLOW_DOC.with(|c| *c.borrow());
    let state = unsafe { &*ptr };
    execute_command_for_script(state, &command_id, allow_doc)
}

fn build_host_api(state: &AppState, source: &str) -> Result<HostApi, String> {
    let registry_ids = state
        .registry
        .lock()
        .map_err(|e| e.to_string())?
        .sorted_command_ids();
    let allowlist = parse_allowlist_from_source(source);
    let mut command_ids = filter_registry_ids(&registry_ids, allowlist.as_deref());
    if !command_ids.iter().any(|id| id == COMMAND_PALETTE_TOGGLE) {
        command_ids.push(COMMAND_PALETTE_TOGGLE.to_string());
    }
    command_ids.sort();
    Ok(HostApi::from_registry_ids(command_ids, &[]))
}

#[tauri::command]
pub fn describe_rhai_script(source: String, state: State<AppState>) -> Result<RhaiScriptInfoDto, String> {
    let allowlist = parse_allowlist_from_source(&source);
    let api = build_host_api(state.inner(), &source)?;
    let document_commands_blocked = match state.session("main") {
        Ok(sess) => sess
            .tabs
            .lock()
            .map(|tabs| {
                tabs
                    .active_tab()
                    .map(|t| t.document.is_some())
                    .unwrap_or(false)
            })
            .unwrap_or(false),
        Err(_) => false,
    };
    Ok(RhaiScriptInfoDto {
        allowlist,
        effective_commands: api.command_ids,
        document_commands_blocked,
    })
}

pub const DEMO_HELLO_SCRIPT: &str = r#"// @allow unibrowser.font.size.increase
let hello = host_echo("Hello from Rhai");
let font = execute_command("unibrowser.font.size.increase");
hello + " · " + font"#;

pub const DEMO_HELLO_SCRIPT_BROWSE: &str =
    r#"host_echo("Hello from Rhai — open a text file and run again to bump font size")"#;

pub fn eval_rhai_script(
    state: &AppState,
    source: &str,
    allow_document_commands: bool,
) -> Result<String, String> {
    let api = build_host_api(state, source)?;
    RHAI_STATE.with(|slot| {
        *slot.borrow_mut() = Some(state as *const AppState);
        RHAI_ALLOW_DOC.with(|c| *c.borrow_mut() = allow_document_commands);
        let result = with_execute_hook(Box::new(rhai_execute), || eval_with_host(source, &api));
        RHAI_ALLOW_DOC.with(|c| *c.borrow_mut() = false);
        *slot.borrow_mut() = None;
        result
    })
}

#[tauri::command]
pub fn run_rhai_script(
    source: String,
    allow_document_commands: Option<bool>,
    state: State<AppState>,
) -> Result<String, String> {
    eval_rhai_script(state.inner(), &source, allow_document_commands.unwrap_or(false))
}
