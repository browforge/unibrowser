//! VSIX TextMate grammars for Monaco.

use serde::Serialize;
use unibrowser_vsix::MonacoGrammarDto;

#[derive(Serialize)]
pub struct MonacoGrammarDtoOut {
    pub language: String,
    pub scope_name: String,
    pub grammar_json: String,
    pub extension_id: String,
}

pub fn list_monaco_grammars() -> Result<Vec<MonacoGrammarDtoOut>, String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    let list = unibrowser_vsix::list_enabled_grammars(&root).map_err(|e| e.to_string())?;
    Ok(list.into_iter().map(map).collect())
}

fn map(g: MonacoGrammarDto) -> MonacoGrammarDtoOut {
    MonacoGrammarDtoOut {
        language: g.language,
        scope_name: g.scope_name,
        grammar_json: g.grammar_json,
        extension_id: g.extension_id,
    }
}
