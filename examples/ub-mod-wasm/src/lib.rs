//! Out-of-tree WASM mod example: one command + `.ubwasm` file handler.

use unibrowser_mod_sdk::{
    register_capability_snapshot, register_command, register_family_snapshot,
    register_file_extension, UB_WASM_ABI_VERSION,
};

pub const DEMO_COMMAND: &str = "mod.wasm.demo";
pub const DEMO_EXTENSION: &str = "ubwasm";
pub const SNAPSHOT: &str = r#"{"api_version":1,"control_id":"unibrowser.control.text.subscript","capability":"emulated","reason":null,"emulation_note":"Wraps the selection in Markdown <sub> tags (proving mod).","selection":"off","value":null,"mixed":false}"#;
pub const FAMILY_SNAPSHOT: &str = r#"{"api_version":1,"family_id":"unibrowser.family.text-formatting","capability":"supported","reason":null}"#;

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
    if register_capability_snapshot(SNAPSHOT) != 0 {
        return -3;
    }
    if register_family_snapshot(FAMILY_SNAPSHOT) != 0 {
        return -4;
    }
    0
}
