//! Offline diagnostics / proofing (ADR 0021 host + ADR 0022 mods).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use dxf_checker::DxfChecker;
use grammar_harper::HarperGrammarChecker;
use spell_en_nz::SpellEnNzChecker;
use unibrow_checker::UnibrowChecker;
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
        if let Ok(grammar) = HarperGrammarChecker::new() {
            h.register_checker(Box::new(grammar));
        }
        h.register_checker(Box::new(DxfChecker));
        h.register_checker(Box::new(UnibrowChecker));
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

fn problem_byte_range(p: &Problem) -> Option<(u32, u32)> {
    match &p.locator {
        Locator::TextRange {
            start_byte,
            end_byte,
            ..
        } => Some((*start_byte, *end_byte)),
        _ => None,
    }
}

fn ranges_overlap(a: (u32, u32), b: (u32, u32)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

fn problem_source_priority(source: &str) -> u8 {
    let s = source.to_ascii_lowercase();
    if s.contains("spell") {
        0
    } else if s.contains("lsp") || s.contains("typescript") || s.contains("eslint") {
        1
    } else if s.contains("harper") || s.contains("grammar") {
        2
    } else {
        3
    }
}

/// Merge overlapping diagnostics into one row (one squiggle / Problems count) while
/// keeping every checker’s suggestions for quick-fix.
fn merge_overlapping_problems(problems: Vec<Problem>) -> Vec<Problem> {
    let n = problems.len();
    if n <= 1 {
        return problems;
    }
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] != i {
            let root = find(parent, parent[i]);
            parent[i] = root;
            root
        } else {
            i
        }
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if let (Some(ai), Some(aj)) =
                (problem_byte_range(&problems[i]), problem_byte_range(&problems[j]))
            {
                if ranges_overlap(ai, aj) {
                    union(&mut parent, i, j);
                }
            }
        }
    }
    let mut groups: std::collections::HashMap<usize, Vec<Problem>> =
        std::collections::HashMap::new();
    for (i, p) in problems.into_iter().enumerate() {
        groups.entry(find(&mut parent, i)).or_default().push(p);
    }
    let mut merged: Vec<Problem> = groups
        .into_values()
        .map(|members| merge_problem_cluster(&members))
        .collect();
    merged.sort_by(|a, b| problem_byte_range(a).cmp(&problem_byte_range(b)));
    merged
}

