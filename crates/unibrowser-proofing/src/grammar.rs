use std::sync::Mutex;

use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use std::sync::Arc;

pub struct GrammarHit {
    pub start: u32,
    pub end: u32,
    pub message: String,
    pub suggestions: Vec<String>,
}

pub struct HarperGrammar {
    linter: Mutex<LintGroup>,
}

impl HarperGrammar {
    pub fn new() -> Result<Self, String> {
        let dict = Arc::new(FstDictionary::curated());
        let linter = LintGroup::new_curated(dict, Dialect::British);
        Ok(Self {
            linter: Mutex::new(linter),
        })
    }

    pub fn check_slice(&self, text: &str) -> Vec<GrammarHit> {
        let doc = Document::new_plain_english_curated(text);
        let mut linter = self.linter.lock().expect("grammar linter");
        let lints = linter.lint(&doc);
        lints
            .into_iter()
            .map(|lint| {
                let span = lint.span;
                GrammarHit {
                    start: span.start as u32,
                    end: span.end as u32,
                    message: lint.message,
                    suggestions: lint
                        .suggestions
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                }
            })
            .collect()
    }
}
