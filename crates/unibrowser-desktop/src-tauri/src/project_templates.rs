use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, Window};
use unibrowser_mod_api::BrowserLocation;
use unibrowser_project::{
    apply_template_project, create_memory_package, export_package_to_zip, export_ubtemplate_pack,
    is_memory_project_root, list_builtin_template_dirs, load_template_pack_dir,
    load_user_template_index, memory_root_for_session, memory_session_id, save_user_template_index,
    write_manifest, TemplateManifest, TemplatePack, TemplateRelationships, TemplateSourceKind,
    UserTemplateIndex, UserTemplateIndexEntry, ProjectPackage, DIR_ASSETS, DIR_DERIVED, DIR_REFS,
};

use crate::commands::{snapshot, update_content_webview, TabContentDto, ws_label};
use crate::config;
use crate::project_backend::{UnibrowBackend, BACKEND_ID};
use crate::preferences::{load_preferences, push_recent_entry, save_preferences};
use crate::state::AppState;

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
pub struct ProjectTemplateSummaryDto {
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub source: String,
    pub default_workspace: Option<String>,
    pub relationships: TemplateRelationships,
    pub needs_mod_ids: Vec<String>,
    pub pack_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MemoryProjectEntry {
    pub package: ProjectPackage,
    pub assets: HashMap<String, Vec<u8>>,
}

pub fn builtin_templates_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../builtin-templates")
}

fn new_session_id() -> String {
    let n = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{:x}-{}", ms, n)
}

pub fn store_memory_project(state: &AppState, entry: MemoryProjectEntry) -> String {
    let id = memory_session_id(&entry.package.root)
        .unwrap_or_else(|| new_session_id());
    let mut map = state.memory_projects.lock().expect("memory_projects");
    map.insert(id.clone(), entry);
    id
}

pub fn memory_project_entry(state: &AppState, session_id: &str) -> Option<MemoryProjectEntry> {
    state
        .memory_projects
        .lock()
        .expect("memory_projects")
        .get(session_id)
        .cloned()
}

pub fn open_memory_package(state: &AppState, path: &Path) -> Result<ProjectPackage, String> {
    let id = memory_session_id(path).ok_or("invalid memory project path")?;
    let map = state.memory_projects.lock().map_err(|e| e.to_string())?;
    let entry = map.get(&id).ok_or("memory project session not found")?;
    Ok(entry.package.clone())
}

fn list_templates_from_dir(
    root: &Path,
    source: TemplateSourceKind,
) -> Result<Vec<ProjectTemplateSummaryDto>, String> {
    let mut out = Vec::new();
    for dir in list_builtin_template_dirs(root) {
        let pack = load_template_pack_dir(&dir, source).map_err(|e| e.to_string())?;
        out.push(summary_from_pack(&pack));
    }
    Ok(out)
}

fn summary_from_pack(pack: &TemplatePack) -> ProjectTemplateSummaryDto {
    let needs_mod_ids = pack
        .manifest
        .relationships
        .requires
        .keys()
        .cloned()
        .collect();
    ProjectTemplateSummaryDto {
        id: pack.manifest.id.clone(),
        version: pack.manifest.version.clone(),
        title: pack.manifest.title.clone(),
        description: pack.manifest.description.clone(),
        categories: pack.manifest.categories.clone(),
        tags: pack.manifest.tags.clone(),
        source: source_label(pack.manifest.source),
        default_workspace: pack.manifest.default_workspace.clone(),
        relationships: pack.manifest.relationships.clone(),
        needs_mod_ids,
        pack_path: pack.manifest.pack_path.clone(),
    }
}

fn source_label(kind: TemplateSourceKind) -> String {
    match kind {
        TemplateSourceKind::Builtin => "builtin".into(),
        TemplateSourceKind::User => "user".into(),
        TemplateSourceKind::Installed => "installed".into(),
    }
}

pub fn list_all_project_templates(state: &AppState) -> Result<Vec<ProjectTemplateSummaryDto>, String> {
    let mut all = list_templates_from_dir(&builtin_templates_root(), TemplateSourceKind::Builtin)?;
    if let Some(cfg) = config::config_dir() {
        let user_index = load_user_template_index(&cfg);
        for entry in user_index.entries {
            let path = PathBuf::from(&entry.path);
            if path.is_dir() {
                if let Ok(pack) = load_template_pack_dir(&path, TemplateSourceKind::User) {
                    all.push(summary_from_pack(&pack));
                }
            }
        }
        let installed_root = cfg.join("templates/installed");
        if installed_root.is_dir() {
            for id_dir in list_builtin_template_dirs(&installed_root) {
                for ver_dir in list_builtin_template_dirs(&id_dir) {
                    if let Ok(pack) =
                        load_template_pack_dir(&ver_dir, TemplateSourceKind::Installed)
                    {
                        all.push(summary_from_pack(&pack));
                    }
                }
            }
        }
    }
    Ok(all)
}

