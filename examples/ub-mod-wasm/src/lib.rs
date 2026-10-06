//! Out-of-tree WASM mod example: one command + `.ubwasm` file handler.

use unibrowser_mod_sdk::{register_command, register_file_extension, UB_WASM_ABI_VERSION};

pub const DEMO_COMMAND: &str = "mod.wasm.demo";
pub const DEMO_EXTENSION: &str = "ubwasm";

#[no_mangle]
pub extern "C" fn ub_wasm_abi_version() -> i32 {
    UB_WASM_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn ub_wasm_register() -> i32 {
    if register_command(DEMO_COMMAND, "WASM demo command") != 0 {
        return -1;
    }
    if register_file_extension(DEMO_EXTENSION) != 0 {
        return -2;
    }
    0
}
