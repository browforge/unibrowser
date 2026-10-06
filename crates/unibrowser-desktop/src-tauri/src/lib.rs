mod commands;
mod tabs_mutex;
mod self_test;
mod launch_args;
mod doc_capture;
mod document_mockup;
mod proofing;
mod toolbar_windows;
mod workspace_personas;
mod config;
pub mod downloads;
mod download_history;
mod downloads_stub;
mod csv_preview;
mod epub_preview;
mod undo_stack;
mod mod_portal;
mod open_vsx;
pub mod path_security;
pub mod preferences;
mod project_backend;
mod live_host;
mod ai_assistant_host;
mod smf;
mod midi_stub;
mod settings_stub;
mod about_stub;
mod keys_stub;
mod diagnostics_stub;
mod command_history;
mod context_help;
mod docs_stub;
mod docs_content;
mod pdf_annotations;
mod diagnostic_log;
mod capture;
mod paths;
mod web_font_prefs;
mod web_zoom_prefs;
mod web_reader_prefs;
pub mod autosave;
mod crash_recovery;
mod crash_log;
mod process_status;
mod project_recovery;
mod project_undo;
mod project_undo_handlers;
mod vsix_host;
mod editor_themes;
mod editor_grammars;
mod terminal_host;
mod web_proxy;
mod vsix_extension_config;
mod document_limits;
mod content_sniff;
mod files_listing;
pub mod file_ops;
mod batch_images;
mod pdf_export;
mod pdf_preview;
mod pdf_proving;
mod dir_list;
mod dir_search;
mod listing_control;
mod trash;
mod closed_tabs;
mod close_undo;
mod crash_launch;
mod lsp_host;
mod problems_store;
pub mod window_session;
mod workspace_layouts;
mod workspace_tree;
mod workspace_quick_search;
mod git_status;
mod rhai_host;
mod session;
mod state;
mod user_backup;

