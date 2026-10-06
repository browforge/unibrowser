use std::sync::Mutex;

use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use std::sync::Arc;
use unibrowser_diagnostics::{
    DictionarySnapshot, Locator, NativeDiagnosticsChecker, Problem, Severity, TextCheckContext,
};
use unibrowser_engine::CommandRegistry;

pub const MOD_ID: &str = "grammar-harper";

pub fn register(_registry: &mut CommandRegistry) {}

pub struct HarperGrammarChecker {
    linter: Mutex<LintGroup>,
}

impl HarperGrammarChecker {
    pub fn new() -> Result<Self, String> {
        let dict = Arc::new(FstDictionary::curated());
        let linter = LintGroup::new_curated(dict, Dialect::British);
        Ok(Self {
            linter: Mutex::new(linter),
        })
    }
}

impl NativeDiagnosticsChecker for HarperGrammarChecker {
    fn checker_id(&self) -> &str {
        MOD_ID
    }

    fn check_text(
        &self,
        doc_uri: &str,
        text: &str,
        ctx: &TextCheckContext,
        _dict: &DictionarySnapshot,
    ) -> Vec<Problem> {
        if !ctx.grammar_enabled || ctx.code_mode {
            return Vec::new();
        }
        let doc = Document::new_plain_english_curated(text);
        let mut linter = self.linter.lock().expect("harper");
        let lints = linter.lint(&doc);
        lints
            .into_iter()
            .enumerate()
            .map(|(i, lint)| {
                let span = lint.span;
                Problem {
                    id: format!("{}:{}:{}", MOD_ID, doc_uri, i),
                    source: MOD_ID.into(),
                    severity: Severity::Warning,
                    message: lint.message,
                    locator: Locator::text_bytes(span.start as u32, span.end as u32),
                    suggestions: lint.suggestions.into_iter().map(|s| s.to_string()).collect(),
                    code_actions: Vec::new(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_disabled_returns_empty() {
        let checker = HarperGrammarChecker::new().expect("harper");
        let ctx = TextCheckContext {
            language_id: "en-NZ".into(),
            grammar_enabled: false,
            code_mode: false,
        };
        let problems =
            checker.check_text("doc", "This are wrong.", &ctx, &DictionarySnapshot::default());
        assert!(problems.is_empty());
    }
}
