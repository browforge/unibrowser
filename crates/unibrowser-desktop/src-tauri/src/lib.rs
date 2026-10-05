mod commands;
mod config;
pub mod downloads;
mod csv_preview;
mod csv_stub;
mod epub_preview;
mod epub_stub;
mod undo_stack;
mod mod_portal;
mod open_vsx;
mod path_security;
pub mod preferences;
mod project_backend;
mod smf;
mod midi_stub;
mod settings_stub;
mod about_stub;
mod web_font_prefs;
mod crash_recovery;
mod rhai_host;
mod session;
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
            if let Some(state) = app.try_state::<AppState>() {
                commands::restore_session_on_launch(state.inner(), app.handle())?;
            }
            let _ = unibrowser_media::warm_sidecar_pool();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Ok(state) = preferences::capture_window_state(window) {
                    let _ = preferences::save_window_state(&state);
                }
                if let Some(state) = window.try_state::<AppState>() {
                    session::persist_session_if_enabled(state.inner());
                }
            }
            if let tauri::WindowEvent::DragDrop(drag) = event {
                if let tauri::DragDropEvent::Drop { paths, .. } = drag {
                    if let Some(state) = window.try_state::<AppState>() {
                        let _ = commands::open_dropped_paths(
                            window.app_handle(),
                            state.inner(),
                            paths.clone(),
                        );
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_browser_state,
            commands::new_tab,
            commands::select_tab,
            commands::close_tab,
            commands::search_open_tabs,
            commands::set_restore_session_on_launch,
            commands::navigate_omnibox,
            commands::open_path,
            commands::go_back,
            commands::go_forward,
            commands::reload_tab,
            commands::list_directory,
            commands::get_command_manager,
            commands::save_command_manager_layout,
            commands::export_command_manager_layout_file,
            commands::import_command_manager_layout_file,
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
            commands::sync_pdf_scale,
            commands::web_page_zoom,
            commands::pick_open_file,
            commands::pick_browse_folder,
            commands::add_unibrow_asset,
            commands::remove_unibrow_node,
            commands::rename_unibrow_node,
            commands::set_unibrow_node_position,
            commands::pick_unibrow_asset_file,
            commands::allow_asset_path,
            commands::allow_unibrow_asset,
            commands::get_ffmpeg_probe,
            commands::get_home_dir,
            commands::get_preferences,
            commands::set_auto_save_settings,
            commands::set_download_preference,
            commands::set_board_snap_to_grid,
            commands::set_crash_recovery_enabled,
            commands::add_unibrow_timeline_clip,
            commands::remove_unibrow_timeline_clip,
            commands::reorder_unibrow_timeline_clips,
            commands::auto_save_active_document,
            commands::pick_and_compare,
            commands::close_file_compare,
            commands::pin_current_location,
            commands::begin_file_compare,
            commands::set_ui_theme,
            commands::get_epub_chapter,
            commands::export_unibrow_project,
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
            commands::document_undo,
            commands::document_redo,
            commands::toggle_split_view,
            commands::set_split_tab,
            commands::omnibox_completions,
            commands::get_csv_preview,
            commands::search_open_vsx_catalog,
            rhai_host::describe_rhai_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
