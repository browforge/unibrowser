//! Guest helpers for Unibrowser T2 WASM mods (`wasm32-unknown-unknown`).
//!
//! Host imports live in module `ub_host`; export `ub_wasm_abi_version` and `ub_wasm_register`.

pub const UB_WASM_ABI_VERSION: i32 = 1;

#[link(wasm_import_module = "ub_host")]
extern "C" {
    fn ub_host_register_command(
        id_ptr: i32,
        id_len: i32,
        title_ptr: i32,
        title_len: i32,
    ) -> i32;
    fn ub_host_register_extension(ext_ptr: i32, ext_len: i32) -> i32;
    fn ub_host_register_capability_snapshot(json_ptr: i32, json_len: i32) -> i32;
}

/// Register one immutable capability snapshot as JSON during guest initialisation.
pub fn register_capability_snapshot(json: &str) -> i32 {
    unsafe {
        ub_host_register_capability_snapshot(json.as_ptr() as i32, json.len() as i32)
    }
}

/// Register a palette / Command Manager command (id + title UTF-8).
pub fn register_command(id: &str, title: &str) -> i32 {
    unsafe {
        ub_host_register_command(
            id.as_ptr() as i32,
            id.len() as i32,
            title.as_ptr() as i32,
            title.len() as i32,
        )
    }
}

/// Register a file extension (no dot) handled by this mod's stub viewer.
pub fn register_file_extension(ext: &str) -> i32 {
    unsafe { ub_host_register_extension(ext.as_ptr() as i32, ext.len() as i32) }
}
