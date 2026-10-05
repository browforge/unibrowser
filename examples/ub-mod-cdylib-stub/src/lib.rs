//! Sketch ABI for community `cdylib` mods ([ADR 0002](../docs/adr/0002-mod-system-and-sandboxing.md)).
//! The desktop host still static-links T0 `base-*` mods; this crate only documents exports.

use std::ffi::CStr;

/// Loader ABI version (independent of `MOD_API_VERSION` semver).
pub const UB_MOD_LOADER_ABI: u32 = 1;

#[no_mangle]
pub extern "C" fn ub_mod_loader_abi_version() -> u32 {
    UB_MOD_LOADER_ABI
}

/// NUL-terminated `MOD_API_VERSION` string from `unibrowser-mod-api`.
#[no_mangle]
pub extern "C" fn ub_mod_api_version() -> *const std::os::raw::c_char {
    static VER: &[u8] = b"0.1.0\0";
    VER.as_ptr() as *const std::os::raw::c_char
}

/// Minimal `info.json` payload for smoke tests (host would parse JSON properly).
#[no_mangle]
pub extern "C" fn ub_mod_info_json() -> *const std::os::raw::c_char {
    static INFO: &[u8] = b"{\"name\":\"ub-mod-cdylib-stub\",\"version\":\"0.1.0\",\"unibrowser_api\":\"0.1.0\"}\0";
    INFO.as_ptr() as *const std::os::raw::c_char
}

#[no_mangle]
pub extern "C" fn ub_mod_register_stub() -> i32 {
    // Future: receive host fn table, call `register_commands`.
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_readable() {
        let ver = unsafe { CStr::from_ptr(ub_mod_api_version()) }
            .to_str()
            .unwrap();
        assert_eq!(ver, unibrowser_mod_api::MOD_API_VERSION);
    }
}
