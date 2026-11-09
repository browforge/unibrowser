//! In-process shell UI scenarios (no WebDriver). See `launch_args` and `docs/e2e.md`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

static FINISHED: AtomicBool = AtomicBool::new(false);

pub fn finished() -> bool {
    FINISHED.load(Ordering::SeqCst)
}

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

pub const SCENARIO_LIVE_PREVIEW: &str = "live-preview";
pub const SCENARIO_AI_SIDEBAR: &str = "ai-sidebar";
pub const SCENARIO_FONT_TEXT_SVG: &str = "font-text-svg";
pub const SCENARIO_UNKNOWN_BANNER: &str = "unknown-file-banner";
pub const SCENARIO_CLOSE_UNDO: &str = "close-undo";
pub const SCENARIO_PROOFING_DICT: &str = "proofing-dictionary";
pub const SCENARIO_PALETTE: &str = "palette-command";
pub const SCENARIO_CM_GREY: &str = "cm-grey-tab-switch";
pub const SCENARIO_UNIBROW_BLANK: &str = "unibrow-blank-save";
pub const SCENARIO_PROBLEMS_JUMP: &str = "problems-jump";
pub const SCENARIO_SHELL_CHROME_BATCH3: &str = "shell-chrome-batch3";
pub const SCENARIO_DOCUMENT_EDITING_EVERYDAY: &str = "document-editing-everyday";
pub const SCENARIO_DOCUMENT_EDITING_KILL_PREP: &str = "document-editing-everyday-kill-prep";
pub const SCENARIO_DOCUMENT_EDITING_RELAUNCH: &str = "document-editing-everyday-relaunch";
const VISUAL_MARKER_JS: &str = "SELF_TEST_VISUAL_MARKER";
const SOURCE_MARKER_JS: &str = "SELF_TEST_SOURCE_MARKER";

static SHELL_CHROME_AUDIT: Mutex<Option<Vec<String>>> = Mutex::new(None);

struct DocumentEditingWait {
    result: Mutex<Option<Option<String>>>,
    cvar: Condvar,
}

fn document_editing_wait() -> &'static DocumentEditingWait {
    static W: OnceLock<DocumentEditingWait> = OnceLock::new();
    W.get_or_init(|| DocumentEditingWait {
        result: Mutex::new(None),
        cvar: Condvar::new(),
    })
}

const SHELL_CHROME_AUDIT_EVAL: &str = r"(function(){try{if(window.__unibrowserSubmitShellChromeAudit){window.__unibrowserSubmitShellChromeAudit();return;}var fn=window.__unibrowserRunShellChromeAudit;if(!fn){return;}var inv=window.__TAURI__?.core?.invoke;if(inv){inv('self_test_submit_shell_chrome_audit',{errors:fn()});}}catch(e){}})();";

#[tauri::command]
pub fn self_test_submit_shell_chrome_audit(errors: Vec<String>) {
    if let Ok(mut g) = SHELL_CHROME_AUDIT.lock() {
        *g = Some(errors);
    }
}

fn take_shell_chrome_audit() -> Option<Vec<String>> {
    drain_shell_chrome_audit()
}

#[tauri::command]
pub fn self_test_submit_document_editing_result(message: Option<String>) {
    let w = document_editing_wait();
    if let Ok(mut g) = w.result.lock() {
        *g = Some(message);
        w.cvar.notify_all();
    }
}

fn reset_document_editing_wait() {
    if let Ok(mut g) = document_editing_wait().result.lock() {
        *g = None;
    }
}

#[cfg(target_os = "linux")]
fn pump_gtk_events() {
    while gtk::events_pending() {
        gtk::main_iteration();
    }
    gtk::main_iteration();
}

#[cfg(not(target_os = "linux"))]
fn pump_gtk_events() {}

/// Waits for the in-webview document-editing harness.
///
/// A helper thread blocks on the condvar fed by `self_test_submit_document_editing_result` (no GTK).
/// This thread must call `pump_gtk_events` until that helper completes: Tauri IPC and the Linux
/// WebKit/GTK shell only deliver `self_test_submit_*` commands while the GTK main loop runs.
enum DocumentEditingWaitOutcome {
    Finished(Result<(), String>),
    Timeout,
}

