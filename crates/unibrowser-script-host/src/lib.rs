//! Thin Rhai sandbox with a curated host API ([ADR 0017](../../docs/adr/0017-scripting-engine-rhai.md)).

use std::cell::RefCell;
use std::collections::HashSet;

use rhai::{Engine, EvalAltResult, Scope};

pub mod allowlist;

#[derive(Debug, Clone)]
pub struct SandboxLimits {
    pub max_execute_calls: usize,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_execute_calls: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostApi {
    pub command_ids: Vec<String>,
    pub blocked_command_ids: HashSet<String>,
    pub limits: SandboxLimits,
}

impl HostApi {
    pub fn from_registry_ids(ids: Vec<String>, blocked: &[&str]) -> Self {
        Self {
            command_ids: ids,
            blocked_command_ids: blocked.iter().map(|s| (*s).to_string()).collect(),
            limits: SandboxLimits::default(),
        }
    }
}

thread_local! {
    static EXEC_HOOK: RefCell<Option<Box<dyn Fn(String) -> Result<String, String>>>> =
        RefCell::new(None);
    static EXEC_COUNT: RefCell<usize> = RefCell::new(0);
}

/// Run Rhai evaluation with a per-evaluation `execute_command` hook (same thread).
pub fn with_execute_hook<R>(
    hook: Box<dyn Fn(String) -> Result<String, String>>,
    f: impl FnOnce() -> R,
) -> R {
    EXEC_HOOK.with(|cell| {
        *cell.borrow_mut() = Some(hook);
        EXEC_COUNT.with(|c| *c.borrow_mut() = 0);
        let out = f();
        *cell.borrow_mut() = None;
        out
    })
}

/// Evaluate Rhai with `list_commands` and sandboxed `execute_command`.
pub fn eval_with_host(source: &str, api: &HostApi) -> Result<String, String> {
    let mut engine = Engine::new();
    let list_ids = api.command_ids.clone();
    let exec_ids = api.command_ids.clone();
    let blocked = api.blocked_command_ids.clone();
    let max_calls = api.limits.max_execute_calls;

    engine.register_fn("list_commands", move || -> rhai::Dynamic {
        rhai::Dynamic::from(list_ids.clone())
    });
    engine.register_fn("host_echo", |msg: &str| -> String {
        format!("host:{}", msg)
    });
    engine.register_fn(
        "execute_command",
        move |command_id: String| -> Result<String, Box<EvalAltResult>> {
            EXEC_COUNT.with(|c| {
                let mut count = c.borrow_mut();
                *count += 1;
                if *count > max_calls {
                    return Err::<(), Box<EvalAltResult>>(format!(
                        "execute_command limit exceeded (max {})",
                        max_calls
                    )
                    .into());
                }
                Ok(())
            })?;
            if blocked.contains(&command_id) {
                return Err(format!("command blocked in scripts: {}", command_id).into());
            }
            if !exec_ids.iter().any(|id| id == &command_id) {
                return Err(format!("unknown command: {}", command_id).into());
            }
            EXEC_HOOK.with(|cell| {
                let hook = cell.borrow();
                let hook = hook
                    .as_ref()
                    .ok_or_else(|| {
                        Box::<EvalAltResult>::from("execute_command unavailable (no host hook)")
                    })?;
                hook(command_id).map_err(|e| e.into())
            })
        },
    );

    let mut scope = Scope::new();
    let value = engine
        .eval_with_scope::<rhai::Dynamic>(&mut scope, source)
        .map_err(|e| e.to_string())?;
    Ok(format!("{}", value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_echo_works() {
        let api = HostApi::from_registry_ids(vec!["a".into()], &[]);
        let out = eval_with_host("host_echo(\"ok\")", &api).unwrap();
        assert_eq!(out, "host:ok");
    }

    #[test]
    fn execute_respects_limit() {
        let api = HostApi {
            command_ids: vec!["x".into()],
            blocked_command_ids: HashSet::new(),
            limits: SandboxLimits {
                max_execute_calls: 0,
            },
        };
        let hook = Box::new(|_: String| Ok("ok".into()));
        let err = with_execute_hook(hook, || eval_with_host("execute_command(\"x\")", &api))
            .unwrap_err();
        assert!(err.contains("limit"));
    }

    #[test]
    fn allowlist_parser() {
        let ids = allowlist::parse_allowlist_from_source(
            "// allow-commands: a, b\nlet x = 1;",
        )
        .unwrap();
        assert_eq!(ids, vec!["a", "b"]);
    }
}
