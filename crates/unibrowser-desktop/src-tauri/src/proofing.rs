//! Offline proofing (ADR 0021): Tauri commands and dictionary paths.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use unibrowser_proofing::{
    CheckContext, Diagnostic, ProofingEngine, ProofingLanguage, ProofingSurface,
};

use crate::config::config_dir;
use crate::preferences::load_preferences;
use crate::state::AppState;
use tauri::State;

static ENGINE: OnceLock<Mutex<ProofingEngine>> = OnceLock::new();

fn engine() -> Result<&'static Mutex<ProofingEngine>, String> {
    Ok(ENGINE.get_or_init(|| {
        Mutex::new(
            ProofingEngine::with_bundled_dictionaries()
                .expect("bundled proofing dictionaries"),
        )
    }))
}

pub fn global_dictionary_added_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("proofing-global-added.txt"))
}

pub fn global_dictionary_ignored_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("proofing-global-ignored.txt"))
}

pub fn workspace_dictionary_added_path(cwd: &Path) -> PathBuf {
    cwd.join(".unibrowser").join("proofing-added.txt")
}

pub fn workspace_dictionary_ignored_path(cwd: &Path) -> PathBuf {
    cwd.join(".unibrowser").join("proofing-ignored.txt")
}

fn load_store(path: &Path) -> unibrowser_proofing::DictionaryStore {
    if path.is_file() {
        unibrowser_proofing::DictionaryStore::load_plain_file(path).unwrap_or_default()
    } else {
        unibrowser_proofing::DictionaryStore::default()
    }
}

fn reload_dictionaries(engine: &mut ProofingEngine, cwd: &Path) {
    let mut global = global_dictionary_added_path()
        .map(|p| load_store(&p))
        .unwrap_or_default();
    if let Some(p) = global_dictionary_ignored_path() {
        global.ignored = load_store(&p).added;
    }
    let mut workspace = load_store(&workspace_dictionary_added_path(cwd));
    workspace.ignored = load_store(&workspace_dictionary_ignored_path(cwd)).added;
    engine.load_user_dictionaries(global, workspace);
}

#[derive(Debug, Deserialize)]
pub struct ProofingCheckRequest {
    pub text: String,
    pub surface: String,
    pub language: Option<String>,
    pub code_language: Option<String>,
    pub grammar_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticDto {
    pub start: u32,
    pub end: u32,
    pub kind: String,
    pub message: String,
    pub suggestions: Vec<String>,
}

fn surface_from_str(s: &str) -> ProofingSurface {
    match s {
        "markdown" => ProofingSurface::Markdown,
        "code" => ProofingSurface::Code,
        "docx" => ProofingSurface::DocxPreview,
        "svg" => ProofingSurface::SvgText,
        "unibrow_title" => ProofingSurface::UnibrowTitle,
        "web" => ProofingSurface::WebForm,
        _ => ProofingSurface::PlainText,
    }
}

fn language_from_str(s: Option<&str>) -> ProofingLanguage {
    match s {
        Some("en-US") | Some("en_US") => ProofingLanguage::EnUs,
        Some("en-GB") | Some("en_GB") => ProofingLanguage::EnGb,
        _ => ProofingLanguage::EnNz,
    }
}

fn dto(d: Diagnostic) -> DiagnosticDto {
    DiagnosticDto {
        start: d.start,
        end: d.end,
        kind: match d.kind {
            unibrowser_proofing::DiagnosticKind::Spelling => "spelling".into(),
            unibrowser_proofing::DiagnosticKind::Grammar => "grammar".into(),
            unibrowser_proofing::DiagnosticKind::Style => "style".into(),
        },
        message: d.message,
        suggestions: d.suggestions,
    }
}

#[tauri::command]
pub fn proofing_check(
    state: State<AppState>,
    req: ProofingCheckRequest,
) -> Result<Vec<DiagnosticDto>, String> {
    let prefs = load_preferences();
    if !prefs.proofing_squiggles_enabled {
        return Ok(Vec::new());
    }
    let eng = engine()?;
    let mut eng = eng.lock().map_err(|e| e.to_string())?;
    reload_dictionaries(&mut *eng, &state.cwd);
    let surface = surface_from_str(&req.surface);
    let mut ctx = CheckContext {
        language: language_from_str(req.language.as_deref()),
        surface,
        grammar_enabled: req.grammar_enabled.unwrap_or(prefs.proofing_grammar_enabled),
        check_ranges: None,
    };
    if surface == ProofingSurface::Code {
        let lang = req.code_language.as_deref().unwrap_or("plaintext");
        let segments = unibrowser_proofing::code_check_segments(&req.text, lang);
        ctx.check_ranges = Some(
            segments
                .iter()
                .filter(|s| {
                    s.kind == unibrowser_proofing::CodeSegmentKind::Comment
                        || s.kind == unibrowser_proofing::CodeSegmentKind::StringLiteral
                })
                .map(|s| (s.start, s.end))
                .collect(),
        );
    }
    Ok(eng.check(&req.text, &ctx).into_iter().map(dto).collect())
}

#[derive(Debug, Deserialize)]
pub struct ProofingWordRequest {
    pub word: String,
    pub scope: String,
    pub ignore: bool,
}

#[tauri::command]
pub fn proofing_dictionary_add(
    state: State<AppState>,
    req: ProofingWordRequest,
) -> Result<(), String> {
    let scope = match req.scope.as_str() {
        "workspace" => unibrowser_proofing::IgnoreScope::Workspace,
        _ => unibrowser_proofing::IgnoreScope::Global,
    };
    let path = if scope == unibrowser_proofing::IgnoreScope::Workspace {
        if req.ignore {
            workspace_dictionary_ignored_path(&state.cwd)
        } else {
            workspace_dictionary_added_path(&state.cwd)
        }
    } else if req.ignore {
        global_dictionary_ignored_path().ok_or("config dir unavailable")?
    } else {
        global_dictionary_added_path().ok_or("config dir unavailable")?
    };
    append_word_line(&path, &req.word)?;
    Ok(())
}

fn append_word_line(path: &Path, word: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut line = word.trim().to_string();
    if line.is_empty() {
        return Ok(());
    }
    let mut existing = String::new();
    if path.is_file() {
        existing = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    }
    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&line);
    existing.push('\n');
    std::fs::write(path, existing).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct DictionaryListDto {
    pub global_added: Vec<String>,
    pub global_ignored: Vec<String>,
    pub workspace_added: Vec<String>,
    pub workspace_ignored: Vec<String>,
}

#[tauri::command]
pub fn proofing_dictionary_list(state: State<AppState>) -> Result<DictionaryListDto, String> {
    let g_add = global_dictionary_added_path()
        .map(|p| load_store(&p))
        .unwrap_or_default();
    let g_ign = global_dictionary_ignored_path()
        .map(|p| load_store(&p))
        .unwrap_or_default();
    let w_add = load_store(&workspace_dictionary_added_path(&state.cwd));
    let w_ign = load_store(&workspace_dictionary_ignored_path(&state.cwd));
    Ok(DictionaryListDto {
        global_added: sorted_set(&g_add.added),
        global_ignored: sorted_set(&g_ign.ignored),
        workspace_added: sorted_set(&w_add.added),
        workspace_ignored: sorted_set(&w_ign.ignored),
    })
}

fn sorted_set(set: &std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.iter().cloned().collect();
    v.sort();
    v
}
