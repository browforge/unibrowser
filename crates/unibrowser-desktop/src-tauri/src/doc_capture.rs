//! Headless docs screenshots (`--doc-capture`). Frontend waits for paint then calls `finish_doc_capture_and_exit`.

use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::commands;
use crate::state::AppState;

use crate::launch_args;
use crate::self_test;
use std::sync::atomic::{AtomicBool, Ordering};

static DOC_CAPTURE_OPENED: AtomicBool = AtomicBool::new(false);

/// Rust-side demo tokens when the shell webview has not finished booting (fallback only).
pub fn apply_demo_state_rust(
    app: &AppHandle,
    window: &tauri::Window,
    state: &State<crate::state::AppState>,
    raw: &str,
) -> Result<(), String> {
    for token in raw.split([',', ';']) {
        let t = token.trim().to_lowercase();
        if t.is_empty() {
            continue;
        }
        match t.as_str() {
            "document-mockup" | "document-mockup-problems" => {
                crate::document_mockup::seed_document_mockup_problems();
                eval_js(
                    app,
                    r"(function(){
  const p = document.getElementById('script-output-panel');
  if (p) p.classList.remove('hidden');
  document.getElementById('bottom-panel-tab-problems')?.click();
})();",
                )?;
            }
            "ntp" => {
                crate::commands::new_tab(window.clone(), state.clone())?;
            }
            "palette" => {
                eval_js(
                    app,
                    r"(function(){
  document.getElementById('palette')?.classList.remove('hidden');
  const q = document.getElementById('palette-query');
  if (q) { q.value = 'font'; q.dispatchEvent(new Event('input', { bubbles: true })); }
})();",
                )?;
            }
            "theme-light" => {
                eval_js(app, "document.documentElement.setAttribute('data-theme','light');")?;
            }
            "theme-dark" => {
                eval_js(app, "document.documentElement.setAttribute('data-theme','dark');")?;
            }
            "settings" => {
                crate::commands::open_path(
                    window.clone(),
                    "unibrowser://settings".into(),
                    None,
                    app.clone(),
                    state.clone(),
                )?;
            }
            "mods" | "vsix" | "vsix-badge" => {
                crate::commands::open_path(
                    window.clone(),
                    "unibrowser://mods".into(),
                    None,
                    app.clone(),
                    state.clone(),
                )?;
            }
            "problems" | "problems-open" => {
                let _ = crate::proofing::proofing_check(
                    state.clone(),
                    crate::proofing::ProofingCheckRequest {
                        text: "teh quick brown fox".into(),
                        surface: "plaintext".into(),
                        language: Some("en-NZ".into()),
                        code_language: None,
                        grammar_enabled: None,
                    },
                );
                eval_js(
                    app,
                    r"(function(){
  const p = document.getElementById('script-output-panel');
  if (p) p.classList.remove('hidden');
  document.getElementById('bottom-panel-tab-problems')?.click();
})();",
                )?;
            }
            "ai" | "ai-sidebar" | "ai-open" => {
                use crate::ai_assistant_host::{AiAssistantPrefs, AiGrantDto, set_ai_assistant_prefs};
                set_ai_assistant_prefs(AiAssistantPrefs {
                    base_url: Some("mock://unibrowser-self-test".into()),
                    model: Some("docs".into()),
                    api_key: None,
                    grants: vec![
                        AiGrantDto {
                            permission: "read".into(),
                            scope_kind: "global".into(),
                            scope_value: None,
                        },
                        AiGrantDto {
                            permission: "ask".into(),
                            scope_kind: "workspace".into(),
                            scope_value: Some("/workspace".into()),
                        },
                    ],
                    panel_visible: true,
                    panel_x: 0.0,
                    panel_y: 0.0,
                    panel_docked: true,
                    panel_width: 320.0,
                })?;
                eval_js(
                    app,
                    r"(function(){var p=document.getElementById('ai-assistant-panel');if(p){p.classList.remove('hidden');}})();",
                )?;
            }
            "live" | "live-preview" => {
                let _ = crate::live_host::set_unibrow_live_mode(window.clone(), true, state.clone());
            }
            "capture-ready" => {
                let _ = crate::demo_ntp::seed_demo_ntp_fixture();
            }
            _ => {}
        }
    }
    Ok(())
}

