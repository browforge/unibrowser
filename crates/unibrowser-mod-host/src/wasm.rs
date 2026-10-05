//! T2 WASM mod host (stub). See [ADR 0019](../../docs/adr/0019-wasm-t2-sandbox.md).

use std::path::{Path, PathBuf};

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{CommandMeta, Enablement};

use crate::ModHostError;

pub const HELLO_WASM_COMMAND: &str = "mod.wasm.hello";

/// Whether a WASM runtime is linked into this build (`wasm-mods` feature stub today).
pub fn runtime_linked() -> bool {
    cfg!(feature = "wasm-mods")
}

/// Path to the repo proving module when developing from the workspace.
pub fn proving_hello_wasm_path() -> Option<PathBuf> {
    let cwd = Path::new("examples/hello-wasm/hello.wasm");
    if cwd.is_file() {
        return cwd.canonicalize().ok();
    }
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/hello-wasm/hello.wasm");
    if from_manifest.is_file() {
        return from_manifest.canonicalize().ok();
    }
    None
}

pub fn proving_hello_wasm_available() -> bool {
    proving_hello_wasm_path().is_some()
}

fn hello_enable(_doc: &DocumentSession) -> Enablement {
    Enablement::Enabled
}

fn hello_execute(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("WASM hello.wasm proving stub (guest runtime not wired; see docs/wasm-t2-status.md)".into())
}

/// When the proving `hello.wasm` file exists, register a no-op command (no wasmtime).
pub fn register_proving_hello_stub(registry: &mut CommandRegistry) -> Option<String> {
    if !proving_hello_wasm_available() {
        return None;
    }
    registry.register_command(CommandMeta {
        id: HELLO_WASM_COMMAND,
        title: "Hello from WASM proving stub",
        icon: "extension",
        default_keybinding: None,
        aliases: &["wasm proving", "hello wasm"],
    });
    registry.register_handler(HandlerRegistration {
        command_id: HELLO_WASM_COMMAND.into(),
        backend_id: "wasm-proving-stub".into(),
        enable: hello_enable,
        execute: hello_execute,
    });
    Some("wasm-proving-hello".into())
}

/// Load a `.wasm` mod from `mod_dir` (not implemented).
pub fn load_wasm_mod(
    _mod_dir: &Path,
    _registry: &mut CommandRegistry,
) -> Result<(), ModHostError> {
    Err(ModHostError::Dynamic(
        "WASM mod load not implemented (enable wasm-mods when wasmtime is wired)".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proving_path_when_examples_present() {
        if proving_hello_wasm_path().is_some() {
            assert!(proving_hello_wasm_available());
        }
    }
}
