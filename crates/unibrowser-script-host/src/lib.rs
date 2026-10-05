//! Thin Rhai sandbox with a curated host API ([ADR 0017](../../docs/adr/0017-scripting-engine-rhai.md)).

use rhai::{Engine, Scope};

#[derive(Debug, Clone, Default)]
pub struct HostApiStub {
    /// Command ids exposed to scripts (read-only stub until full registry bridge).
    pub command_ids: Vec<String>,
}

impl HostApiStub {
    pub fn with_sample_commands() -> Self {
        Self {
            command_ids: vec![
                "unibrowser.font.size.increase".into(),
                "unibrowser.font.size.decrease".into(),
                "unibrowser.palette.toggle".into(),
            ],
        }
    }
}

/// Evaluate Rhai with registered host stubs (`list_commands`, `host_echo`).
pub fn eval_with_host(source: &str, api: &HostApiStub) -> Result<String, String> {
    let mut engine = Engine::new();
    let ids = api.command_ids.clone();
    engine.register_fn("list_commands", move || -> rhai::Dynamic {
        rhai::Dynamic::from(ids.clone())
    });
    engine.register_fn("host_echo", |msg: &str| -> String {
        format!("host:{}", msg)
    });
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
    fn list_commands_stub() {
        let out = eval_with_host("host_echo(\"ok\")", &HostApiStub::with_sample_commands())
            .unwrap();
        assert_eq!(out, "host:ok");
    }
}
