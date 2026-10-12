//! Open documents through the surface capability provider registry (SKEL-04).

use std::path::Path;

use unibrowser_engine::{
    CapabilityPrefs, SurfaceProviderRegistry, capability_for_effective_extension,
    resolve_surface_provider,
};

use crate::preferences::Preferences;
use crate::state::{is_known_text_extension, FileOpenMode};

pub fn capability_prefs_from_preferences(prefs: &Preferences) -> CapabilityPrefs {
    CapabilityPrefs {
        default_by_capability: prefs.capability_default_providers.clone(),
        open_with_by_extension: prefs.open_with_by_extension.clone(),
        open_with_by_path: prefs.open_with_by_path.clone(),
    }
}

pub fn open_backend_for_path(
    registry: &SurfaceProviderRegistry,
    prefs: &Preferences,
    path: &Path,
    effective_ext: &str, // sniffed extension token
    mode: FileOpenMode,
) -> Option<Box<dyn unibrowser_engine::document::DocumentBackend>> {
    if matches!(mode, FileOpenMode::PlainText) {
        return base_hex::HexBackend::open_path_as_text(path)
            .ok()
            .map(|b| Box::new(b) as Box<dyn unibrowser_engine::document::DocumentBackend>);
    }
    if matches!(mode, FileOpenMode::Hex) {
        return base_hex::HexBackend::open_path(path)
            .ok()
            .map(|b| Box::new(b) as Box<dyn unibrowser_engine::document::DocumentBackend>);
    }

    let cap = capability_for_effective_extension(effective_ext)
        .or_else(|| {
            if is_known_text_extension(effective_ext) {
                Some(unibrowser_mod_api::capability_surface::ids::OPEN_TEXT)
            } else {
                None
            }
        });

    if let Some(capability_id) = cap {
        let cap_prefs = capability_prefs_from_preferences(prefs);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        let resolved = resolve_surface_provider(
            registry,
            capability_id,
            &cap_prefs,
            Some(path),
            ext.as_deref(),
        );
        if let Some(mod_name) = resolved.mod_name {
            if let Some(backend) = open_via_provider_mod(&mod_name, path, effective_ext) {
                return Some(backend);
            }
            eprintln!(
                "capability_router: provider {} for {} failed to open {}; using legacy fallback",
                mod_name,
                capability_id,
                path
            );
        } else {
            eprintln!(
                "capability_router: no provider for {} ({}) on {}; using legacy fallback",
                capability_id,
                resolved.reason,
                path
            );
        }
        return legacy_open_backend(path, effective_ext);
    }

    legacy_open_backend(path, effective_ext)
}

fn open_via_provider_mod(
    mod_name: &str,
    path: &Path,
    effective_ext: &str, // sniffed extension token
) -> Option<Box<dyn unibrowser_engine::document::DocumentBackend>> {
    match mod_name {
        "base-text" => base_text::open_file(path).ok(),
        "base-hex" => base_hex::HexBackend::open_path_as_text(path)
            .ok()
            .map(|b| Box::new(b) as Box<dyn unibrowser_engine::document::DocumentBackend>),
        "toy-markdown" if effective_ext == "md" || effective_ext == "markdown" => {
            Some(toy_markdown::open_sample(path.to_string_lossy().as_ref()))
        }
        "base-png" if matches!(effective_ext, "png" | "jpg" | "jpeg" | "gif" | "webp") => {
            Some(base_png::open_file(path))
        }
        "base-svg" if effective_ext == "svg" => Some(base_svg::open_file(path)),
        "base-docx" if effective_ext == "docx" => base_docx::open_file(path).ok(),
        "base-pdf" if effective_ext == "pdf" => base_pdf::open_file(path).ok(),
        "base-dxf" if effective_ext == "dxf" => base_dxf::open_file(path).ok(),
        "base-js" if matches!(effective_ext, "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx") => {
            base_js::open_file(path).ok()
        }
        "base-video" if matches!(effective_ext, "mp4" | "webm" | "avi") => {
            base_video::open_file(path).ok()
        }
        "base-audio" if matches!(effective_ext, "wav" | "mp3" | "ogg" | "flac") => {
            base_audio::open_file(path).ok()
        }
        "base-epub" if effective_ext == "epub" => Some(base_epub::open_file(path)),
        "base-csv" if matches!(effective_ext, "csv" | "tsv") => Some(base_csv::open_file(path)),
        _ => None,
    }
}

/// Legacy routing for hosts without registry coverage (MIDI, archives, WASM extensions)
/// and when a resolved provider cannot open the file.
fn legacy_open_backend(
    path: &Path,
    effective_ext: &str, // sniffed extension token
) -> Option<Box<dyn unibrowser_engine::document::DocumentBackend>> {
    match effective_ext {
        "mid" | "midi" => crate::midi_stub::open_file(path).ok(),
        "zip" | "7z" | "rar" | "tar" | "gz" => None,
        _ if is_known_text_extension(effective_ext) => base_text::open_file(path).ok(),
        _ => {
            if capability_for_effective_extension(effective_ext).is_some() {
                legacy_builtin_open(path, effective_ext)
            } else {
                unibrowser_mod_host::wasm::open_wasm_extension_backend(path)
            }
        }
    }
}

fn legacy_builtin_open(
    path: &Path,
    effective_ext: &str,
) -> Option<Box<dyn unibrowser_engine::document::DocumentBackend>> {
    match effective_ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" => Some(base_png::open_file(path)),
        "svg" => Some(base_svg::open_file(path)),
        "docx" => base_docx::open_file(path).ok(),
        "pdf" => base_pdf::open_file(path).ok(),
        "dxf" => base_dxf::open_file(path).ok(),
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" => base_js::open_file(path).ok(),
        "rhai" => base_text::open_file(path).ok(),
        "csv" | "tsv" => Some(base_csv::open_file(path)),
        "txt" | "md" | "json" | "rs" | "toml" | "html" | "css" => base_text::open_file(path).ok(),
        "mp4" | "webm" | "avi" => base_video::open_file(path).ok(),
        "wav" | "mp3" | "ogg" | "flac" => base_audio::open_file(path).ok(),
        "epub" => Some(base_epub::open_file(path)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_engine::build_surface_provider_registry;
    use unibrowser_mod_api::{CapabilityProvideEntry, ModManifest};

    fn manifest(name: &str, cap: &str) -> ModManifest {
        ModManifest {
            name: name.into(),
            version: "0.1.0".into(),
            title: name.into(),
            author: String::new(),
            unibrowser_api: "0.1.0".into(),
            dependencies: Default::default(),
            optional_dependencies: Default::default(),
            incompatibilities: Default::default(),
            provides: Default::default(),
            api_dependencies: Default::default(),
            description: String::new(),
            library: None,
            wasm_module: None,
            capabilities: Vec::new(),
            capability_provides: vec![CapabilityProvideEntry {
                capability: cap.into(),
                default: true,
                priority: 100,
                content_kinds: vec![],
            }],
            contributes: None,
        }
    }

    #[test]
    fn registry_first_png_open_uses_base_png() {
        let reg = build_surface_provider_registry(&[manifest(
            "base-png",
            unibrowser_mod_api::capability_surface::ids::RENDER_IMAGE,
        )]);
        let prefs = Preferences::default();
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("x.png");
        std::fs::write(&path, b"\x89PNG\r\n\x1a\n").expect("write");
        let backend = open_backend_for_path(&reg, &prefs, &path, "png", FileOpenMode::Auto);
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().backend_id(), "base-png");
    }
}