fn merge_problem_cluster(members: &[Problem]) -> Problem {
    let mut ordered: Vec<&Problem> = members.iter().collect();
    ordered.sort_by_key(|p| {
        (
            problem_source_priority(&p.source),
            problem_byte_range(p).unwrap_or((0, 0)),
        )
    });
    let primary = ordered[0];
    let mut suggestions: Vec<String> = Vec::new();
    let mut seen_suggestions: HashSet<String> = HashSet::new();
    let mut by_source = ordered.clone();
    by_source.sort_by_key(|p| problem_source_priority(&p.source));
    for p in by_source {
        for s in &p.suggestions {
            let key = s.trim().to_ascii_lowercase();
            if key.is_empty() || !seen_suggestions.insert(key) {
                continue;
            }
            suggestions.push(s.clone());
        }
    }
    let mut code_actions = primary.code_actions.clone();
    for p in ordered.iter().skip(1) {
        for action in &p.code_actions {
            if !code_actions.iter().any(|a| a.title == action.title && a.kind == action.kind) {
                code_actions.push(action.clone());
            }
        }
    }
    let severity = ordered
        .iter()
        .map(|p| p.severity)
        .max_by_key(|s| match s {
            Severity::Error => 3,
            Severity::Warning => 2,
            Severity::Information => 1,
            Severity::Hint => 0,
        })
        .unwrap_or(primary.severity);
    let mut sources: Vec<String> = ordered.iter().map(|p| p.source.clone()).collect();
    sources.sort();
    sources.dedup();
    Problem {
        id: primary.id.clone(),
        source: sources.join(" · "),
        severity,
        message: primary.message.clone(),
        locator: primary.locator.clone(),
        suggestions,
        code_actions,
    }
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
    let kind = if p.source.contains("spell") {
        "spelling".into()
    } else if p.source.contains("grammar") || p.source.contains("harper") {
        "grammar".into()
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
        crate::problems_store::set_problems_from_proofing(Vec::new());
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
    let grammar_on = req
        .grammar_enabled
        .unwrap_or(prefs.proofing_grammar_enabled)
        && req.surface != "markdown";
    let markdown_mode = req.surface == "markdown";
    let ctx = TextCheckContext {
        language_id,
        grammar_enabled: grammar_on,
        code_mode,
        markdown_mode,
    };
    let doc_uri = format!("unibrowser://proofing/{}", req.surface);
    let mut problems = h.check_text_document(&doc_uri, &req.text, &ctx);
    if req.surface == "code" {
        if let Ok(lsp_problems) = crate::lsp_host::lsp_check_text_cached(
            &state.cwd,
            &doc_uri,
            &ctx.language_id,
            &req.text,
        ) {
            problems.extend(lsp_problems);
        }
    }
    problems = merge_overlapping_problems(problems);
    crate::problems_store::set_problems_from_proofing(problems.clone());
    Ok(problems.iter().filter_map(dto).collect())
}

#[tauri::command]
pub fn list_problems_panel(
    severity: Option<String>,
    source: Option<String>,
) -> Result<Vec<crate::problems_store::ProblemDto>, String> {
    let items = crate::problems_store::all_problems();
    Ok(items
        .iter()
        .filter(|p| {
            severity.as_ref().map_or(true, |s| {
                crate::problems_store::problem_to_dto(p).severity == *s
            })
        })
        .filter(|p| {
            source.as_ref().map_or(true, |s| p.source.contains(s))
        })
        .map(crate::problems_store::problem_to_dto)
        .collect())
}

#[tauri::command]
pub fn proofing_check_unibrow_project(
    state: State<AppState>,
    project_json: String,
) -> Result<Vec<DiagnosticDto>, String> {
    let h = host()?;
    let mut h = h.lock().map_err(|e| e.to_string())?;
    h.set_dictionary(reload_dictionary_snapshot(&state.cwd));
    let problems = h.check_unibrow(&project_json);
    crate::problems_store::set_problems_from_proofing(problems.clone());
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

#[derive(Debug, Deserialize)]
pub struct ProofingRemoveWordRequest {
    pub word: String,
    pub scope: String,
    pub ignore: bool,
}

#[tauri::command]
pub fn proofing_dictionary_remove(
    state: State<AppState>,
    req: ProofingRemoveWordRequest,
) -> Result<(), String> {
    let path = dictionary_path_for_scope(&state.cwd, &req.scope, req.ignore)?;
    remove_word_line(&path, &req.word)?;
    Ok(())
}

fn dictionary_path_for_scope(cwd: &Path, scope: &str, ignore: bool) -> Result<PathBuf, String> {
    if scope == "workspace" {
        Ok(if ignore {
            workspace_dictionary_ignored_path(cwd)
        } else {
            workspace_dictionary_added_path(cwd)
        })
    } else if ignore {
        global_dictionary_ignored_path().ok_or("config dir unavailable".into())
    } else {
        global_dictionary_added_path().ok_or("config dir unavailable".into())
    }
}

fn remove_word_line(path: &Path, word: &str) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }
    let target = word.trim().to_lowercase();
    let lines: Vec<String> = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|l| l.trim().to_lowercase() != target)
        .map(|l| l.to_string())
        .collect();
    let body = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    std::fs::write(path, body).map_err(|e| e.to_string())
}

fn sorted_set(set: &std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.iter().cloned().collect();
    v.sort();
    v
}

#[cfg(test)]
mod rec_capture_proofing_tests {
    use super::*;