/// Seed fixtures before the shell webview boots (tab opens stay on the JS capture path).
pub fn prepare_doc_capture_session(_app: &AppHandle, _state: &AppState) -> Result<(), String> {
    if !launch_args::is_doc_capture_mode() {
        return Ok(());
    }
    let demo = launch_args::get_demo_state()
        .map(|d| d.to_ascii_lowercase())
        .unwrap_or_default();
    if demo.contains("capture-ready") {
        let _ = crate::demo_ntp::seed_demo_ntp_fixture();
    }
    Ok(())
}

/// Push an initial `TabContentDto` into the shell without a webview `invoke` (avoids capture bootstrap deadlocks).
pub fn schedule_shell_state_sync(app: AppHandle) {
    if !launch_args::is_doc_capture_mode() {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1200));
        for attempt in 0..120 {
            if push_shell_state_once(&app).is_ok() {
                eprintln!("[doc-capture] shell state sync ok (attempt {})", attempt + 1);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        eprintln!("[doc-capture] shell state sync gave up");
    });
}

fn push_shell_state_once(app: &AppHandle) -> Result<(), String> {
    let window = app.get_window("main").ok_or("main window missing")?;
    let state = app.state::<AppState>();
    if !DOC_CAPTURE_OPENED.swap(true, Ordering::SeqCst) {
        let demo = launch_args::get_demo_state()
            .map(|d| d.to_ascii_lowercase())
            .unwrap_or_default();
        let opened_paths = launch_args::take_launch_open_paths();
        let had_open_paths = !opened_paths.is_empty();
        for open in opened_paths {
            let resolved = launch_args::resolve_capture_path_str(&open)?;
            let _ = commands::open_path(
                window.clone(),
                resolved,
                None,
                app.clone(),
                state.clone(),
            )?;
        }
        if demo.contains("ntp") && !had_open_paths {
            let _ = commands::new_tab(window.clone(), state.clone())?;
        }
    }
    let dto = commands::get_browser_state(window, state)?;
    let json = serde_json::to_string(&dto).map_err(|e| e.to_string())?;
    let wv = app.get_webview("main").ok_or("main shell webview missing")?;
    let json_literal = serde_json::to_string(&json).map_err(|e| e.to_string())?;
    let script = format!(
        "(function(){{try{{var d=JSON.parse({0});var a=window.__selfTestApplyState||window.__unibrowserApplyState;if(!a){{return false;}}void Promise.resolve(a(d));return true;}}catch(e){{return false;}}}})();",
        json_literal
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run_capture_on_main(app: &AppHandle) -> Result<PathBuf, String> {
    let path = launch_args::doc_capture_output_path().ok_or("doc capture path missing")?;
    let window = app.get_window("main").ok_or("main window missing")?;
    let state = app.state::<crate::state::AppState>();

    for open in launch_args::launch_open_paths_snapshot() {
        let resolved = launch_args::resolve_capture_path_str(&open).unwrap_or(open);
        let _ = crate::commands::open_path(
            window.clone(),
            resolved,
            None,
            app.clone(),
            state.clone(),
        );
    }

    if let Some(demo) = launch_args::get_demo_state() {
        if !demo.is_empty() {
            apply_demo_state_rust(app, &window, &state, &demo)?;
        }
    }

    eval_js(
        app,
        r"(function(){try{var h=window.__unibrowserMeasureChromeHeights?.();if(h){console.error('[chrome-metrics] '+JSON.stringify(h));}}catch(e){}})();",
    )?;
    self_test::scenario_shell_chrome_batch3(app)?;
    launch_args::write_doc_capture_png(app, &path)?;
    Ok(path)
}

fn eval_js(app: &AppHandle, script: &str) -> Result<(), String> {
    let wv = app.get_webview("main").ok_or("main shell webview missing")?;
    wv.eval(script).map_err(|e| e.to_string())?;
    Ok(())
}
