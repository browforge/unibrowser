//! Linux `cdylib` mod loader (feature `dynamic-mods`).

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::path::Path;

use libloading::{Library, Symbol};
use unibrowser_engine::{CommandRegistry, HandlerRegistration};
use unibrowser_mod_api::Enablement;

use crate::ModHostError;

pub const UB_MOD_LOADER_ABI: u32 = 1;

#[repr(C)]
pub struct UbModHostV1 {
    pub abi_version: u32,
    pub host_ctx: *mut c_void,
    pub register_command:
        Option<unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> i32>,
}

type RegisterFn = unsafe extern "C" fn(*const UbModHostV1) -> i32;
type AbiVersionFn = unsafe extern "C" fn() -> u32;

unsafe extern "C" fn host_register_command(
    ctx: *mut c_void,
    id: *const c_char,
    title: *const c_char,
) -> i32 {
    if ctx.is_null() || id.is_null() || title.is_null() {
        return -1;
    }
    let registry = &mut *(ctx as *mut CommandRegistry);
    let id = match CStr::from_ptr(id).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let title = match CStr::from_ptr(title).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    registry.register_command(unibrowser_mod_api::CommandMeta {
        id,
        title,
        icon: "extension",
        default_keybinding: None,
        aliases: &["hello mod"],
    });
    0
}

pub struct LoadedCdylibMod {
    pub library: Library,
    pub name: String,
}

/// Paths tried when `UNIBROWSER_CDYLIB_MOD_PATH` is unset.
pub fn default_cdylib_search_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Ok(p) = std::env::var("UNIBROWSER_CDYLIB_MOD_PATH") {
        paths.push(std::path::PathBuf::from(p));
    }
    paths.push(std::path::PathBuf::from("libub_mod_cdylib_stub.so"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("libub_mod_cdylib_stub.so"));
        }
    }
    paths.push(
        std::path::PathBuf::from("examples/ub-mod-cdylib-stub/target/debug/libub_mod_cdylib_stub.so"),
    );
    paths
}

/// Load enabled native mods from the user config `mods` directory.
pub fn load_user_native_mods(
    user_root: &Path,
    registry: &mut CommandRegistry,
) -> Result<Vec<LoadedCdylibMod>, ModHostError> {
    let mut loaded = Vec::new();
    for (mod_dir, lib) in crate::native::enabled_load_candidates(user_root)? {
        loaded.push(load_cdylib_mod(Some(&mod_dir), &lib, registry)?);
    }
    Ok(loaded)
}

pub fn load_cdylib_mod(
    mod_dir: Option<&Path>,
    path: &Path,
    registry: &mut CommandRegistry,
) -> Result<LoadedCdylibMod, ModHostError> {
    if let Some(dir) = mod_dir {
        crate::signing::verify_mod_dir(dir)?;
    }
    if !path.is_file() {
        return Err(ModHostError::Dynamic(format!(
            "cdylib not found: {}",
            path.display()
        )));
    }
    let library = unsafe { Library::new(path) }
        .map_err(|e| ModHostError::Dynamic(e.to_string()))?;
    let abi: Symbol<AbiVersionFn> = unsafe {
        library
            .get(b"ub_mod_loader_abi_version\0")
            .map_err(|e| ModHostError::Dynamic(e.to_string()))?
    };
    let abi_version = unsafe { abi() };
    if abi_version != UB_MOD_LOADER_ABI {
        return Err(ModHostError::Dynamic(format!(
            "cdylib ABI {} != host {}",
            abi_version,
            UB_MOD_LOADER_ABI
        )));
    }
    let register: Symbol<RegisterFn> = unsafe {
        library
            .get(b"ub_mod_register\0")
            .map_err(|e| ModHostError::Dynamic(e.to_string()))?
    };
    let host = UbModHostV1 {
        abi_version: UB_MOD_LOADER_ABI,
        host_ctx: (registry as *mut CommandRegistry).cast(),
        register_command: Some(host_register_command),
    };
    let code = unsafe { register(&host) };
    if code != 0 {
        return Err(ModHostError::Dynamic(format!(
            "ub_mod_register returned {}",
            code
        )));
    }
    if registry.command_meta("mod.hello").is_some() {
        registry.register_handler(HandlerRegistration {
            command_id: "mod.hello".into(),
            backend_id: "ub-mod-cdylib-stub".into(),
            enable: |_| Enablement::Enabled,
            execute: |_| Ok("Hello from dynamic cdylib mod".into()),
        });
    }
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "cdylib-mod".into());
    Ok(LoadedCdylibMod { library, name })
}

#[cfg(all(test, feature = "dynamic-mods"))]
mod tests {
    use super::*;
    use unibrowser_engine::CommandRegistry;

    #[test]
    fn load_cdylib_stub_when_built() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ub-mod-cdylib-stub/target/debug/libub_mod_cdylib_stub.so");
        if !path.is_file() {
            return;
        }
        let mut registry = CommandRegistry::with_builtins();
        let loaded = load_cdylib_mod(None, &path, &mut registry).expect("load stub");
        assert_eq!(loaded.name, "libub_mod_cdylib_stub");
        assert!(registry.command_meta("mod.hello").is_some());
    }
}
