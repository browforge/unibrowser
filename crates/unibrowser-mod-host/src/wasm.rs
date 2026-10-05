//! T2 WASM mod host (stub). See [ADR 0019](../../docs/adr/0019-wasm-t2-sandbox.md).

use std::path::Path;

use unibrowser_engine::CommandRegistry;

use crate::ModHostError;

/// Whether a WASM runtime is linked into this build (`wasm-mods` feature stub today).
pub fn runtime_linked() -> bool {
    cfg!(feature = "wasm-mods")
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
