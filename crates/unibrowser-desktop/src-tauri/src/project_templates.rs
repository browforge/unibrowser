use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, State, Window};
use unibrowser_mod_api::BrowserLocation;
use unibrowser_project::{
    apply_template_project, create_memory_package, export_ubtemplate_pack, is_memory_project_root,
    list_builtin_template_dirs, load_template_pack_dir, load_user_template_index,
    memory_root_for_session, memory_session_id, save_user_template_index, TemplateManifest,
    TemplatePack, TemplateRelationships, TemplateSourceKind, UserTemplateIndex,
    UserTemplateIndexEntry, ProjectPackage,
};

use crate::commands::{snapshot, update_content_webview, TabContentDto, ws_label};
use crate::config;
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
        tabs.open_tab(loc.clone());
        if let Some(tab) = tabs.active_tab_mut() {
            tab.is_directory = false;
            tab.document = Some(doc);
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
pub fn remove_user_template(template_id: String) -> Result<(), String> {
    let cfg = config::config_dir().ok_or("config dir unavailable")?;
    let mut index = load_user_template_index(&cfg);
    index.entries.retain(|e| e.id != template_id);
    save_user_template_index(&cfg, &index).map_err(|e| e.to_string())?;
    Ok(())
}