fn resolve_template_pack(state: &AppState, template_id: &str) -> Result<TemplatePack, String> {
    let builtin = builtin_templates_root();
    for dir in list_builtin_template_dirs(&builtin) {
        let pack = load_template_pack_dir(&dir, TemplateSourceKind::Builtin).map_err(|e| e.to_string())?;
        if pack.manifest.id == template_id {
            return Ok(pack);
        }
    }
    if let Some(cfg) = config::config_dir() {
        let user_index = load_user_template_index(&cfg);
        for entry in user_index.entries {
            if entry.id == template_id {
                let path = PathBuf::from(&entry.path);
                return load_template_pack_dir(&path, TemplateSourceKind::User)
                    .map_err(|e| e.to_string());
            }
        }
        let installed_root = cfg.join("templates/installed");
        if installed_root.join(template_id.replace('.', "/")).is_dir() {
            // flat id dirs: templates/installed/<id>/<version>/
        }
        for id_dir in list_builtin_template_dirs(&installed_root) {
            for ver_dir in list_builtin_template_dirs(&id_dir) {
                let pack = load_template_pack_dir(&ver_dir, TemplateSourceKind::Installed)
                    .map_err(|e| e.to_string())?;
                if pack.manifest.id == template_id {
                    return Ok(pack);
                }
            }
        }
    }
    Err(format!("template not found: {}", template_id))
}

fn tab_is_new_tab_home(tab: &unibrowser_engine::tabs::Tab) -> bool {
    matches!(
        &tab.location,
        BrowserLocation::Internal { page } if page == "home"
    )
}

fn open_memory_project_tab(
    state: &AppState,
    window: &Window,
    app: &AppHandle,
    session_id: &str,
    _title: &str,
) -> Result<TabContentDto, String> {
    let root = memory_root_for_session(session_id);
    let doc = state.try_open_unibrow_document(&root)?;
    let loc = BrowserLocation::File { path: root };
    let _sess_tabs = ws_label(state, window.label())?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        let reuse_home = tabs.tab_count() <= 1
            || tabs
                .active_tab()
                .map(tab_is_new_tab_home)
                .unwrap_or(false);
        if reuse_home {
            tabs.navigate_active(loc.clone(), false, Some(doc));
        } else {
            tabs.open_tab(loc.clone());
            if let Some(tab) = tabs.active_tab_mut() {
                tab.is_directory = false;
                tab.document = Some(doc);
            }
        }
    }
    update_content_webview(app, window.label(), &loc)?;
    snapshot(state, window.label())
}

#[tauri::command]
pub fn new_empty_project(
    window: Window,
    state: State<AppState>,
    app: AppHandle,
    title: Option<String>,
) -> Result<TabContentDto, String> {
    let title = title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| "Untitled project".into());
    let session_id = new_session_id();
    let package = create_memory_package(&title, &session_id);
    store_memory_project(
        state.inner(),
        MemoryProjectEntry {
            package,
            assets: HashMap::new(),
        },
    );
    open_memory_project_tab(state.inner(), &window, &app, &session_id, &title)
}

#[tauri::command]
pub fn new_project_from_template(
    window: Window,
    state: State<AppState>,
    app: AppHandle,
    template_id: String,
    project_name: Option<String>,
    _location: Option<String>,
) -> Result<TabContentDto, String> {
    let pack = resolve_template_pack(state.inner(), &template_id)?;
    let title = project_name
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| pack.manifest.title.clone());
    let session_id = new_session_id();
    let mut package = create_memory_package(&title, &session_id);
    let pin = format!("{}@{}", pack.manifest.id, pack.manifest.version);
    apply_template_project(&mut package, &pack, Some(&pin)).map_err(|e| e.to_string())?;
    store_memory_project(
        state.inner(),
        MemoryProjectEntry {
            package,
            assets: pack.assets.clone(),
        },
    );
    open_memory_project_tab(state.inner(), &window, &app, &session_id, &title)
}

