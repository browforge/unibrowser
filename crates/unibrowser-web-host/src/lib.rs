#![recursion_limit = "512"]

mod ai_prefs;
mod document_model;
mod host;
mod prefs;
mod snapshot;

use std::cell::RefCell;

use host::WebHost;
use serde::Serialize;
use wasm_bindgen::prelude::*;

thread_local! {
    static HOST: RefCell<Option<WebHost>> = RefCell::new(None);
}

fn with_host_mut<R>(f: impl FnOnce(&mut WebHost) -> R) -> R {
    HOST.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            *slot = Some(WebHost::new());
        }
        f(slot.as_mut().expect("host"))
    })
}

#[derive(Serialize)]
struct InvokeEnvelope {
    ok: bool,
    value: serde_json::Value,
    error: Option<String>,
}

/// Initialise the WASM host (idempotent).
#[wasm_bindgen]
pub fn host_init() {
    with_host_mut(|_| ());
}

/// Dispatch a Tauri-shaped command. `args_json` is a JSON object or `null`.
#[wasm_bindgen]
pub fn host_invoke(cmd: &str, args_json: &str) -> String {
    let args: serde_json::Value = if args_json.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(args_json).unwrap_or(serde_json::Value::Null)
    };
    let result = with_host_mut(|host| host.dispatch(cmd, &args));
    let envelope = match result {
        Ok(value) => InvokeEnvelope {
            ok: true,
            value,
            error: None,
        },
        Err(err) => InvokeEnvelope {
            ok: false,
            value: serde_json::Value::Null,
            error: Some(err),
        },
    };
    serde_json::to_string(&envelope).unwrap_or_else(|e| {
        format!(
            r#"{{"ok":false,"value":null,"error":"{}"}}"#,
            e.to_string().replace('"', "'")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_has_home_tab() {
        host_init();
        let raw = host_invoke("get_browser_state", "null");
        assert!(raw.contains("\"show_new_tab_page\":true"));
    }

    #[test]
    fn command_manager_round_trip() {
        host_init();
        let raw = host_invoke("get_command_manager", "null");
        assert!(raw.contains("\"tabs\""));
        assert!(raw.contains("\"ok\":true"));
    }

    #[test]
    fn command_manager_many_invokes() {
        host_init();
        for _ in 0..200 {
            let raw = host_invoke("get_command_manager", "null");
            assert!(raw.contains("\"ok\":true"));
        }
    }

    #[test]
    fn markdown_tab_exposes_compact_toolbar_families() {
        host_init();
        let open_args = serde_json::json!({
            "path": "demo-pack/sample.md",
            "content": "## Demo\n"
        });
        let open_raw = host_invoke(
            "open_text_with_content",
            &serde_json::to_string(&open_args).unwrap(),
        );
        assert!(open_raw.contains("\"ok\":true"), "{open_raw}");
        let cm_raw = host_invoke("get_command_manager", "null");
        assert!(
            cm_raw.contains("\"compact_toolbar_families\""),
            "{cm_raw}"
        );
        assert!(
            cm_raw.contains("unibrowser.family.text_compact_toolbar")
                || cm_raw.contains("unibrowser.control.semantic.block_style"),
            "{cm_raw}"
        );
    }
}
