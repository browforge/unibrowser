//! VSIX TextMate grammars for Monaco (`vscode-textmate` + Oniguruma WASM).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::VsixError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsixGrammarEntry {
    pub grammar_id: String,
    pub language: String,
    pub scope_name: String,
    pub extension_id: String,
    /// Path relative to `extension/` inside the install dir.
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonacoGrammarDto {
    pub language: String,
    pub scope_name: String,
    /// Raw JSON (tmLanguage / plist converted to JSON).
    pub grammar_json: String,
    pub extension_id: String,
}

pub fn grammar_id_for(extension_id: &str, language: &str) -> String {
    format!("vsix:{}:{}", extension_id, language)
}

pub fn discover_grammars_on_install(
    extension_id: &str,
    install_dir: &Path,
    grammar_contribs: &[(Option<String>, Option<String>, Option<String>)],
) -> (Vec<VsixGrammarEntry>, Vec<String>) {
    let mut grammars = Vec::new();
    let mut notes = Vec::new();
    for g in grammar_contribs {
        let language = g.0.clone().unwrap_or_else(|| "unknown".into());
        let scope = g.1.clone().unwrap_or_else(|| "source.unknown".into());
        let rel = g.2.clone().unwrap_or_default();
        if rel.is_empty() {
            notes.push(format!(
                "{language} grammar — recognised, not applied (no path)"
            ));
            continue;
        }
        let path = install_dir.join("extension").join(&rel);
        if !path.is_file() {
            notes.push(format!(
                "{language} grammar ({rel}) — recognised, not applied (missing file)"
            ));
            continue;
        }
        if load_grammar_json(&path).is_err() {
            notes.push(format!(
                "{language} grammar ({rel}) — recognised, not applied (invalid grammar file)"
            ));
            continue;
        }
        notes.push(format!(
            "{language} grammar ({scope}) — TextMate applied in Monaco when language matches"
        ));
        grammars.push(VsixGrammarEntry {
            grammar_id: grammar_id_for(extension_id, &language),
            language,
            scope_name: scope,
            extension_id: extension_id.to_string(),
            path: rel,
        });
    }
    (grammars, notes)
}

fn load_grammar_json(path: &Path) -> Result<String, VsixError> {
    let raw = fs::read_to_string(path)?;
    let trimmed = raw.trim();
    if trimmed.starts_with('{') {
        return Ok(raw);
    }
    Err(VsixError::Msg("expected JSON tmLanguage".into()))
}

pub fn list_enabled_grammars(root: &Path) -> Result<Vec<MonacoGrammarDto>, VsixError> {
    let mut out = Vec::new();
    for rec in crate::list_installed(root)? {
        if !rec.enabled {
            continue;
        }
        for g in &rec.grammars {
            let install = PathBuf::from(&rec.install_dir);
            let path = install.join("extension").join(&g.path);
            let grammar_json = load_grammar_json(&path)?;
            out.push(MonacoGrammarDto {
                language: g.language.clone(),
                scope_name: g.scope_name.clone(),
                grammar_json,
                extension_id: rec.extension_id.clone(),
            });
        }
    }
    Ok(out)
}

pub fn resolve_grammar_for_language(
    root: &Path,
    language_id: &str,
) -> Result<Option<MonacoGrammarDto>, VsixError> {
    for dto in list_enabled_grammars(root)? {
        if dto.language == language_id {
            return Ok(Some(dto));
        }
    }
    Ok(None)
}