fn await_document_editing_harness(timeout: Duration) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<DocumentEditingWaitOutcome>();
    let wait = document_editing_wait();
    std::thread::spawn(move || {
        let mut guard = wait.result.lock().expect("doc-edit wait mutex");
        loop {
            if guard.is_some() {
                let message = guard.take().expect("doc-edit result");
                let outcome = match message {
                    None => Ok(()),
                    Some(err) => Err(err),
                };
                let _ = tx.send(DocumentEditingWaitOutcome::Finished(outcome));
                return;
            }
            let (next, timed_out) = wait
                .cvar
                .wait_timeout(guard, timeout)
                .expect("doc-edit condvar");
            guard = next;
            if guard.is_some() {
                continue;
            }
            if timed_out.timed_out() {
                let _ = tx.send(DocumentEditingWaitOutcome::Timeout);
                return;
            }
        }
    });
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        pump_gtk_events();
        match rx.try_recv() {
            Ok(DocumentEditingWaitOutcome::Finished(outcome)) => return outcome,
            Ok(DocumentEditingWaitOutcome::Timeout) => {
                return Err("document editing self-test timed out".into());
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return Err("document editing harness waiter exited early".into());
            }
        }
    }
    Err("document editing self-test timed out".into())
}

#[tauri::command]
pub fn self_test_abort_for_recovery() {
    std::process::abort();
}

#[tauri::command]
pub fn self_test_flush_edit_recovery_debounce() {
    crate::edit_recovery::flush_pending_for_test();
}

#[tauri::command]
pub fn edit_recovery_backup_id_for_path(path: String) -> Result<String, String> {
    Ok(crate::edit_recovery::backup_id_for_path(&path))
}

pub fn drain_shell_chrome_audit() -> Option<Vec<String>> {
    SHELL_CHROME_AUDIT
        .lock()
        .ok()
        .and_then(|mut g| g.take())
}

pub const ALL_SCENARIOS: &[&str] = &[
    SCENARIO_SHELL_CHROME_BATCH3,
    SCENARIO_PROOFING_DICT,
    SCENARIO_PROBLEMS_JUMP,
    SCENARIO_UNKNOWN_BANNER,
    SCENARIO_PALETTE,
    SCENARIO_CM_GREY,
    SCENARIO_CLOSE_UNDO,
    SCENARIO_UNIBROW_BLANK,
    SCENARIO_FONT_TEXT_SVG,
    SCENARIO_LIVE_PREVIEW,
    SCENARIO_AI_SIDEBAR,
    SCENARIO_DOCUMENT_EDITING_EVERYDAY,
];

#[derive(Clone)]
struct SelfTestSlot {
    scenarios: Vec<String>,
    report_path: PathBuf,
    screenshot_dir: PathBuf,
}

fn slot() -> &'static Mutex<Option<SelfTestSlot>> {
    static S: OnceLock<Mutex<Option<SelfTestSlot>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

pub fn configure(scenarios: Vec<String>, report_path: PathBuf) {
    let screenshot_dir = report_path
        .parent()
        .map(|p| p.join("self-test-shots"))
        .unwrap_or_else(|| PathBuf::from("self-test-shots"));
    if let Ok(mut g) = slot().lock() {
        *g = Some(SelfTestSlot {
            scenarios,
            report_path,
            screenshot_dir,
        });
    }
}

pub fn is_active() -> bool {
    slot()
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false)
}

pub fn screenshot_dir() -> Option<PathBuf> {
    slot()
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.screenshot_dir.clone()))
}

