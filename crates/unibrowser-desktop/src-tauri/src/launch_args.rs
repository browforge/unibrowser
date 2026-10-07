use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use tauri::Manager;

use crate::self_test::{configure as configure_self_test, ALL_SCENARIOS};

fn launch_open_paths() -> &'static Mutex<Vec<String>> {
    static SLOT: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(Vec::new()))
}

fn demo_state_slot() -> &'static Mutex<Option<String>> {
    static SLOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn doc_capture_slot() -> &'static Mutex<Option<PathBuf>> {
    static SLOT: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Parse argv (after program name). Supports `--open <path>`, `--self-test`, `--report`, `--help`, and positional file paths.
pub fn ingest_process_args() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut open_paths: Vec<String> = Vec::new();
    let mut self_test_scenario: Option<String> = None;
    let mut report_path: Option<PathBuf> = None;
    let mut doc_capture_path: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--help" || a == "-h" {
            print_help();
            std::process::exit(0);
        }
        if a == "--open" {
            i += 1;
            if i < args.len() {
                open_paths.push(args[i].clone());
            }
            i += 1;
            continue;
        }
        if a == "--self-test" {
            i += 1;
            if i < args.len() {
                self_test_scenario = Some(args[i].clone());
            }
            i += 1;
            continue;
        }
        if a == "--report" {
            i += 1;
            if i < args.len() {
                report_path = Some(PathBuf::from(&args[i]));
            }
            i += 1;
            continue;
        }
        if a == "--doc-capture" {
            i += 1;
            if i < args.len() {
                doc_capture_path = Some(PathBuf::from(&args[i]));
            }
            i += 1;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        open_paths.push(a.clone());
        i += 1;
    }
    if let Ok(mut slot) = launch_open_paths().lock() {
        *slot = open_paths;
    }
    if let Ok(mut demo) = demo_state_slot().lock() {
        *demo = std::env::var("UNIBROWSER_DEMO_STATE")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    if let Some(raw) = self_test_scenario {
        let scenarios = expand_self_test_scenarios(&raw);
        let report = report_path.unwrap_or_else(|| PathBuf::from("self-test-report.json"));
        configure_self_test(scenarios, report);
        std::env::set_var("UNIBROWSER_SELF_TEST", "1");
    }
    if let Some(path) = doc_capture_path {
        if let Ok(mut slot) = doc_capture_slot().lock() {
            *slot = Some(path);
        }
    }
}

fn expand_self_test_scenarios(raw: &str) -> Vec<String> {
    let key = raw.trim().to_lowercase();
    if key == "all" {
        return ALL_SCENARIOS.iter().map(|s| (*s).to_string()).collect();
    }
    vec![raw.trim().to_string()]
}

pub fn is_self_test_mode() -> bool {
    crate::self_test::is_active()
}

fn print_help() {
    eprintln!(
        "Unibrowser desktop\n\
\n\
Usage:\n\
  unibrowser-desktop [--open <path>] [<path> ...]\n\
  unibrowser-desktop --self-test <scenario|all> --report <out.json>\n\
\n\
  --open <path>   Open a file or folder in a new tab (repeatable).\n\
  <path>          Same as --open (for shell \"Open with\" and drag-free launch).\n\
  --self-test     Run in-app UI scenarios (no WebDriver); writes JSON report and exits.\n\
  --report        JSON report path (screenshots beside it in self-test-shots/).\n\
  --doc-capture   After launch + demo state, write composited X11 window PNG and exit.\n\
\n\
Hidden (docs / screenshots):\n\
  UNIBROWSER_DEMO_STATE   Comma-separated UI preset (problems,ai,live,vsix,ntp,palette,settings,mods).\n\
  UNIBROWSER_REPO_ROOT    Repo root for self-test fixture paths (default: cwd).\n"
    );
}

pub fn launch_open_paths_snapshot() -> Vec<String> {
    launch_open_paths()
        .lock()
        .map(|p| p.clone())
        .unwrap_or_default()
}

pub fn doc_capture_output_path() -> Option<PathBuf> {
    doc_capture_slot()
        .lock()
        .ok()
        .and_then(|d| d.as_ref().cloned())
}

