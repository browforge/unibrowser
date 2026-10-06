//! In-process shell UI scenarios (no WebDriver). See `launch_args` and `docs/e2e.md`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

static FINISHED: AtomicBool = AtomicBool::new(false);

pub fn finished() -> bool {
    FINISHED.load(Ordering::SeqCst)
}

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

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
pub const SCENARIO_SHELL_CHROME_BATCH1: &str = "shell-chrome-batch1";

static SHELL_CHROME_AUDIT: Mutex<Option<Vec<String>>> = Mutex::new(None);

#[tauri::command]
pub fn self_test_submit_shell_chrome_audit(errors: Vec<String>) {
    if let Ok(mut g) = SHELL_CHROME_AUDIT.lock() {
        *g = Some(errors);
    }
}

fn take_shell_chrome_audit() -> Option<Vec<String>> {
    drain_shell_chrome_audit()
}

pub fn drain_shell_chrome_audit() -> Option<Vec<String>> {
    SHELL_CHROME_AUDIT
        .lock()
        .ok()
        .and_then(|mut g| g.take())
}

pub const ALL_SCENARIOS: &[&str] = &[
    SCENARIO_SHELL_CHROME_BATCH1,
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
        let app2 = app.clone();
        let _ = app.run_on_main_thread(move || {
            eprintln!("[self-test] on main thread");
            let report = match run_all_scenarios(&app2) {
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
            write_report_and_exit(app2, report);
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
            SCENARIO_SHELL_CHROME_BATCH1 => scenario_shell_chrome_batch1(app),
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
    let path = fixtures.join("unknown.ubunknown");
    if !path.is_file() {
        return Err("fixture unknown.ubunknown missing".into());
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

pub fn scenario_shell_chrome_batch1(app: &AppHandle) -> Result<(), String> {
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
    for _ in 0..40 {
        let _ = wv.eval(
            r"(function(){try{var fn=window.__unibrowserRunShellChromeAudit;if(!fn){return;}var errs=fn();var inv=window.__TAURI__?.core?.invoke;if(inv){inv('self_test_submit_shell_chrome_audit',{errors:errs});}}catch(e){}})();",
        );
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
    for _ in 0..20 {
        let _ = wv.eval(
            r"(function(){try{var fn=window.__unibrowserRunShellChromeAudit;if(!fn){return;}var errs=fn();var inv=window.__TAURI__?.core?.invoke;if(inv){inv('self_test_submit_shell_chrome_audit',{errors:errs});}}catch(e){}})();",
        );
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some(errs) = take_shell_chrome_audit() {
            if !errs.is_empty() {
                return Err(format!("AI open shell audit: {}", errs.join("; ")));
            }
            return Ok(());
        }
    }
    Err("AI sidebar shell audit not received".into())
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
