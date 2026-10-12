use serde::Serialize;
use tauri::State;

use unibrowser_engine::resolve_surface_provider;
use unibrowser_mod_api::capability_surface::ids;

use crate::commands::{snapshot, TabContentDto};
use crate::preferences::{load_preferences, save_preferences, Preferences};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceProviderDto {
    pub mod_name: String,
    pub mod_title: String,
    pub capability_id: String,
    pub default: bool,
    pub priority: i32,
    pub content_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceCapabilityGroupDto {
    pub capability_id: String,
    pub label: String,
    pub providers: Vec<SurfaceProviderDto>,
    pub user_default: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenWithOptionDto {
    pub mod_name: String,
    pub mod_title: String,
    pub active: bool,
}

fn capability_label(id: &str) -> String {
    match id {
        ids::OPEN_TEXT => "Plain text editing".into(),
        ids::RENDER_PDF => "PDF viewing".into(),
        ids::RENDER_IMAGE => "Raster images".into(),
        ids::RENDER_SVG => "SVG graphics".into(),
        ids::OPEN_DOCX => "Word (DOCX)".into(),
        ids::OPEN_DXF => "CAD (DXF)".into(),
        ids::OPEN_VIDEO => "Video playback".into(),
        ids::OPEN_AUDIO => "Audio playback".into(),
        ids::OPEN_EPUB => "EPUB ebooks".into(),
        ids::OPEN_CSV => "CSV / TSV tables".into(),
        ids::OPEN_SCRIPT => "Scripts (JS/TS)".into(),
        ids::PROJECT_SURFACE => "Native project".into(),
        ids::BROWSE_FILES => "File browsing".into(),
        other => other.replace("unibrowser.capability.", "").replace('_', " "),
    }
}

pub fn surface_provider_dtos(state: &AppState) -> Vec<SurfaceProviderDto> {
    state
        .surface_registry()
        .providers
        .iter()
        .map(|p| SurfaceProviderDto {
            mod_name: p.mod_name.clone(),
            mod_title: p.mod_title.clone(),
            capability_id: p.capability_id.clone(),
            default: p.default,
            priority: p.priority,
            content_kinds: p.content_kinds.clone(),
        })
        .collect()
}

pub fn surface_capability_groups(state: &AppState) -> Vec<SurfaceCapabilityGroupDto> {
    let prefs = load_preferences();
    let reg = state.surface_registry();
    let mut caps: Vec<String> = reg
        .providers
        .iter()
        .map(|p| p.capability_id.clone())
        .collect();
    caps.sort();
    caps.dedup();
    caps.into_iter()
        .map(|capability_id| {
            let providers = reg
                .providers
                .iter()
                .filter(|p| p.capability_id == capability_id)
                .map(|p| SurfaceProviderDto {
                    mod_name: p.mod_name.clone(),
                    mod_title: p.mod_title.clone(),
                    capability_id: p.capability_id.clone(),
                    default: p.default,
                    priority: p.priority,
                    content_kinds: p.content_kinds.clone(),
                })
                .collect::<Vec<_>>();
            let user_default = prefs.capability_default_providers.get(&capability_id).cloned();
            SurfaceCapabilityGroupDto {
                label: capability_label(&capability_id),
                capability_id,
                providers,
                user_default,
            }
        })
        .collect()
}

#[tauri::command]
pub fn list_surface_capability_groups(
    state: State<AppState>,
) -> Vec<SurfaceCapabilityGroupDto> {
    surface_capability_groups(&state)
}

#[tauri::command]
pub fn set_capability_default_provider(
    capability_id: String,
    mod_name: Option<String>,
) -> Result<Preferences, String> {
    let mut prefs = load_preferences();
    if let Some(name) = mod_name {
        prefs
            .capability_default_providers
            .insert(capability_id, name);
    } else {
        prefs.capability_default_providers.remove(&capability_id);
    }
    save_preferences(&prefs)?;
    Ok(prefs)
}

#[tauri::command]
pub fn list_open_with_providers(
    path: String,
    state: State<AppState>,
) -> Result<Vec<OpenWithOptionDto>, String> {
    let file = crate::path_security::resolve_open_path(&path, &state.cwd)?;
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let effective = crate::state::effective_extension_for_path(file.as_path(), &ext);
    let capability = unibrowser_engine::capability_for_effective_extension(&effective)
        .or_else(|| {
            if crate::state::is_known_text_extension(&effective) {
                Some(ids::OPEN_TEXT)
            } else {
                None
            }
        })
        .ok_or("No surface capability for this file type")?;
    let reg = state.surface_registry();
    let prefs = load_preferences();
    let resolved = resolve_surface_provider(
        &reg,
        capability,
        &crate::capability_router::capability_prefs_from_preferences(&prefs),
        Some(file.as_path()),
        Some(&ext),
    );
    let active = resolved.mod_name.clone();
    let providers = reg
        .providers
        .iter()
        .filter(|p| p.capability_id == capability)
        .map(|p| OpenWithOptionDto {
            mod_name: p.mod_name.clone(),
            mod_title: p.mod_title.clone(),
            active: active.as_deref() == Some(p.mod_name.as_str()),
        })
        .collect();
    Ok(providers)
}

#[tauri::command]
pub fn open_path_with_provider(
    window: tauri::Window,
    path: String,
    mod_name: String,
    remember: Option<String>,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let file = crate::path_security::resolve_open_path(&path, &state.cwd)?;
    {
        let mut prefs = load_preferences();
        let path_key = file.to_string_lossy().into_owned();
        prefs.open_with_by_path.insert(path_key, mod_name.clone());
        if remember.as_deref() == Some("extension") {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                prefs
                    .open_with_by_extension
                    .insert(ext.to_ascii_lowercase(), mod_name.clone());
            }
        }
        save_preferences(&prefs)?;
    }
    crate::commands::allow_asset_protocol_file(&app, &file);
    let loc = unibrowser_mod_api::BrowserLocation::File {
        path: file.clone(),
    };
    let doc = state.open_file_document(&file).ok_or("could not open with provider")?;
    let _sess = crate::commands::ws(&state, &window)?;
    {
        let mut tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
        tabs.navigate_active(loc.clone(), false, Some(doc));
    }
    crate::commands::update_content_webview(&app, window.label(), &loc)?;
    snapshot(&state, window.label())
}

#[derive(Debug, Clone, Serialize)]
pub struct BundledModPluginDto {
    pub name: String,
    pub title: String,
    pub capabilities: Vec<String>,
}

#[tauri::command]
pub fn list_bundled_mod_plugins(state: State<AppState>) -> Vec<BundledModPluginDto> {
    let reg = state.surface_registry();
    state
        ._mod_host
        .mods
        .iter()
        .map(|loaded| {
            let capabilities = reg
                .providers
                .iter()
                .filter(|p| p.mod_name == loaded.manifest.name)
                .map(|p| p.capability_id.clone())
                .collect();
            BundledModPluginDto {
                name: loaded.manifest.name.clone(),
                title: loaded.manifest.title.clone(),
                capabilities,
            }
        })
        .collect()
}
