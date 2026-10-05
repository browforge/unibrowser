mod commands;
mod config;
mod downloads;
mod epub_stub;
mod mod_portal;
mod open_vsx;
mod preferences;
mod project_backend;
mod rhai_host;
mod state;

pub use state::open_file_document;
mod webview;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .setup(|app| {
            webview::ensure_content_webview(app.handle())?;
            if let Some(window) = app.get_window("main") {
                let saved = preferences::load_window_state();
                let _ = preferences::apply_window_state(&window, &saved);
            }
            let _ = unibrowser_media::warm_sidecar_pool();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Ok(state) = preferences::capture_window_state(window) {
                    let _ = preferences::save_window_state(&state);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_browser_state,
            commands::new_tab,
            commands::select_tab,
            commands::navigate_omnibox,
            commands::open_path,
            commands::go_back,
            commands::go_forward,
            commands::reload_tab,
            commands::list_directory,
            commands::get_command_manager,
            commands::save_command_manager_layout,
            commands::select_command_manager_tab,
            commands::pin_command_manager_context,
            commands::clear_command_manager_context_pin,
            commands::execute_command,
            commands::palette_search,
            commands::get_keybindings,
            commands::dispatch_key_chord,
            commands::sync_text_document,
            commands::handle_download_url,
            commands::set_content_bounds,
            commands::read_document_bytes,
            rhai_host::run_rhai_script,
            commands::sync_pdf_viewer_state,
            commands::sync_pdf_page,
            commands::add_unibrow_asset,
            commands::remove_unibrow_node,
            commands::rename_unibrow_node,
            commands::pick_unibrow_asset_file,
            commands::allow_asset_path,
            commands::get_ffmpeg_probe,
            commands::get_home_dir,
            commands::get_preferences,
            commands::dismiss_onboarding,
            commands::save_window_geometry,
            commands::create_unibrow_project,
            commands::open_mod_portal,
            commands::list_installed_vsix,
            commands::install_vsix_from_path,
            commands::download_install_vsix,
            commands::pick_and_install_vsix,
            commands::set_vsix_extension_enabled,
            commands::uninstall_vsix_extension,
            commands::sync_svg_document,
            commands::search_open_vsx_catalog,
            rhai_host::describe_rhai_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
