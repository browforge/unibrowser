//! T2 WASM mod host (wasmi). See [ADR 0019](../../docs/adr/0019-wasm-t2-sandbox.md).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use unibrowser_engine::{CommandRegistry, HandlerRegistration};
use unibrowser_mod_api::{CommandMeta, Enablement, ModManifest};

use crate::ModHostError;
use crate::wasm_backend::WasmStubBackend;

pub const HELLO_WASM_COMMAND: &str = "mod.wasm.hello";
pub const UB_WASM_ABI_VERSION: i32 = 1;

static WASM_FILE_EXTENSIONS: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

fn extension_map() -> &'static RwLock<HashMap<String, String>> {
    WASM_FILE_EXTENSIONS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Whether a WASM runtime is linked into this build.
pub fn runtime_linked() -> bool {
    cfg!(feature = "wasm-mods")
}

pub fn wasm_file_extensions() -> HashMap<String, String> {
    extension_map()
        .read()
        .map(|m| m.clone())
        .unwrap_or_default()
}

pub fn open_wasm_extension_backend(
    path: &Path,
) -> Option<Box<dyn unibrowser_engine::document::DocumentBackend>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mod_name = extension_map()
        .read()
        .ok()
        .and_then(|m| m.get(&ext).cloned())?;
    WasmStubBackend::open_path(path, &mod_name)
        .ok()
        .map(|b| Box::new(b) as Box<dyn unibrowser_engine::document::DocumentBackend>)
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

fn hello_enable(_doc: &unibrowser_engine::document::DocumentSession) -> Enablement {
    Enablement::Enabled
}

fn hello_execute(_doc: &mut unibrowser_engine::document::DocumentSession) -> Result<String, String> {
    Ok("WASM hello.wasm proving artefact (empty module; use ub-mod-wasm for a live guest)".into())
}

