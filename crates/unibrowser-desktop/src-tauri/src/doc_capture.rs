//! Headless docs screenshots (`--doc-capture`). Runs on the GTK main thread like self-test.

use std::path::PathBuf;
use std::time::Duration;

use tauri::{AppHandle, Manager, State};

use crate::launch_args;
use crate::self_test;

pub fn schedule_run(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(8));
        eprintln!("[doc-capture] scheduling capture on main thread");
        let app2 = app.clone();
        let _ = app.run_on_main_thread(move || {
            match run_on_main(&app2) {
                Ok(path) => {
                    eprintln!("[doc-capture] wrote {}", path.display());
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[doc-capture] failed: {}", e);
                    std::process::exit(2);
                }
            }
        });
    });
}

fn run_on_main(app: &AppHandle) -> Result<PathBuf, String> {
    let path = launch_args::doc_capture_output_path().ok_or("doc capture path missing")?;
    let window = app.get_window("main").ok_or("main window missing")?;
    let state = app.state::<crate::state::AppState>();

    for open in launch_args::launch_open_paths_snapshot() {
        let _ = crate::commands::open_path(
            window.clone(),
            open,
            None,
            app.clone(),
            state.clone(),
        );
    }

    if let Some(demo) = launch_args::get_demo_state() {
        if !demo.is_empty() {
            apply_demo_state(app, &window, &state, &demo)?;
            std::thread::sleep(Duration::from_millis(900));
        }
    }

    self_test::scenario_shell_chrome_batch1(app)?;
    launch_args::write_doc_capture_png(app, &path)?;
    Ok(path)
}

fn apply_demo_state(
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
            "ntp" => {
                crate::commands::new_tab(window.clone(), state.clone())?;
            }
            "palette" => {
                eval_js(
                    app,
                    "document.getElementById('palette')?.classList.remove('hidden');",
                )?;
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
                    "document.getElementById('problems-bottom-panel')?.classList.remove('hidden');",
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
            _ => {}
        }
    }
    Ok(())
}

fn eval_js(app: &AppHandle, script: &str) -> Result<(), String> {
    let wv = app.get_webview("main").ok_or("main shell webview missing")?;
    wv.eval(script).map_err(|e| e.to_string())?;
    Ok(())
}
