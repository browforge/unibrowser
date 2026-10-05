mod commands;
mod downloads;
mod state;
mod webview;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            webview::ensure_content_webview(app.handle())?;
            Ok(())
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
            commands::get_toolbar_commands,
            commands::execute_command,
            commands::palette_search,
            commands::handle_download_url,
            commands::set_content_bounds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
