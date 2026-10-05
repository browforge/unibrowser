mod commands;
mod config;
mod downloads;
mod mod_portal;
mod preferences;
mod project_backend;
mod state;
mod webview;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            webview::ensure_content_webview(app.handle())?;
            if let Some(window) = app.get_window("main") {
                let saved = preferences::load_window_state();
                let _ = preferences::apply_window_state(&window, &saved);
            }
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
            commands::run_rhai_script,
            commands::get_ffmpeg_probe,
            commands::get_home_dir,
            commands::get_preferences,
            commands::dismiss_onboarding,
            commands::save_window_geometry,
            commands::create_unibrow_project,
            commands::open_mod_portal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