/// When the proving `hello.wasm` file exists, register a no-op command (legacy empty module).
pub fn register_proving_hello_stub(registry: &mut CommandRegistry) -> Option<String> {
    if !proving_hello_wasm_available() {
        return None;
    }
    if registry.command_meta(HELLO_WASM_COMMAND).is_some() {
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

#[cfg(feature = "wasm-mods")]
mod runtime {
    use super::*;
    use wasmi::{Caller, Engine, Linker, Module, Store};

    pub struct PendingWasmRegistrations {
        pub commands: Vec<(String, String)>,
        pub extensions: Vec<String>,
        pub mod_name: String,
    }

    struct HostState {
        pending: PendingWasmRegistrations,
    }

    fn read_guest_utf8(
        caller: &mut Caller<'_, HostState>,
        ptr: i32,
        len: i32,
    ) -> Result<String, String> {
        if ptr < 0 || len < 0 {
            return Err("invalid guest string bounds".into());
        }
        let memory = caller
            .get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or("guest missing memory export")?;
        let data = memory.data(caller);
        let start = ptr as usize;
        let end = start + len as usize;
        if end > data.len() {
            return Err("guest string out of bounds".into());
        }
        Ok(String::from_utf8_lossy(&data[start..end]).into_owned())
    }

    fn host_register_command(
        mut caller: Caller<'_, HostState>,
        id_ptr: i32,
        id_len: i32,
        title_ptr: i32,
        title_len: i32,
    ) -> i32 {
        let id = match read_guest_utf8(&mut caller, id_ptr, id_len) {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let title = match read_guest_utf8(&mut caller, title_ptr, title_len) {
            Ok(s) => s,
            Err(_) => return -2,
        };
        caller.data_mut().pending.commands.push((id, title));
        0
    }

    fn wasm_command_execute(
        _doc: &mut unibrowser_engine::document::DocumentSession,
    ) -> Result<String, String> {
        Ok("Executed WASM mod command".into())
    }

    fn host_register_extension(mut caller: Caller<'_, HostState>, ext_ptr: i32, ext_len: i32) -> i32 {
        let ext = match read_guest_utf8(&mut caller, ext_ptr, ext_len) {
            Ok(s) => s,
            Err(_) => return -1,
        };
        caller.data_mut().pending.extensions.push(ext);
        0
    }

    pub fn load_wasm_bytes(
        wasm_bytes: &[u8],
        mod_name: &str,
        registry: &mut CommandRegistry,
    ) -> Result<(), ModHostError> {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)
            .map_err(|e| ModHostError::Dynamic(format!("wasm parse: {}", e)))?;

        let pending = PendingWasmRegistrations {
            commands: Vec::new(),
            extensions: Vec::new(),
            mod_name: mod_name.to_string(),
        };
        let mut store = Store::new(&engine, HostState { pending });
        let mut linker = Linker::new(&engine);
        linker
            .func_wrap(
                "ub_host",
                "ub_host_register_command",
                host_register_command,
            )
            .map_err(|e| ModHostError::Dynamic(e.to_string()))?;
        linker
            .func_wrap("ub_host", "ub_host_register_extension", host_register_extension)
            .map_err(|e| ModHostError::Dynamic(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .and_then(|pre| pre.start(&mut store))
            .map_err(|e| ModHostError::Dynamic(format!("wasm instantiate: {}", e)))?;

        let abi = instance
            .get_typed_func::<(), i32>(&mut store, "ub_wasm_abi_version")
            .and_then(|f| f.call(&mut store, ()))
            .map_err(|e| ModHostError::Dynamic(format!("wasm abi: {}", e)))?;
        if abi != UB_WASM_ABI_VERSION {
            return Err(ModHostError::Dynamic(format!(
                "wasm ABI {} != host {}",
                abi,
                UB_WASM_ABI_VERSION
            )));
        }

        let code = instance
            .get_typed_func::<(), i32>(&mut store, "ub_wasm_register")
            .and_then(|f| f.call(&mut store, ()))
            .map_err(|e| ModHostError::Dynamic(format!("ub_wasm_register: {}", e)))?;
        if code != 0 {
            return Err(ModHostError::Dynamic(format!(
                "ub_wasm_register returned {}",
                code
            )));
        }

        let pending = &store.data().pending;
        for (id, title) in &pending.commands {
            let id_static: &'static str = Box::leak(id.clone().into_boxed_str());
            let title_static: &'static str = Box::leak(title.clone().into_boxed_str());
            registry.register_command(CommandMeta {
                id: id_static,
                title: title_static,
                icon: "extension",
                default_keybinding: None,
                aliases: &[],
            });
            let cmd_id = id.clone();
            let backend_id = format!("wasm:{}", pending.mod_name);
            registry.register_handler(HandlerRegistration {
                command_id: cmd_id,
                backend_id,
                enable: |_| Enablement::Enabled,
                execute: wasm_command_execute,
            });
        }
        if let Ok(mut map) = extension_map().write() {
            for ext in &pending.extensions {
                map.insert(ext.to_ascii_lowercase(), pending.mod_name.clone());
            }
        }
        Ok(())
    }
}

#[cfg(feature = "wasm-mods")]
pub use runtime::load_wasm_bytes;

pub fn resolve_wasm_module_path(mod_dir: &Path, manifest: &ModManifest) -> Option<PathBuf> {
    if let Some(name) = &manifest.wasm_module {
        let p = mod_dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    let stem = manifest.name.replace('-', "_");
    for candidate in [format!("{}.wasm", stem), "mod.wasm".to_string()] {
        let p = mod_dir.join(&candidate);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Load a WASM mod from `mod_dir` (`info.json` + module bytes).
pub fn load_wasm_mod(
    mod_dir: &Path,
    registry: &mut CommandRegistry,
) -> Result<String, ModHostError> {
    let info = mod_dir.join("info.json");
    if !info.is_file() {
        return Err(ModHostError::MissingManifest(mod_dir.to_path_buf()));
    }
    let text = std::fs::read_to_string(&info)?;
    let manifest: ModManifest = serde_json::from_str(&text)?;
    crate::loader::check_api_version(&manifest)?;
    let wasm_path = resolve_wasm_module_path(mod_dir, &manifest).ok_or_else(|| {
        ModHostError::Dynamic(format!("wasm module not found in {}", mod_dir.display()))
    })?;
    let bytes = std::fs::read(&wasm_path)?;
    #[cfg(feature = "wasm-mods")]
    {
        load_wasm_bytes(&bytes, &manifest.name, registry)?;
        return Ok(manifest.name);
    }
    #[cfg(not(feature = "wasm-mods"))]
    {
        let _ = bytes;
        Err(ModHostError::Dynamic(
            "WASM mod load requires wasm-mods feature (wasmi)".into(),
        ))
    }
}

/// Enabled user WASM mods under `user_root`.
pub fn load_user_wasm_mods(
    user_root: &Path,
    registry: &mut CommandRegistry,
) -> Result<Vec<String>, ModHostError> {
    let mut loaded = Vec::new();
    for rec in crate::native::list_installed(user_root)? {
        if !rec.enabled {
            continue;
        }
        crate::signing::verify_mod_dir(&rec.install_dir)?;
        let info = std::fs::read_to_string(rec.install_dir.join("info.json"))?;
        let manifest: ModManifest = serde_json::from_str(&info)?;
        if manifest.library.is_some() {
            continue;
        }
        if resolve_wasm_module_path(&rec.install_dir, &manifest).is_none() {
            continue;
        }
        let name = load_wasm_mod(&rec.install_dir, registry)?;
        loaded.push(name);
    }
    Ok(loaded)
}

pub fn example_ub_mod_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ub-mod-wasm/target/wasm32-unknown-unknown/debug/ub_mod_wasm.wasm")
}

#[cfg(all(test, feature = "wasm-mods"))]
mod tests {
    use super::*;
    use std::process::Command;
    use unibrowser_engine::CommandRegistry;

    fn ensure_example_wasm_built() -> Option<PathBuf> {
        let path = example_ub_mod_wasm_path();
        if path.is_file() {
            return Some(path);
        }
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ub-mod-wasm");
        let status = Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .current_dir(&manifest_dir)
            .status();
        match status {
            Ok(s) if s.success() && path.is_file() => Some(path),
            _ => None,
        }
    }

    #[test]
    fn loads_built_ub_mod_wasm() {
        let wasm_path = ensure_example_wasm_built();
        if wasm_path.is_none() {
            eprintln!("skip loads_built_ub_mod_wasm: wasm32 target or build unavailable");
            return;
        }
        let bytes = std::fs::read(wasm_path.unwrap()).expect("read wasm");
        let mut registry = CommandRegistry::with_builtins();
        load_wasm_bytes(&bytes, "ub-mod-wasm", &mut registry).expect("load wasm");
        assert!(registry.command_meta("mod.wasm.demo").is_some());
        let exts = wasm_file_extensions();
        assert_eq!(exts.get("ubwasm").as_deref(), Some(&"ub-mod-wasm".to_string()));
    }

    #[test]
    fn load_wasm_mod_from_example_dir() {
        let mod_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/ub-mod-wasm");
        let _ = ensure_example_wasm_built();
        if !example_ub_mod_wasm_path().is_file() {
            return;
        }
        std::fs::copy(
            example_ub_mod_wasm_path(),
            mod_dir.join("ub_mod_wasm.wasm"),
        )
        .expect("copy wasm beside info.json");
        let mut registry = CommandRegistry::with_builtins();
        let name = load_wasm_mod(&mod_dir, &mut registry).expect("load dir");
        assert_eq!(name, "ub-mod-wasm");
    }

    #[test]
    fn proving_path_when_examples_present() {
        if proving_hello_wasm_path().is_some() {
            assert!(proving_hello_wasm_available());
        }
    }
}