#[tauri::command]
pub fn list_project_templates(state: State<AppState>) -> Result<Vec<ProjectTemplateSummaryDto>, String> {
    list_all_project_templates(state.inner())
}

#[tauri::command]
pub fn save_project_as_template(
    project_path: String,
    dest_path: String,
    title: Option<String>,
    description: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let path = PathBuf::from(project_path);
    let package = if is_memory_project_root(&path) {
        open_memory_package(state.inner(), &path)?
    } else {
        crate::commands::open_unibrow_package(state.inner(), &path)?
    };
    let dest = PathBuf::from(dest_path);
    let manifest = TemplateManifest {
        template_schema_version: 1,
        id: format!(
            "user.template.{}",
            package
                .manifest
                .title
                .to_ascii_lowercase()
                .replace(' ', "-")
        ),
        version: "1.0.0".into(),
        title: title.unwrap_or_else(|| package.manifest.title.clone()),
        description: description.unwrap_or_else(|| "User template".into()),
        categories: vec!["document".into()],
        tags: vec![],
        relationships: TemplateRelationships::default(),
        default_workspace: None,
        min_app_version: None,
        hub: None,
        source: TemplateSourceKind::User,
        pack_path: None,
    };
    let assets_dir = package.root.join("assets");
    let asset_files: Vec<(String, PathBuf)> = if assets_dir.is_dir() {
        std::fs::read_dir(&assets_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                (name, e.path())
            })
            .collect()
    } else {
        Vec::new()
    };
    export_ubtemplate_pack(&dest, &manifest, &package.manifest, &asset_files)
        .map_err(|e| e.to_string())?;
    if let Some(cfg) = config::config_dir() {
        let mut index = load_user_template_index(&cfg);
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        index.entries.retain(|e| e.id != manifest.id);
        index.entries.push(UserTemplateIndexEntry {
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            title: manifest.title.clone(),
            path: dest.to_string_lossy().into_owned(),
            added_at_ms: ms,
        });
        save_user_template_index(&cfg, &index).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pick_save_unibrow_project_path(
    _window: Window,
    app: AppHandle,
    suggested_name: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Save project")
        .set_file_name(suggested_name)
        .add_filter("Unibrowser project", &["unibrow"])
        .blocking_save_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn pick_save_ubtemplate_path(
    _window: Window,
    app: AppHandle,
    suggested_name: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Save template")
        .set_file_name(suggested_name)
        .add_filter("Unibrowser template", &["ubtemplate"])
        .blocking_save_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn remove_user_template(template_id: String) -> Result<(), String> {
    let cfg = config::config_dir().ok_or("config dir unavailable")?;
    let mut index = load_user_template_index(&cfg);
    index.entries.retain(|e| e.id != template_id);
    save_user_template_index(&cfg, &index).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_package_tree(
    package: &ProjectPackage,
    assets: &HashMap<String, Vec<u8>>,
    dest_root: &Path,
) -> Result<(), String> {
    fs::create_dir_all(dest_root.join(DIR_ASSETS)).map_err(|e| e.to_string())?;
    fs::create_dir_all(dest_root.join(DIR_REFS)).map_err(|e| e.to_string())?;
    fs::create_dir_all(dest_root.join(DIR_DERIVED)).map_err(|e| e.to_string())?;
    write_manifest(dest_root, &package.manifest).map_err(|e| e.to_string())?;
    for (name, bytes) in assets {
        fs::write(dest_root.join(DIR_ASSETS).join(name), bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn materialize_memory_to_archive(
    package: &ProjectPackage,
    assets: &HashMap<String, Vec<u8>>,
    dest_zip: &Path,
) -> Result<(), String> {
    let parent = dest_zip
        .parent()
        .ok_or("invalid archive parent directory")?;
    let staging = parent.join(format!(
        ".ub-staging-{}",
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| e.to_string())?;
    }
    write_package_tree(package, assets, &staging)?;
    if dest_zip.exists() {
        fs::remove_file(dest_zip).map_err(|e| e.to_string())?;
    }
    export_package_to_zip(&staging, dest_zip).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

fn materialize_memory_to_folder(
    package: &ProjectPackage,
    assets: &HashMap<String, Vec<u8>>,
    dest_folder: &Path,
) -> Result<(), String> {
    if dest_folder.exists() {
        return Err(format!("destination already exists: {}", dest_folder.display()));
    }
    write_package_tree(package, assets, dest_folder)?;
    Ok(())
}

fn active_memory_project(
    state: &AppState,
    window: &Window,
) -> Result<(ProjectPackage, HashMap<String, Vec<u8>>, PathBuf), String> {
    let _sess = ws_label(state, window.label())?;
    let mut tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no active tab")?;
    let doc = tab.document.as_mut().ok_or("no project open")?;
    if doc.backend.backend_id() != BACKEND_ID {
        return Err("active tab is not a project".into());
    }
    let memory_path = PathBuf::from(&doc.path);
    if !is_memory_project_root(&memory_path) {
        return Err("project is already saved to disk".into());
    }
    let session_id = memory_session_id(&memory_path).ok_or("invalid memory session")?;
    let backend = doc
        .backend
        .as_any_mut()
        .downcast_mut::<UnibrowBackend>()
        .ok_or("project backend unavailable")?;
    let assets = state
        .memory_projects
        .lock()
        .map_err(|e| e.to_string())?
        .get(&session_id)
        .map(|e| e.assets.clone())
        .unwrap_or_default();
    if let Some(entry) = state
        .memory_projects
        .lock()
        .map_err(|e| e.to_string())?
        .get_mut(&session_id)
    {
        entry.package = backend.package.clone();
    }
    Ok((backend.package.clone(), assets, memory_path))
}

#[derive(Debug, Deserialize)]
pub struct SaveMemoryProjectInput {
    pub dest_path: String,
    /// `archive` (.unibrow file) or `folder` (unpacked package directory).
    pub mode: String,
}

#[tauri::command]
pub fn save_memory_project(
    window: Window,
    state: State<AppState>,
    app: AppHandle,
    input: SaveMemoryProjectInput,
) -> Result<TabContentDto, String> {
    let (package, assets, memory_path) = active_memory_project(state.inner(), &window)?;
    let session_id = memory_session_id(&memory_path).ok_or("invalid memory session")?;
    let dest = PathBuf::from(input.dest_path.trim());
    let saved_path = if input.mode.eq_ignore_ascii_case("folder") {
        materialize_memory_to_folder(&package, &assets, &dest)?;
        dest
    } else {
        materialize_memory_to_archive(&package, &assets, &dest)?;
        dest
    };
    {
        let mut map = state.memory_projects.lock().map_err(|e| e.to_string())?;
        map.remove(&session_id);
    }
    let loc = BrowserLocation::File {
        path: saved_path.clone(),
    };
    let doc = state.inner().try_open_unibrow_document(&saved_path)?;
    let _sess = ws_label(state.inner(), window.label())?;
    {
        let mut tabs = _sess.tabs.lock().map_err(|e| e.to_string())?;
        if let Some(tab) = tabs.active_tab_mut() {
            tab.location = loc.clone();
            tab.document = Some(doc);
            tab.is_directory = saved_path.is_dir();
        }
    }
    let label = package.manifest.title.clone();
    let mut prefs = load_preferences();
    push_recent_entry(
        &mut prefs,
        saved_path.to_string_lossy().into_owned(),
        label,
        Some("Project".into()),
    );
    let _ = save_preferences(&prefs);
    update_content_webview(&app, window.label(), &loc)?;
    snapshot(state.inner(), window.label())
}

#[tauri::command]
pub fn list_hub_template_catalog() -> Vec<crate::mod_portal::HubTemplateCatalogEntryDto> {
    crate::mod_portal::hub_template_catalog()
}

#[tauri::command]
pub fn install_hub_template_pack(fixture_id: String) -> Result<crate::mod_portal::InstalledHubTemplateDto, String> {
    crate::mod_portal::install_hub_template_fixture(&fixture_id)
}

#[tauri::command]
pub fn list_installed_hub_template_packs(
) -> Result<Vec<crate::mod_portal::InstalledHubTemplateDto>, String> {
    crate::mod_portal::list_installed_hub_templates()
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    use unibrowser_project::create_memory_package;

    #[test]
    fn empty_memory_package_writes_no_disk_files() {
        let pkg = create_memory_package("Untitled project", "sess-test");
        assert!(is_memory_project_root(&pkg.root));
        assert!(!pkg.root.join("project.json").exists());
    }

    #[test]
    fn materialize_archive_round_trip() {
        let pkg = create_memory_package("Save me", "sess-save");
        let dir = std::env::temp_dir().join(format!("ub-mem-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let zip = dir.join("Untitled.unibrow");
        materialize_memory_to_archive(&pkg, &HashMap::new(), &zip).expect("zip");
        assert!(zip.is_file());
        let _ = fs::remove_dir_all(&dir);
    }
}
