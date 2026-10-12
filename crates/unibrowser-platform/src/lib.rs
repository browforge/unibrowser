//! Host platform surface (filesystem, dialogs, windowing, process).
//! Desktop implements these in Tauri; web implements in TypeScript + browser APIs.

use serde::{Deserialize, Serialize};

/// Result of a save dialog or download disposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDisposition {
    pub path_label: String,
    pub used_native_picker: bool,
}

/// Read-only metadata the shell may show in diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub can_spawn_process: bool,
    pub can_embed_https_webview: bool,
    pub can_load_native_mods: bool,
    pub can_load_wasm_mods: bool,
}

pub const DESKTOP: PlatformInfo = PlatformInfo {
    id: "desktop",
    display_name: "Tauri desktop",
    can_spawn_process: true,
    can_embed_https_webview: true,
    can_load_native_mods: true,
    can_load_wasm_mods: true,
};

pub const WEB: PlatformInfo = PlatformInfo {
    id: "web",
    display_name: "Static web (WASM)",
    can_spawn_process: false,
    can_embed_https_webview: false,
    can_load_native_mods: false,
    can_load_wasm_mods: true,
};

/// Filesystem operations the engine must not call directly on web (host mediates).
pub trait HostFilesystem {
    fn read_text(&self, path: &str) -> Result<String, String>;
    fn write_text(&self, path: &str, content: &str) -> Result<(), String>;
}

/// User-visible pickers (web: File System Access API or hidden input).
pub trait HostDialogs {
    fn pick_open_files(&self, extensions: &[&str]) -> Result<Vec<String>, String>;
    fn pick_save_path(&self, suggested_name: &str) -> Result<Option<String>, String>;
}

/// Window chrome (web: DOM-only; no multi-window in v1).
pub trait HostWindowing {
    fn set_title(&self, title: &str);
}

/// Child processes and sidecars (desktop only in v1).
pub trait HostProcess {
    fn spawn_sidecar(&self, _name: &str, _args: &[String]) -> Result<(), String> {
        Err("Not available in this platform build".into())
    }
}
