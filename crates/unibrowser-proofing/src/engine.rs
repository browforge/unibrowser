use serde::{Deserialize, Serialize};

use crate::dictionaries::ProofingDictionaries;
use crate::grammar::HarperGrammar;
use crate::spelling::HunspellSpelling;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofingLanguage {
    EnNz,
    EnGb,
    EnUs,
}

impl ProofingLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnNz => "en-NZ",
            Self::EnGb => "en-GB",
            Self::EnUs => "en-US",
        }
    }

    pub fn default_locale() -> Self {
        Self::EnNz
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofingSurface {
    PlainText,
    Markdown,
    Code,
    DocxPreview,
    SvgText,
    UnibrowTitle,
    UnibrowVideoText,
    WebForm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckContext {
    pub language: ProofingLanguage,
    pub surface: ProofingSurface,
    /// When set, only these byte ranges are checked (e.g. comments/strings in code).
    pub check_ranges: Option<Vec<(u32, u32)>>,
    pub grammar_enabled: bool,
}

impl Default for CheckContext {
    fn default() -> Self {
        Self {
            language: ProofingLanguage::default_locale(),
            surface: ProofingSurface::PlainText,
            check_ranges: None,
            grammar_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticKind {
    Spelling,
    Grammar,
    Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub start: u32,
    pub end: u32,
    pub kind: DiagnosticKind,
    pub message: String,
    pub suggestions: Vec<String>,
}

pub struct ProofingEngine {
    spelling: HunspellSpelling,
    grammar: HarperGrammar,
    dictionaries: ProofingDictionaries,
}

impl ProofingEngine {
    pub fn with_bundled_dictionaries() -> Result<Self, String> {
        Ok(Self {
            spelling: HunspellSpelling::with_bundled()?,
            grammar: HarperGrammar::new()?,
            dictionaries: ProofingDictionaries::empty(),
        })
    }

    pub fn load_user_dictionaries(
        &mut self,
        global: DictionaryStore,
        workspace: DictionaryStore,
    ) {
        self.dictionaries = ProofingDictionaries {
            global,
            workspace,
        };
    }

    pub fn check(&mut self, text: &str, ctx: &CheckContext) -> Vec<Diagnostic> {
        if text.is_empty() {
            return Vec::new();
        }
        self.spelling.apply_stores(&self.dictionaries.global, &self.dictionaries.workspace);
        let ranges = effective_ranges(text, ctx);
        let mut out = Vec::new();
        for (start, end) in ranges {
            if start >= end || end as usize > text.len() {
                continue;
            }
            let slice = &text[start as usize..end as usize];
            if slice.trim().is_empty() {
                continue;
            }
            let offset = start;
            for d in self.spelling.check_slice(slice) {
                if self
                    .dictionaries
                    .should_ignore(&d.word, offset + d.start, offset + d.end)
                {
                    continue;
                }
                out.push(Diagnostic {
                    start: offset + d.start,
                    end: offset + d.end,
                    kind: DiagnosticKind::Spelling,
                    message: format!("Unknown word: {}", d.word),
                    suggestions: d.suggestions,
                });
            }
            if ctx.grammar_enabled {
                for g in self.grammar.check_slice(slice) {
                    out.push(Diagnostic {
                        start: offset + g.start,
                        end: offset + g.end,
                        kind: DiagnosticKind::Grammar,
                        message: g.message,
                        suggestions: g.suggestions,
                    });
                }
            }
        }
        out.sort_by_key(|d| d.start);
        out
    }
}

fn effective_ranges(text: &str, ctx: &CheckContext) -> Vec<(u32, u32)> {
    if let Some(ranges) = &ctx.check_ranges {
        return ranges.clone();
    }
    vec![(0, text.len() as u32)]
}

pub use crate::dictionaries::DictionaryStore;