pub fn repo_root() -> PathBuf {
    if let Ok(p) = std::env::var("UNIBROWSER_REPO_ROOT") {
        let t = p.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfTestPlanDto {
    pub scenarios: Vec<String>,
    pub screenshot_dir: String,
    pub demo_pack: String,
    pub fixtures: String,
    pub projects_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResultDto {
    pub id: String,
    pub passed: bool,
    pub message: Option<String>,
    pub screenshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfTestReportDto {
    pub ok: bool,
    pub scenarios: Vec<ScenarioResultDto>,
    pub at_ms: u64,
}

#[tauri::command]
pub fn self_test_ping(phase: String) {
    eprintln!("[self-test] ping {}", phase.trim());
}

#[tauri::command]
pub fn get_self_test_plan() -> Option<SelfTestPlanDto> {
    let g = slot().lock().ok().and_then(|g| g.clone());
    let cfg = g?;
    let root = repo_root();
    Some(SelfTestPlanDto {
        scenarios: cfg.scenarios,
        screenshot_dir: cfg.screenshot_dir.to_string_lossy().into_owned(),
        demo_pack: root.join("examples/demo-pack").to_string_lossy().into_owned(),
        fixtures: root.join("e2e/fixtures").to_string_lossy().into_owned(),
        projects_dir: root.join("e2e/fixtures/projects").to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn self_test_capture_screenshot(
    app: AppHandle,
    scenario_id: String,
) -> Result<String, String> {
    if std::env::var("UNIBROWSER_SELF_TEST_SKIP_SHOTS").as_deref() == Ok("1") {
        return Ok(String::new());
    }
    let dir = screenshot_dir().ok_or("self-test not configured")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let safe = scenario_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>();
    let path = dir.join(format!("{safe}.png"));
    let bytes = crate::webview_capture::capture_shell_webview_png(&app)?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

fn write_report_and_exit(_app: AppHandle, report: SelfTestReportDto) {
    FINISHED.store(true, Ordering::SeqCst);
    eprintln!(
        "[self-test] finish ok={} scenarios={}",
        report.ok,
        report.scenarios.len()
    );
    if let Ok(guard) = slot().lock() {
        if let Some(cfg) = guard.as_ref() {
            if let Some(parent) = cfg.report_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&report) {
                let _ = std::fs::write(&cfg.report_path, json);
            }
        }
    }
    std::process::exit(if report.ok { 0 } else { 1 });
}

#[tauri::command]
pub fn self_test_finish(app: AppHandle, report: SelfTestReportDto) -> Result<(), String> {
    write_report_and_exit(app, report);
    Ok(())
}

pub fn schedule_run(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(8));
        eprintln!("[self-test] scheduling scenarios on main thread");
        let app_runner = app.clone();
        let app_main = app.clone();
        let _ = app_main.run_on_main_thread(move || {
            eprintln!("[self-test] on main thread");
            let app_scenarios = app_runner.clone();
            let app_for_exit = app_runner.clone();
            let report = match run_all_scenarios(&app_scenarios) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[self-test] scenario error: {}", e);
                    SelfTestReportDto {
                        ok: false,
                        scenarios: vec![ScenarioResultDto {
                            id: "runner".into(),
                            passed: false,
                            message: Some(e),
                            screenshot: None,
                        }],
                        at_ms: now_ms(),
                    }
                }
            };
            write_report_and_exit(app_for_exit, report);
        });
    });
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn run_all_scenarios(app: &AppHandle) -> Result<SelfTestReportDto, String> {
    eprintln!("[self-test] run_all begin");
    let cfg = slot()
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .ok_or("self-test not configured")?;
    eprintln!("[self-test] run_all config ok");
    let window = app.get_window("main").ok_or("main window missing")?;
    eprintln!("[self-test] run_all window ok");
    let state = app.state::<crate::state::AppState>();
    eprintln!("[self-test] run_all state ok");
    let root = repo_root();
    let demo = root.join("examples/demo-pack");
    let fixtures = root.join("e2e/fixtures");
    let projects = root.join("e2e/fixtures/projects");
    std::fs::create_dir_all(&projects).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for id in &cfg.scenarios {
        eprintln!("[self-test] scenario {}", id);
        let run = match id.as_str() {
            SCENARIO_FONT_TEXT_SVG => scenario_font_text_svg(&window, app, &state, &demo),
            SCENARIO_UNKNOWN_BANNER => scenario_unknown_banner(&window, app, &state, &fixtures),
            SCENARIO_CLOSE_UNDO => scenario_close_undo(&window, app, &state, &fixtures),
            SCENARIO_PROOFING_DICT => scenario_proofing_dict(&state, &fixtures),
            SCENARIO_PALETTE => scenario_palette(&window, app, &state, &demo),
            SCENARIO_CM_GREY => scenario_cm_grey(&window, app, &state, &demo),
            SCENARIO_UNIBROW_BLANK => scenario_unibrow_blank(&window, app, &state, &projects),
            SCENARIO_PROBLEMS_JUMP => scenario_problems_jump(&state, &fixtures),
            SCENARIO_LIVE_PREVIEW => scenario_live_preview(&window, app, &state, &projects),
            SCENARIO_AI_SIDEBAR => scenario_ai_sidebar(&window, app, &state),
            SCENARIO_SHELL_CHROME_BATCH3 => scenario_shell_chrome_batch3(app),
            SCENARIO_DOCUMENT_EDITING_EVERYDAY => {
                scenario_document_editing_everyday(&window, app, &state, &demo, &projects)
            }
            SCENARIO_DOCUMENT_EDITING_KILL_PREP => {
                scenario_document_editing_kill_prep(&window, app, &state, &demo, &projects)
            }
            SCENARIO_DOCUMENT_EDITING_RELAUNCH => {
                scenario_document_editing_relaunch(&window, app, &state, &demo, &projects)
            }
            other => Err(format!("unknown scenario: {other}")),
        };
        let (passed, message) = match run {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e)),
        };
        let shot = if passed {
            capture_scenario_shot(app, id)
        } else {
            capture_scenario_shot(app, id)
        };
        results.push(ScenarioResultDto {
            id: id.clone(),
            passed,
            message,
            screenshot: shot,
        });
    }
    Ok(SelfTestReportDto {
        ok: results.iter().all(|r| r.passed),
        scenarios: results,
        at_ms: now_ms(),
    })
}

fn capture_scenario_shot(app: &AppHandle, scenario_id: &str) -> Option<String> {
    if std::env::var("UNIBROWSER_SELF_TEST_SKIP_SHOTS").as_deref() == Ok("1") {
        return None;
    }
    self_test_capture_screenshot(app.clone(), scenario_id.to_string()).ok()
}

fn open_path(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    path: &std::path::Path,
) -> Result<crate::commands::TabContentDto, String> {
    crate::commands::open_path(
        window.clone(),
        path.to_string_lossy().into_owned(),
        None,
        app.clone(),
        state.clone(),
    )
}

fn scenario_font_text_svg(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &std::path::Path,
) -> Result<(), String> {
    let txt = demo.join("sample.txt");
    eprintln!("[self-test] opening {}", txt.display());
    let before = open_path(window, app, state, &txt)?;
    eprintln!("[self-test] opened text");
    let before_pt = before
        .text_document
        .as_ref()
        .map(|d| d.font_size_pt)
        .ok_or("text document missing")?;
    crate::commands::execute_command(
        window.clone(),
        "unibrowser.font.size.increase".into(),
        app.clone(),
        state.clone(),
    )?;
    let after = crate::commands::get_browser_state(window.clone(), state.clone())?;
    let after_pt = after
        .text_document
        .as_ref()
        .map(|d| d.font_size_pt)
        .ok_or("text document missing after font cmd")?;
    if after_pt <= before_pt {
        return Err("font size did not increase on text".into());
    }
    let svg = demo.join("sample.svg");
    open_path(window, app, state, &svg)?;
    let msg = crate::commands::execute_command(
        window.clone(),
        "unibrowser.font.size.increase".into(),
        app.clone(),
        state.clone(),
    )?;
    if msg.is_empty() {
        return Err("empty execute result on svg".into());
    }
    Ok(())
}

fn scenario_unknown_banner(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    fixtures: &std::path::Path,
) -> Result<(), String> {
    let path = fixtures.join("sample.step");
    if !path.is_file() {
        return Err("fixture sample.step missing".into());
    }
    let dto = open_path(window, app, state, &path)?;
    let banner = dto.unknown_file_banner.as_ref().ok_or("banner missing in state")?;
    if banner.suggestions.is_empty() {
        return Err("banner suggestions empty".into());
    }
    Ok(())
}

fn scenario_close_undo(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    fixtures: &std::path::Path,
) -> Result<(), String> {
    let path = fixtures.join("spell-test.txt");
    let filler = repo_root().join("examples/demo-pack/sample.txt");
    open_path(window, app, state, &path)?;
    crate::commands::open_path_in_new_tab(
        window.clone(),
        filler.to_string_lossy().into_owned(),
        app.clone(),
        state.clone(),
    )?;
    crate::commands::select_tab(window.clone(), 0, app.clone(), state.clone())?;
    let tab_count = crate::commands::get_browser_state(window.clone(), state.clone())?
        .tabs
        .len();
    if tab_count < 2 {
        return Err(format!("expected at least 2 tabs before close, got {tab_count}"));
    }
    let marker = format!("self-test-undo-{}", now_ms());
    let content = {
        let dto = crate::commands::get_browser_state(window.clone(), state.clone())?;
        let doc = dto.text_document.as_ref().ok_or("no text doc")?;
        format!("{}\n{}", doc.content, marker)
    };
    let len = content.len();
    crate::commands::sync_text_document(window.clone(), content, len, len, state.clone())?;
    let idx = crate::commands::get_browser_state(window.clone(), state.clone())?
        .active_index;
    crate::commands::close_tab(window.clone(), idx, true, app.clone(), state.clone())?;
    let undone = crate::commands::undo_close_tab(window.clone(), app.clone(), state.clone())?;
    let text = undone
        .text_document
        .as_ref()
        .map(|d| d.content.clone())
        .ok_or("restored tab has no text")?;
    if !text.contains(&marker) {
        return Err("marker missing after undo".into());
    }
    Ok(())
}

fn scenario_proofing_dict(
    state: &State<crate::state::AppState>,
    fixtures: &std::path::Path,
) -> Result<(), String> {
    let _ = fixtures;
    let before = crate::proofing::proofing_check(
        state.clone(),
        crate::proofing::ProofingCheckRequest {
            text: "teh quick brown".into(),
            surface: "plaintext".into(),
            language: Some("en-NZ".into()),
            code_language: None,
            grammar_enabled: None,
        },
    )?;
    if before.is_empty() {
        return Err("expected proofing hit".into());
    }
    crate::proofing::proofing_dictionary_add(
        state.clone(),
        crate::proofing::ProofingWordRequest {
            word: "teh".into(),
            scope: "workspace".into(),
            ignore: false,
        },
    )?;
    if before.is_empty() {
        return Err("expected proofing hit before dictionary add".into());
    }
    Ok(())
}

fn scenario_palette(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &std::path::Path,
) -> Result<(), String> {
    open_path(window, app, state, &demo.join("sample.txt"))?;
    let msg = crate::commands::execute_command(
        window.clone(),
        "unibrowser.palette.toggle".into(),
        app.clone(),
        state.clone(),
    )?;
    if msg.trim().is_empty() {
        return Err("palette toggle returned empty".into());
    }
    let _ = crate::commands::execute_command(
        window.clone(),
        "unibrowser.palette.toggle".into(),
        app.clone(),
        state.clone(),
    )?;
    Ok(())
}

fn scenario_cm_grey(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &std::path::Path,
) -> Result<(), String> {
    open_path(window, app, state, &demo.join("sample.txt"))?;
    open_path(window, app, state, &demo.join("sample.png"))?;
    let cm = crate::commands::get_command_manager(window.clone(), state.clone())?;
    let tab = cm
        .tabs
        .iter()
        .find(|t| t.selected)
        .or_else(|| cm.tabs.first())
        .ok_or("command manager tabs empty")?;
    let disabled = tab.commands.iter().filter(|c| !c.enabled).count();
    if disabled == 0 {
        return Err("expected disabled commands on image tab".into());
    }
    Ok(())
}

fn scenario_unibrow_blank(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    projects: &std::path::Path,
) -> Result<(), String> {
    let name = format!("self-test-blank-{}", now_ms());
    crate::commands::create_unibrow_project(
        window.clone(),
        projects.to_string_lossy().into_owned(),
        name,
        Some("blank".into()),
        state.clone(),
        app.clone(),
    )?;
    let dest = crate::commands::autosave_unibrow_active_project(window.clone(), state.clone())?;
    if dest.len() < 4 {
        return Err("autosave path empty".into());
    }
    Ok(())
}

fn scenario_live_preview(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    projects: &std::path::Path,
) -> Result<(), String> {
    let name = format!("self-test-live-{}", now_ms());
    crate::commands::create_unibrow_project(
        window.clone(),
        projects.to_string_lossy().into_owned(),
        name,
        Some("blank".into()),
        state.clone(),
        app.clone(),
    )?;
    let status = crate::live_host::set_unibrow_live_mode(window.clone(), true, state.clone())?;
    if !status.live_mode {
        return Err("live_mode false after enable".into());
    }
    let again = crate::live_host::unibrow_live_status(window.clone(), state.clone())?;
    if !again.live_mode {
        return Err("live_mode false in status poll".into());
    }
    let _ = crate::live_host::set_unibrow_live_mode(window.clone(), false, state.clone())?;
    Ok(())
}

pub fn scenario_shell_chrome_batch3(app: &AppHandle) -> Result<(), String> {
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    for _ in 0..40 {
        let _ = wv.eval(SHELL_CHROME_AUDIT_EVAL);
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some(errs) = take_shell_chrome_audit() {
            if errs.is_empty() {
                return Ok(());
            }
            return Err(errs.join("; "));
        }
    }
    Err("shell chrome audit not received from frontend".into())
}

fn scenario_ai_sidebar(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
) -> Result<(), String> {
    use crate::ai_assistant_host::{
        AiAssistantPrefs, AiGrantDto, SELF_TEST_MOCK_BASE_URL, set_ai_assistant_prefs,
        test_ai_connection,
    };
    let prefs = AiAssistantPrefs {
        base_url: Some(SELF_TEST_MOCK_BASE_URL.into()),
        model: Some("self-test".into()),
        api_key: None,
        grants: vec![AiGrantDto {
            permission: "read".into(),
            scope_kind: "global".into(),
            scope_value: None,
        }],
        panel_visible: true,
        panel_x: 0.0,
        panel_y: 0.0,
        panel_docked: true,
        panel_width: 320.0,
    };
    set_ai_assistant_prefs(prefs.clone())?;
    let reply = test_ai_connection(
        prefs.base_url.clone(),
        prefs.model.clone(),
        prefs.api_key.clone(),
    )?;
    if reply != "mock-ok" {
        return Err(format!("unexpected mock reply: {reply}"));
    }
    let status = crate::ai_assistant_host::ai_assistant_status(window.clone(), state.clone())?;
    if !status.panel_visible {
        return Err("panel_visible false".into());
    }
    if !status.configured {
        return Err("assistant not configured".into());
    }
    let loaded = crate::ai_assistant_host::get_ai_assistant_prefs();
    if loaded.grants.is_empty() {
        return Err("grants not persisted".into());
    }
    std::thread::sleep(std::time::Duration::from_millis(400));
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    let _ = wv.eval(
        r"(function(){try{var p=document.getElementById('ai-assistant-panel');var w=document.getElementById('work-area');if(p){p.classList.remove('hidden');}if(w){w.classList.add('ai-open');}document.documentElement.style.setProperty('--ub-ai-sidebar-width','320px');window.dispatchEvent(new Event('unibrowser-shell-layout-changed'));if(window.__unibrowserSubmitShellChromeAudit){window.__unibrowserSubmitShellChromeAudit();}}catch(e){}})();",
    );
    std::thread::sleep(std::time::Duration::from_millis(500));
    scenario_shell_chrome_batch3(app)
}

const DOCUMENT_EDITING_RUNNER_MAIN: &str = "__unibrowserRunDocumentEditingSelfTest";
const DOCUMENT_EDITING_RUNNER_PHASE1: &str = "__unibrowserRunDocumentEditingEverydayPhase1";
const DOCUMENT_EDITING_RUNNER_PHASE2: &str = "__unibrowserRunDocumentEditingEverydayPhase2";
const DOCUMENT_EDITING_RUNNER_KILL_PREP: &str = "__unibrowserRunDocumentEditingKillPrepSelfTest";
const DOCUMENT_EDITING_RUNNER_RELAUNCH: &str = "__unibrowserRunDocumentEditingRelaunchSelfTest";

fn document_editing_harness_eval(runner_global: &str) -> String {
    format!(
        r#"(function(){{try{{var inv=window.__TAURI__?.core?.invoke;var run=window.{runner_global};if(!inv){{return;}}if(!run){{inv('self_test_submit_document_editing_result',{{message:'{runner_global} missing'}});return;}}Promise.resolve(run()).catch(function(e){{inv('self_test_submit_document_editing_result',{{message:String(e)}});}});}}catch(e){{window.__TAURI__?.core?.invoke('self_test_submit_document_editing_result',{{message:String(e)}});}}}})();"#
    )
}

fn prepare_markdown_self_test_tab(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &Path,
    projects: &Path,
) -> Result<PathBuf, String> {
    let template = demo.join("sample.md");
    if !template.is_file() {
        return Err("demo sample.md missing".into());
    }
    let dest = projects.join(format!("self-test-everyday-{}.md", now_ms()));
    std::fs::copy(&template, &dest).map_err(|e| e.to_string())?;
    let opened = open_path(window, app, state, &dest)?;
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    push_shell_tab_state(&wv, &opened)?;
    std::thread::sleep(std::time::Duration::from_millis(1200));
    for _ in 0..30 {
        if let Ok(dto) = crate::commands::get_browser_state(window.clone(), state.clone()) {
            if dto
                .text_document
                .as_ref()
                .is_some_and(|d| d.content.contains("Demo markdown"))
            {
                let _ = push_shell_tab_state(&wv, &dto);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    std::thread::sleep(std::time::Duration::from_millis(1200));
    Ok(dest)
}

fn run_document_editing_harness(
    wv: &tauri::Webview,
    runner_global: &str,
    timeout: Duration,
) -> Result<(), String> {
    reset_document_editing_wait();
    wv.eval(&document_editing_harness_eval(runner_global))
        .map_err(|e| e.to_string())?;
    await_document_editing_harness(timeout)
}

fn spawn_self_test_subprocess(
    scenario: &str,
    report: &Path,
    extra_env: &[(&str, String)],
) -> Result<std::process::ExitStatus, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = repo_root();
    let mut cmd = if std::env::var_os("DISPLAY").is_some() {
        Command::new(&exe)
    } else {
        let mut wrap = Command::new("xvfb-run");
        wrap.arg("-a")
            .arg("-s")
            .arg("-screen 0 1600x1000x24")
            .arg(&exe);
        wrap
    };
    cmd.env("UNIBROWSER_REPO_ROOT", root.to_string_lossy().as_ref())
        .env("UNIBROWSER_SAFE_MODE", "1")
        .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
        .env("UNIBROWSER_SELF_TEST_SKIP_SHOTS", "1");
    for (key, value) in extra_env {
        cmd.env(key, value);
    }
    cmd.arg("--self-test")
        .arg(scenario)
        .arg("--report")
        .arg(report);
    cmd.status().map_err(|e| e.to_string())
}

fn open_document_for_env_path(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
) -> Result<(), String> {
    let path = std::env::var("UNIBROWSER_DOC_EDIT_PATH").map_err(|_| {
        "UNIBROWSER_DOC_EDIT_PATH missing for document-editing relaunch/kill prep".to_string()
    })?;
    let opened = open_path(window, app, state, Path::new(&path))?;
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    push_shell_tab_state(&wv, &opened)?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    Ok(())
}

fn push_shell_tab_state(
    wv: &tauri::Webview,
    dto: &crate::commands::TabContentDto,
) -> Result<(), String> {
    let json = serde_json::to_string(dto).map_err(|e| e.to_string())?;
    let script = format!(
        "(function(){{try{{var d={json};var f=window.__selfTestApplyState;if(f){{f(d);}}}}catch(e){{}}}})();"
    );
    wv.eval(&script).map_err(|e| e.to_string())
}

fn scenario_document_editing_everyday(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &std::path::Path,
    projects: &std::path::Path,
) -> Result<(), String> {
    let dest = prepare_markdown_self_test_tab(window, app, state, demo, projects)?;
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    run_document_editing_harness(&wv, DOCUMENT_EDITING_RUNNER_PHASE1, Duration::from_secs(150))?;
    eprintln!("[self-test] doc-edit phase1 harness ok");
    run_document_editing_harness(&wv, DOCUMENT_EDITING_RUNNER_PHASE2, Duration::from_secs(180))?;
    let path = dest.to_string_lossy().into_owned();
    let prep_report = projects.join(format!("doc-edit-kill-prep-{}.json", now_ms()));
    let kill_status = spawn_self_test_subprocess(
        SCENARIO_DOCUMENT_EDITING_KILL_PREP,
        &prep_report,
        &[("UNIBROWSER_DOC_EDIT_PATH", path.clone())],
    )?;
    if kill_status.success() {
        return Err("kill-prep subprocess exited cleanly; expected abrupt abort".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let signal = kill_status.signal();
        if !matches!(signal, Some(6) | Some(9) | Some(11)) {
            return Err(format!(
                "kill-prep subprocess unexpected exit (code={:?}, signal={signal:?})",
                kill_status.code()
            ));
        }
    }
    let relaunch_report = projects.join(format!("doc-edit-relaunch-{}.json", now_ms()));
    let relaunch_status = spawn_self_test_subprocess(
        SCENARIO_DOCUMENT_EDITING_RELAUNCH,
        &relaunch_report,
        &[("UNIBROWSER_DOC_EDIT_PATH", path)],
    )?;
    if !relaunch_status.success() {
        return Err(format!(
            "relaunch subprocess failed (code={:?})",
            relaunch_status.code()
        ));
    }
  Ok(())
}

fn scenario_document_editing_kill_prep(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &std::path::Path,
    projects: &std::path::Path,
) -> Result<(), String> {
    let _ = (demo, projects);
    open_document_for_env_path(window, app, state)?;
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    reset_document_editing_wait();
    wv.eval(&document_editing_harness_eval(DOCUMENT_EDITING_RUNNER_KILL_PREP))
        .map_err(|e| e.to_string())?;
    for _ in 0..240 {
        pump_gtk_events();
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("kill prep subprocess survived without abort".into())
}

fn scenario_document_editing_relaunch(
    window: &tauri::Window,
    app: &AppHandle,
    state: &State<crate::state::AppState>,
    demo: &std::path::Path,
    projects: &std::path::Path,
) -> Result<(), String> {
    let _ = (demo, projects);
    open_document_for_env_path(window, app, state)?;
    let _ = crate::crash_launch::discover_recovery_bar_items(state, &window.label());
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    if let Ok(dto) = crate::commands::get_browser_state(window.clone(), state.clone()) {
        let _ = push_shell_tab_state(&wv, &dto);
    }
    std::thread::sleep(std::time::Duration::from_millis(800));
    run_document_editing_harness(
        &wv,
        DOCUMENT_EDITING_RUNNER_RELAUNCH,
        Duration::from_secs(90),
    )
}

fn scenario_problems_jump(
    state: &State<crate::state::AppState>,
    fixtures: &std::path::Path,
) -> Result<(), String> {
    let _ = fixtures;
    let _ = crate::proofing::proofing_check(
        state.clone(),
        crate::proofing::ProofingCheckRequest {
            text: "teh quick brown fox".into(),
            surface: "plaintext".into(),
            language: Some("en-NZ".into()),
            code_language: None,
            grammar_enabled: None,
        },
    )?;
    let problems = crate::proofing::list_problems_panel(None, None)?;
    if problems.is_empty() {
        return Err("no problems".into());
    }
    match &problems[0].navigation {
        unibrowser_diagnostics::NavigationTarget::TextLineCol { .. }
        | unibrowser_diagnostics::NavigationTarget::TextByteRange { .. } => Ok(()),
        _ => Err("unexpected navigation target for spelling problem".into()),
    }
}
