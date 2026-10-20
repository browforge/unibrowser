//! Tauri IPC for package storage modes (PKG-03..06).

use std::path::Path;

use tauri::{AppHandle, State};
use unibrowser_project::PackageStorageMode;

use crate::commands::{
    allow_asset_protocol_file, snapshot, touch_recent_file, update_content_webview, ws, TabContentDto,
};
use crate::package_storage_session::{
    cancel_unpack_active, open_with_storage_mode, pkg_capture_fixture_archive_path,
    pkg_capture_unpack_dest_path, query_unibrow_open, run_pack_active, run_unpack_active,
    session_to_dto,
};
use crate::preferences::{load_preferences, save_preferences};
use crate::state::AppState;
use unibrowser_mod_api::BrowserLocation;

#[tauri::command]
pub fn query_unibrow_open_cmd(path: String, state: State<AppState>) -> Result<unibrowser_project::UnibrowOpenEstimate, String> {
    let p = crate::path_security::resolve_open_path(&path, &state.cwd)?;
    let prefs = load_preferences();
    query_unibrow_open(&p, prefs.project_folder_hint_bytes)
}

#[tauri::command]
pub fn open_unibrow_with_storage_mode(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
    path: String,
    mode: String,
    folder_dest: Option<String>,
) -> Result<TabContentDto, String> {
    use crate::project_backend::UnibrowBackend;
    let file_path = crate::path_security::resolve_open_path(&path, &state.cwd)?;
    let storage_mode = match mode.as_str() {
        "view_only" => PackageStorageMode::ViewOnly,
        "archive" => PackageStorageMode::Archive,
        "folder" => PackageStorageMode::Folder,
        _ => return Err("invalid storage mode".into()),
    };
    let folder = folder_dest
        .as_ref()
        .map(|d| crate::path_security::resolve_open_path(d, &state.cwd))
        .transpose()?;
    let prefs = load_preferences();
    let (pkg, _session) = open_with_storage_mode(
        state.inner(),
        &file_path,
        &file_path,
        storage_mode,
        folder.as_deref(),
        prefs.project_folder_hint_bytes,
    )?;
    allow_asset_protocol_file(&app, &file_path);
    let doc = unibrowser_engine::DocumentSession::new(
        file_path.to_string_lossy().into_owned(),
        Box::new(UnibrowBackend::open(pkg)),
    );
    let loc = BrowserLocation::File {
        path: file_path.clone(),
    };
    let _sess_tabs = ws(&state, &window)?;
    {
        let mut tabs = _sess_tabs.tabs.lock().map_err(|e| e.to_string())?;
        tabs.navigate_active(loc.clone(), false, Some(doc));
    }
    touch_recent_file(&file_path);
    if !crate::launch_args::is_doc_capture_mode() {
        update_content_webview(&app, window.label(), &loc)?;
    }
    snapshot(state.inner(), window.label())
}

#[tauri::command]
pub fn dismiss_package_history_notice(
    window: tauri::Window,
    state: State<AppState>,
    project_path: String,
) -> Result<TabContentDto, String> {
    state
        .package_storage
        .update(&project_path, |s| s.history_notice_dismissed = true);
    snapshot(state.inner(), window.label())
}

#[tauri::command]
pub fn cancel_unpack_unibrow(
    state: State<AppState>,
    project_path: String,
) -> Result<(), String> {
    cancel_unpack_active(state.inner(), &project_path)
}

#[tauri::command]
pub fn pkg_capture_fixture_archive_path_cmd(state: State<AppState>) -> Result<String, String> {
    pkg_capture_fixture_archive_path(state.inner())
}

#[tauri::command]
pub fn pkg_capture_unpack_dest_path_cmd(state: State<AppState>) -> Result<String, String> {
    pkg_capture_unpack_dest_path(state.inner())
}

#[tauri::command]
pub fn pkg_capture_seed_archive_session(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
) -> Result<TabContentDto, String> {
    let path = pkg_capture_fixture_archive_path(state.inner())?;
    open_unibrow_with_storage_mode(
        window,
        app,
        state,
        path,
        "archive".into(),
        None,
    )
}

#[tauri::command]
pub fn unpack_unibrow_to_folder(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
    project_path: String,
    destination_dir: String,
) -> Result<TabContentDto, String> {
    let dest = crate::path_security::resolve_open_path(&destination_dir, &state.cwd)?;
    let root = run_unpack_active(state.inner(), &project_path, &dest)?;
    let _ = root;
    update_content_webview(
        &app,
        window.label(),
        &BrowserLocation::File {
            path: Path::new(&project_path).to_path_buf(),
        },
    )?;
    snapshot(state.inner(), window.label())
}

#[tauri::command]
pub fn pack_unibrow_to_archive(
    window: tauri::Window,
    state: State<AppState>,
    project_path: String,
    dest_zip: String,
) -> Result<(), String> {
    let zip = crate::path_security::resolve_open_path(&dest_zip, &state.cwd)?;
    run_pack_active(state.inner(), &project_path, &zip)?;
    Ok(())
}

#[tauri::command]
pub fn set_project_folder_hint_settings(
    _window: tauri::Window,
    megabytes: u32,
) -> Result<crate::commands::PreferencesDto, String> {
    let mut p = load_preferences();
    p.project_folder_hint_bytes = if megabytes == 0 {
        0
    } else {
        (megabytes as u64) * 1024 * 1024
    };
    save_preferences(&p)?;
    Ok(crate::commands::preferences_dto())
}

pub fn active_package_storage_dto(
    state: &AppState,
    window_label: &str,
) -> Option<crate::package_storage_session::PackageStorageSessionDto> {
    let sess = crate::commands::ws_label(state, window_label).ok()?;
    let tabs = sess.tabs.lock().ok()?;
    let tab = tabs.active_tab()?;
    let path = match &tab.location {
        BrowserLocation::File { path } => path.to_string_lossy().into_owned(),
        _ => return None,
    };
    state
        .package_storage
        .get(&path)
        .map(|s| session_to_dto(&s))
}
