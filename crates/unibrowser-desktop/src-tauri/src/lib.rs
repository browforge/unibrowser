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
pub mod path_security;
pub mod preferences;
mod project_backend;
mod smf;
mod midi_stub;
mod settings_stub;
mod about_stub;
mod keys_stub;
mod web_font_prefs;
mod crash_recovery;
mod document_limits;
mod content_sniff;
mod files_listing;
pub mod file_ops;
mod batch_images;
mod pdf_preview;
mod dir_list;
mod dir_search;
mod listing_control;
mod trash;
pub mod window_session;
mod workspace_layouts;
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
            webview::ensure_content_webview(app.handle(), "main")?;
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
                    session::persist_session_if_enabled(state.inner(), "main");
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
            commands::begin_directory_listing,
            commands::cancel_directory_listing,
            commands::drop_paths_on_tab,
            commands::toggle_recent_pin,
            commands::clear_recent_files,
            commands::search_filenames,
            commands::cancel_filename_search,
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
            commands::get_build_info,
            commands::get_preferences,
            commands::set_ntp_sections,
            commands::set_advanced_webview_devtools,
            commands::set_advanced_experimental_spellcheck,
            commands::set_files_view_prefs,
            commands::open_path_in_new_tab,
            commands::reveal_path_in_system,
            commands::toggle_folder_favorite,
            commands::peek_archive_entries,
            commands::open_archive_entry,
            commands::delete_paths,
            commands::transfer_paths,
            commands::create_folder,
            commands::create_text_file,
            commands::rename_path,
            commands::duplicate_path,
            commands::path_properties,
            commands::compress_paths_to_zip,
            commands::set_auto_save_settings,
            commands::set_download_preference,
            commands::set_board_snap_to_grid,
            commands::set_crash_recovery_enabled,
            commands::list_crash_recovery_backups,
            commands::dismiss_crash_recovery_backup,
            commands::restore_crash_recovery_backup,
            commands::list_workspace_layout_presets,
            commands::save_workspace_layout_preset,
            commands::load_workspace_layout_preset,
            commands::delete_workspace_layout_preset,
            commands::write_document_bytes,
            commands::pick_save_image_path,
            commands::resolve_preview_link,
            commands::open_new_browser_window,
            commands::add_unibrow_timeline_clip,
            commands::remove_unibrow_timeline_clip,
            commands::reorder_unibrow_timeline_clips,
            commands::update_unibrow_timeline_clip,
            commands::video_scrub_thumbnail,
            commands::export_unibrow_mp4_preview,
            commands::export_unibrow_mp4_h264,
            commands::open_url_externally,
            commands::set_open_downloads_in_system_browser,
            commands::batch_convert_images,
            commands::timeline_clip_scrub_preview,
            commands::set_unibrow_timeline_transition,
            commands::set_unibrow_node_color,
            commands::set_unibrow_timeline_clip_color,
            commands::add_unibrow_timeline_marker,
            commands::remove_unibrow_timeline_marker,
            commands::set_midi_scrub_audio_enabled,
            commands::auto_save_active_document,
            commands::pick_and_compare,
            commands::close_file_compare,
            commands::pin_current_location,
            commands::begin_file_compare,
            commands::set_ui_theme,
            commands::get_epub_chapter,
            commands::search_epub_in_book,
            commands::export_unibrow_project,
            commands::export_unibrow_share_package,
            commands::import_unibrow_share_package,
            commands::autosave_unibrow_active_project,
            commands::set_project_autosave_settings,
            commands::dismiss_onboarding,
            commands::set_accessibility_prefs,
            commands::set_command_manager_dense,
            commands::complete_onboarding_checklist_step,
            commands::set_onboarding_checklist_skip_forever,
            commands::save_window_geometry,
            commands::create_unibrow_project,
            commands::open_mod_portal,
            commands::list_installed_vsix,
            commands::install_vsix_from_path,
            commands::download_install_vsix,
            commands::pick_and_install_vsix,
            commands::set_vsix_extension_enabled,
            commands::list_installed_native_mods,
            commands::set_native_mod_enabled,
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