pub use state::open_file_document;
mod webview;
mod webview_capture;
mod webview_find;
mod webview_download;
#[cfg(target_os = "windows")]
mod webview_download_win;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    launch_args::ingest_process_args();
    crash_log::install_panic_hook();
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());
    #[cfg(any(debug_assertions, feature = "e2e"))]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .manage(AppState::new())
        .setup(|app| {
            let e2e = std::env::var("UNIBROWSER_E2E").as_deref() == Ok("1");
            let self_test = launch_args::is_self_test_mode();
            let doc_capture = launch_args::is_doc_capture_mode();
            if e2e || self_test || doc_capture {
                let mut p = preferences::load_preferences();
                p.onboarding_dismissed = true;
                p.onboarding_tour_skipped_forever = true;
                p.restore_session_on_launch = false;
                if self_test {
                    p.proofing_squiggles_enabled = true;
                }
                let _ = preferences::save_preferences(&p);
            }
            if !e2e {
                webview::ensure_content_webview(app.handle(), "main")?;
            }
            if let Some(window) = app.get_window("main") {
                if e2e || self_test || doc_capture {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let saved = preferences::load_window_state();
                let _ = preferences::apply_window_state(&window, &saved);
                if self_test {
                    self_test::schedule_run(app.handle().clone());
                }
            }
            if let Some(state) = app.try_state::<AppState>() {
                if !e2e && !self_test && !doc_capture {
                    commands::restore_session_on_launch(state.inner(), app.handle())?;
                }
                let _ = toolbar_windows::restore_detached_toolbars(app.handle(), state.inner());
            }
            let _ = unibrowser_media::warm_sidecar_pool();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if toolbar_windows::is_toolbar_window_label(window.label()) {
                    if let Some(state) = window.try_state::<AppState>() {
                        let _ = toolbar_windows::dock_bar_on_window_close(
                            window.app_handle(),
                            state.inner(),
                            window.label(),
                        );
                    }
                }
                if let Ok(state) = preferences::capture_window_state(window) {
                    let _ = preferences::save_window_state(&state);
                }
                if let Some(state) = window.try_state::<AppState>() {
                    session::persist_session_if_enabled(state.inner(), window.app_handle());
                }
            }
            if let tauri::WindowEvent::DragDrop(drag) = event {
                if let tauri::DragDropEvent::Drop { paths, .. } = drag {
                    if let Some(state) = window.try_state::<AppState>() {
                        let _ = commands::open_dropped_paths(
                            window.app_handle(),
                            state.inner(),
                            window.label(),
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
            commands::reorder_tabs,
            commands::toggle_tab_pin,
            commands::toggle_tab_mute,
            commands::close_tab,
            commands::undo_close_tab,
            commands::reload_active_tab_from_disk,
            commands::dismiss_recovery_bar_item,
            commands::search_open_tabs,
            commands::activate_tab_in_window,
            commands::open_crash_logs_folder,
            commands::open_releases_page,
            commands::open_report_issue_page,
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
            commands::clear_all_recent_files,
            commands::reset_onboarding_prefs_cmd,
            commands::reset_floating_toolbars,
            commands::search_filenames,
            commands::cancel_filename_search,
            commands::search_workspace_quick_files,
            commands::probe_midi_audio_render,
            commands::get_command_manager,
            commands::save_command_manager_layout,
            commands::detach_floating_toolbar_window,
            commands::dock_floating_toolbar_window,
            commands::open_active_file_as_text,
            commands::open_active_file_as_hex,
            commands::list_workspace_personas,
            commands::apply_workspace_persona,
            commands::export_command_manager_layout_file,
            commands::import_command_manager_layout_file,
            commands::select_command_manager_tab,
            commands::pin_command_manager_context,
            commands::clear_command_manager_context_pin,
            commands::execute_command,
            commands::palette_search,
            commands::get_keybindings,
            commands::get_keyboard_shortcuts,
            commands::reload_keybindings,
            commands::set_bookmark_bar_enabled,
            commands::set_bookmark_quick_links,
            commands::dispatch_key_chord,
            commands::sync_text_document,
            commands::handle_download_url,
            commands::record_download_failure,
            commands::clear_download_history,
            commands::retry_download,
            commands::set_content_bounds,
            commands::read_document_bytes,
            rhai_host::run_rhai_script,
            commands::sync_pdf_viewer_state,
            commands::sync_pdf_page,
            commands::ensure_pdf_raster_proxy,
            commands::sync_pdf_scale,
            commands::get_help_doc_markdown,
            commands::get_help_doc_toc,
            commands::set_unibrow_timeline_clip_duck,
            commands::set_unibrow_timeline_clip_fade,
            commands::dismiss_scripting_extension_tip,
            commands::list_pdf_highlights,
            commands::add_pdf_highlight,
            commands::web_page_zoom,
            commands::reset_active_page_zoom,
            commands::reset_active_context_font,
            commands::pick_open_file,
            commands::pick_browse_folder,
            commands::add_unibrow_asset,
            commands::remove_unibrow_node,
            commands::remove_unibrow_orphan_asset,
            commands::rename_unibrow_node,
            commands::rename_unibrow_timeline_clip,
            commands::set_unibrow_node_position,
            commands::pick_unibrow_asset_file,
            commands::allow_asset_path,
            commands::allow_unibrow_asset,
            commands::get_ffmpeg_probe,
            live_host::set_unibrow_live_mode,
            live_host::unibrow_live_status,
            live_host::start_unibrow_live_recording,
            live_host::stop_unibrow_live_recording,
            ai_assistant_host::get_ai_assistant_prefs,
            ai_assistant_host::set_ai_assistant_prefs,
            ai_assistant_host::test_ai_connection,
            ai_assistant_host::ai_assistant_invoke,
            ai_assistant_host::ai_assistant_status,
            ai_assistant_host::document_projection_snapshot,
            ai_assistant_host::ai_assistant_tools,
            ai_assistant_host::ai_assistant_read_tab_text,
            ai_assistant_host::ai_assistant_propose_write,
            ai_assistant_host::ai_assistant_approve_pending,
            ai_assistant_host::ai_assistant_chat,
            commands::get_home_dir,
            commands::get_downloads_dir,
            commands::get_build_info,
            commands::get_preferences,
            commands::set_ntp_sections,
            commands::set_advanced_webview_devtools,
            commands::set_advanced_experimental_spellcheck,
            proofing::proofing_check,
            proofing::proofing_dictionary_add,
            proofing::proofing_dictionary_remove,
            proofing::proofing_dictionary_list,
            commands::set_proofing_squiggles_enabled,
            commands::set_proofing_grammar_enabled,
            commands::set_advanced_node_extension_host,
            commands::set_disable_user_mods,
            commands::set_files_view_prefs,
            commands::open_path_in_new_tab,
            commands::open_selected_paths,
            commands::web_find_in_page,
            commands::focus_content_dom_selector,
            commands::reveal_path_in_system,
            commands::toggle_folder_favorite,
            commands::set_workspace_root,
            commands::clear_workspace_root,
            commands::list_workspace_tree,
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
            commands::list_project_recovery_backups,
            commands::dismiss_project_recovery_backup,
            commands::restore_project_recovery_backup,
            commands::list_workspace_layout_presets,
            commands::save_workspace_layout_preset,
            commands::load_workspace_layout_preset,
            commands::delete_workspace_layout_preset,
            commands::write_document_bytes,
            commands::pick_save_image_path,
            commands::pick_save_html_path,
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
            commands::set_web_proxy_prefs,
            commands::set_warn_insecure_http,
            commands::batch_convert_images,
            commands::timeline_clip_scrub_preview,
            commands::timeline_clip_block_preview,
            commands::add_unibrow_pdf_page_timeline_clip,
            commands::add_unibrow_svg_still_timeline_clip,
            commands::add_unibrow_title_timeline_clip,
            commands::add_unibrow_midi_timeline_clip,
            commands::set_unibrow_timeline_clip_pdf_page,
            commands::set_unibrow_timeline_clip_title_text,
            commands::set_unibrow_timeline_transition,
            commands::set_unibrow_node_color,
            commands::set_unibrow_timeline_clip_color,
            commands::add_unibrow_timeline_marker,
            commands::remove_unibrow_timeline_marker,
            commands::add_unibrow_clip_marker,
            commands::remove_unibrow_clip_marker,
            commands::set_midi_scrub_audio_enabled,
            commands::set_fluidsynth_soundfont_path,
            commands::auto_save_active_document,
            commands::get_autosave_status,
            commands::list_autosave_slots_for_active_tab,
            commands::restore_latest_autosave_for_active_tab,
            commands::restore_autosave_from_slot,
            proofing::list_problems_panel,
            proofing::proofing_check_unibrow_project,
            commands::pick_and_compare,
            commands::close_file_compare,
            commands::pick_and_compare_images,
            commands::close_image_compare,
            commands::begin_image_compare,
            commands::capture_visible_tab,
            commands::save_tab_capture_png,
            commands::record_diagnostic_error,
            commands::log_shell_ui_event,
            commands::get_diagnostics_info,
            commands::pin_current_location,
            commands::begin_file_compare,
            commands::set_ui_theme,
            commands::list_editor_themes_cmd,
            commands::list_monaco_grammars_cmd,
            commands::save_docx_from_preview_html,
            commands::ocr_tesseract_available_cmd,
            commands::terminal_spawn,
            commands::terminal_write,
            commands::terminal_resize,
            commands::get_editor_monaco_theme,
            commands::set_editor_theme,
            commands::get_epub_chapter,
            commands::list_epub_chapters,
            commands::get_epub_read_position,
            commands::remember_epub_chapter_position,
            commands::get_epub_reading_notes,
            commands::set_epub_reading_notes,
            commands::export_epub_reading_notes,
            commands::search_epub_in_book,
            commands::export_grandma_backup_file,
            commands::import_grandma_backup_file,
            commands::export_unibrow_project,
            commands::export_unibrow_share_package,
            commands::import_unibrow_share_package,
            commands::autosave_unibrow_active_project,
            commands::set_project_autosave_settings,
            commands::dismiss_onboarding,
            commands::skip_onboarding_tour_forever,
            commands::set_ui_locale,
            commands::set_accessibility_prefs,
            commands::set_command_manager_dense,
            commands::set_chrome_focus_mode,
            commands::set_larger_hit_targets,
            commands::complete_onboarding_checklist_step,
            commands::set_onboarding_checklist_skip_forever,
            commands::save_window_geometry,
            commands::create_unibrow_project,
            commands::list_mod_catalog,
            commands::paste_unibrow_still_clip,
            commands::write_temp_png,
            commands::dismiss_cm_pin_tip,
            commands::open_mod_portal,
            commands::list_installed_vsix,
            commands::install_vsix_from_path,
            commands::download_install_vsix,
            commands::pick_and_install_vsix,
            commands::set_vsix_extension_enabled,
            commands::trust_vsix_extension,
            commands::set_offline_mode,
            commands::list_installed_native_mods,
            commands::set_native_mod_enabled,
            commands::uninstall_vsix_extension,
            commands::sync_svg_document,
            commands::document_undo,
            commands::document_redo,
            commands::export_print_pdf,
            commands::toggle_split_view,
            commands::set_split_tab,
            commands::omnibox_completions,
            commands::audio_waveform_peaks,
            commands::get_csv_preview,
            commands::search_open_vsx_catalog,
            rhai_host::describe_rhai_script,
            launch_args::take_launch_open_paths,
            launch_args::get_demo_state,
            launch_args::take_doc_capture_path,
            launch_args::abort_doc_capture,
            launch_args::finish_doc_capture_and_exit,
            document_mockup::seed_document_mockup_problems_cmd,
            self_test::self_test_submit_shell_chrome_audit,
            self_test::self_test_ping,
            self_test::get_self_test_plan,
            self_test::self_test_capture_screenshot,
            self_test::self_test_finish,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
