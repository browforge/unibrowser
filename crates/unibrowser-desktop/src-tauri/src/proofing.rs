//! Offline diagnostics / proofing (ADR 0021 host + ADR 0022 mods).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use spell_en_nz::SpellEnNzChecker;
use unibrowser_diagnostics::{
    DictionarySnapshot, DictionaryStore, DiagnosticsHost, IgnoreScope, Locator, Problem, Severity,
    TextCheckContext,
};

use crate::config::config_dir;
use crate::preferences::load_preferences;
use crate::state::AppState;
use tauri::State;

static HOST: OnceLock<Mutex<DiagnosticsHost>> = OnceLock::new();

fn host() -> Result<&'static Mutex<DiagnosticsHost>, String> {
    Ok(HOST.get_or_init(|| {
        let mut h = DiagnosticsHost::new();
        if let Ok(spell) = SpellEnNzChecker::with_bundled() {
            h.register_checker(Box::new(spell));
        }
        Mutex::new(h)
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

fn load_store(path: &Path) -> DictionaryStore {
    if path.is_file() {
        DictionaryStore::load_plain_file(path).unwrap_or_default()
    } else {
        DictionaryStore::default()
    }
}

fn reload_dictionary_snapshot(cwd: &Path) -> DictionarySnapshot {
    let mut global = global_dictionary_added_path()
        .map(|p| load_store(&p))
        .unwrap_or_default();
    if let Some(p) = global_dictionary_ignored_path() {
        global.ignored = load_store(&p).added;
    }
    let mut workspace = load_store(&workspace_dictionary_added_path(cwd));
    workspace.ignored = load_store(&workspace_dictionary_ignored_path(cwd)).added;
    DictionarySnapshot {
        global,
        workspace,
    }
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

fn dto(p: &Problem) -> Option<DiagnosticDto> {
    let (start, end) = match &p.locator {
        Locator::TextRange {
            start_byte,
            end_byte,
            ..
        } => (*start_byte, *end_byte),
        _ => return None,
    };
    let kind = if p.source.contains("grammar") || p.source.contains("harper") {
        "grammar".into()
    } else if p.source.contains("spell") {
        "spelling".into()
    } else {
        match p.severity {
            Severity::Error | Severity::Warning => "grammar".into(),
            _ => "spelling".into(),
        }
    };
    Some(DiagnosticDto {
        start,
        end,
        kind,
        message: p.message.clone(),
        suggestions: p.suggestions.clone(),
    })
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
    let h = host()?;
    let mut h = h.lock().map_err(|e| e.to_string())?;
    h.set_dictionary(reload_dictionary_snapshot(&state.cwd));
    let code_mode = req.surface == "code";
    let language_id = if code_mode {
        req.code_language.as_deref().unwrap_or("plaintext").to_string()
    } else {
        req.language.clone().unwrap_or_else(|| "en-NZ".into())
    };
    let _grammar = req.grammar_enabled.unwrap_or(prefs.proofing_grammar_enabled);
    let ctx = TextCheckContext {
        language_id,
        grammar_enabled: false,
        code_mode,
    };
    let doc_uri = format!("unibrowser://proofing/{}", req.surface);
    let problems = h.check_text_document(&doc_uri, &req.text, &ctx);
    Ok(problems.iter().filter_map(dto).collect())
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
        "workspace" => IgnoreScope::Workspace,
        _ => IgnoreScope::Global,
    };
    let path = if scope == IgnoreScope::Workspace {
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