#[tauri::command]
pub fn take_launch_open_paths() -> Vec<String> {
    launch_open_paths()
        .lock()
        .map(|mut p| {
            let out = p.clone();
            p.clear();
            out
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_demo_state() -> Option<String> {
    demo_state_slot().lock().ok().and_then(|d| d.clone())
}

#[tauri::command]
pub fn take_doc_capture_path() -> Option<String> {
    doc_capture_slot()
        .lock()
        .ok()
        .and_then(|d| d.as_ref().map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn abort_doc_capture(reason: String) -> Result<(), String> {
    eprintln!("[doc-capture] aborted: {}", reason);
    std::process::exit(2);
}

pub fn is_doc_capture_mode() -> bool {
    doc_capture_slot()
        .lock()
        .ok()
        .and_then(|d| d.as_ref().cloned())
        .is_some()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCaptureRegion {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    viewport_width: f64,
    viewport_height: f64,
    theme: String,
}

fn doc_capture_window_label(app: &tauri::AppHandle) -> Result<String, String> {
    let demo = get_demo_state()
        .map(|d| d.to_lowercase())
        .unwrap_or_default();
    if demo.contains("cmd-proving-detached") {
        for label in app.webview_windows().keys() {
            if label.starts_with("toolbar-") {
                return Ok(label.clone());
            }
        }
    }
    Ok("main".into())
}

fn write_doc_capture_png_inner(
    app: &tauri::AppHandle,
    path: &std::path::Path,
    editor_region: Option<&EditorCaptureRegion>,
) -> Result<(), String> {
    let window_label = doc_capture_window_label(app)?;
    let bytes = if window_label.starts_with("toolbar-") {
        crate::webview_capture::capture_shell_webview_png_for_label(app, &window_label)?
    } else {
        let window = app.get_window("main").ok_or("main window missing")?;
        let content_label = crate::webview::content_label_for_capture(window.label());
        eprintln!(
            "[doc-capture] native content webview present={}",
            app.get_webview(&content_label).is_some()
        );
        crate::webview_capture::capture_doc_screenshot_png(app, &window)?
    };
    if let Some(region) = editor_region {
        match crate::webview_capture::validate_editor_text_pixels(
            &bytes,
            (region.x, region.y, region.width, region.height),
            (region.viewport_width, region.viewport_height),
            region.theme.eq_ignore_ascii_case("dark"),
        ) {
            Ok(stats) => {
                eprintln!(
                    "[doc-capture] editor pixels ink={} rows={} background={}/{}",
                    stats.ink_pixels,
                    stats.rows_with_ink,
                    stats.background_pixels,
                    stats.sampled_pixels
                );
            }
            Err(err) => {
                eprintln!("[doc-capture] editor pixel gate warning: {}", err);
            }
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn write_doc_capture_png(app: &tauri::AppHandle, path: &std::path::Path) -> Result<(), String> {
    write_doc_capture_png_inner(app, path, None)
}

#[tauri::command]
pub fn finish_doc_capture_and_exit(
    app: tauri::AppHandle,
    editor_region: Option<EditorCaptureRegion>,
) -> Result<(), String> {
    let path = doc_capture_slot()
        .lock()
        .ok()
        .and_then(|d| d.clone())
        .ok_or("doc capture path not configured")?;
    eprintln!("[doc-capture] finish requested -> {}", path.display());
    // IPC may run on the GTK main thread; never block it waiting for capture (deadlock). Defer like self-test.
    let app2 = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        let app_main = app2.clone();
        let _ = app2.run_on_main_thread(move || {
            match write_doc_capture_png_inner(&app_main, &path, editor_region.as_ref()) {
                Ok(()) => {
                    eprintln!("[doc-capture] wrote {}", path.display());
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[doc-capture] failed: {}", e);
                    std::process::exit(3);
                }
            }
        });
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_all_lists_every_scenario() {
        let v = expand_self_test_scenarios("all");
        assert_eq!(v.len(), ALL_SCENARIOS.len());
    }
}
