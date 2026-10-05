//! Example `cdylib` mod for the Plugin SDK draft ([`docs/plugin-sdk/README.md`](../../docs/plugin-sdk/README.md)).

pub const UB_MOD_LOADER_ABI: u32 = 1;

#[repr(C)]
pub struct UbModHostV1 {
    pub abi_version: u32,
    pub host_ctx: *mut std::os::raw::c_void,
    pub register_command: Option<
        unsafe extern "C" fn(
            *mut std::os::raw::c_void,
            *const std::os::raw::c_char,
            *const std::os::raw::c_char,
        ) -> i32,
    >,
}

#[no_mangle]
pub extern "C" fn ub_mod_loader_abi_version() -> u32 {
    UB_MOD_LOADER_ABI
}

#[no_mangle]
pub extern "C" fn ub_mod_api_version() -> *const std::os::raw::c_char {
    static VER: &[u8] = b"0.1.0\0";
    VER.as_ptr() as *const std::os::raw::c_char
}

#[no_mangle]
pub extern "C" fn ub_mod_info_json() -> *const std::os::raw::c_char {
    static INFO: &[u8] =
        b"{\"name\":\"ub-mod-cdylib-stub\",\"version\":\"0.1.0\",\"unibrowser_api\":\"0.1.0\"}\0";
    INFO.as_ptr() as *const std::os::raw::c_char
}

#[no_mangle]
pub unsafe extern "C" fn ub_mod_register(host: *const UbModHostV1) -> i32 {
    if host.is_null() {
        return -1;
    }
    let host = &*host;
    if host.abi_version != UB_MOD_LOADER_ABI {
        return -2;
    }
    let register = match host.register_command {
        Some(f) => f,
        None => return -3,
    };
    static ID: &[u8] = b"mod.hello\0";
    static TITLE: &[u8] = b"Hello from mod\0";
    register(
        host.host_ctx,
        ID.as_ptr() as *const std::os::raw::c_char,
        TITLE.as_ptr() as *const std::os::raw::c_char,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_readable() {
        assert_eq!(ub_mod_loader_abi_version(), UB_MOD_LOADER_ABI);
    }
}