    #[test]
    fn spell_test_fixture_flags_only_teh() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../e2e/fixtures/spell-test.txt");
        let text = std::fs::read_to_string(&fixture).expect("spell-test fixture");
        let recovered = format!(
            "{}\n\nRecovered unsaved paragraph for REC capture.\n",
            text.trim_end()
        );
        let h = host().expect("host");
        let mut h = h.lock().expect("lock");
        h.set_dictionary(DictionarySnapshot::default());
        let ctx = TextCheckContext {
            language_id: "en-NZ".into(),
            grammar_enabled: false,
            code_mode: false,
            markdown_mode: false,
        };
        let problems = h.check_text_document("doc", &recovered, &ctx);
        let spell_hits: Vec<String> = problems
            .iter()
            .filter(|p| p.source.contains("spell"))
            .filter_map(|p| {
                if let Locator::TextRange { start_byte, end_byte, .. } = p.locator {
                    Some(recovered[start_byte as usize..end_byte as usize].to_string())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            spell_hits,
            vec!["teh".to_string()],
            "unexpected spell hits: {:?}",
            spell_hits
        );
    }

    #[test]
    fn live_block_sample_markdown_proofing_stays_quiet() {
        let sample = include_str!("../../../../examples/demo-pack/live-block-sample.md");
        let h = host().expect("host");
        let mut h = h.lock().expect("lock");
        h.set_dictionary(DictionarySnapshot::default());
        let ctx = TextCheckContext {
            language_id: "en-NZ".into(),
            grammar_enabled: false,
            code_mode: false,
            markdown_mode: true,
        };
        let raw = h.check_text_document("doc.md", sample, &ctx);
        let merged = super::merge_overlapping_problems(raw);
        assert_eq!(
            merged.len(),
            0,
            "capture sample should not spell-check prose markers or fenced code: {:?}",
            merged.iter().map(|p| p.message.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn live_block_sample_markdown_skips_fenced_rust_in_prose_ranges() {
        let sample = include_str!("../../../../examples/demo-pack/live-block-sample.md");
        let ranges = unibrowser_diagnostics::markdown_prose_ranges(sample);
        assert!(!ranges.iter().any(|(s, e)| sample[*s as usize..*e as usize].contains("fn main")));
        assert!(ranges.iter().any(|(s, e)| sample[*s as usize..*e as usize].contains("paragraph")));
    }

    #[test]
    fn rec_capture_banner_merges_spell_and_grammar_overlap_with_all_suggestions() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../e2e/fixtures/spell-test.txt");
        let banner = std::fs::read_to_string(&fixture).expect("spell-test fixture");
        let h = host().expect("host");
        let mut h = h.lock().expect("lock");
        h.set_dictionary(DictionarySnapshot::default());
        let ctx = TextCheckContext {
            language_id: "en-NZ".into(),
            grammar_enabled: true,
            code_mode: false,
            markdown_mode: false,
        };
        let raw = h.check_text_document("doc", &banner, &ctx);
        assert!(
            raw.len() >= 2,
            "expected spell + grammar overlap on 'teh', got {}",
            raw.len()
        );
        let merged = super::merge_overlapping_problems(raw.clone());
        assert_eq!(
            merged.len(),
            1,
            "expected one visible problem for banner text, got {:?}",
            merged
                .iter()
                .map(|p| format!("{}: {}", p.source, p.message))
                .collect::<Vec<_>>()
        );
        let grammar_suggestions: Vec<String> = raw
            .iter()
            .filter(|p| p.source.contains("harper") || p.source.contains("grammar"))
            .flat_map(|p| p.suggestions.clone())
            .collect();
        let spell_suggestions: Vec<String> = raw
            .iter()
            .filter(|p| p.source.contains("spell"))
            .flat_map(|p| p.suggestions.clone())
            .collect();
        assert!(
            !grammar_suggestions.is_empty() && !spell_suggestions.is_empty(),
            "fixture must expose grammar and spelling suggestions"
        );
        let merged_suggestions = &merged[0].suggestions;
        for s in grammar_suggestions {
            assert!(
                merged_suggestions.iter().any(|m| m.eq_ignore_ascii_case(&s)),
                "merged problem missing grammar suggestion {s:?}: {merged_suggestions:?}"
            );
        }
        for s in spell_suggestions {
            assert!(
                merged_suggestions.iter().any(|m| m.eq_ignore_ascii_case(&s)),
                "merged problem missing spelling suggestion {s:?}: {merged_suggestions:?}"
            );
        }
        assert!(
            merged_suggestions.iter().any(|s| s.eq_ignore_ascii_case("the")),
            "expected definite-article fix among suggestions: {merged_suggestions:?}"
        );
    }
}
