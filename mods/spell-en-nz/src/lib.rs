use spellbook::Dictionary;
use unibrowser_engine::CommandRegistry;
use unibrowser_diagnostics::{
    DictionarySnapshot, Locator, NativeDiagnosticsChecker, Problem, Severity, TextCheckContext,
};

pub const MOD_ID: &str = "spell-en-nz";

pub fn register(_registry: &mut CommandRegistry) {}

pub struct SpellEnNzChecker {
    dictionary: Dictionary,
}

impl SpellEnNzChecker {
    pub fn with_bundled() -> Result<Self, String> {
        let aff = include_str!("../dictionaries/en.aff");
        let dic = include_str!("../dictionaries/en-GB.dic");
        let dictionary = Dictionary::new(aff, dic).map_err(|e| format!("{}", e))?;
        Ok(Self { dictionary })
    }
}

impl NativeDiagnosticsChecker for SpellEnNzChecker {
    fn checker_id(&self) -> &str {
        MOD_ID
    }

    fn check_text(
        &self,
        doc_uri: &str,
        text: &str,
        ctx: &TextCheckContext,
        dict: &DictionarySnapshot,
    ) -> Vec<Problem> {
        let _ = ctx;
        let mut out = Vec::new();
        let mut dict_mut = self.dictionary.clone();
        for w in dict.global.added.iter().chain(dict.workspace.added.iter()) {
            let _ = dict_mut.add(w);
        }
        for word in split_words(text) {
            if word.token.len() <= 1 {
                continue;
            }
            if dict.should_suppress_spelling(&word.token) {
                continue;
            }
            let lower = word.token.to_lowercase();
            if dict_mut.check(&word.token) || dict_mut.check(&lower) {
                continue;
            }
            let mut suggestions = Vec::new();
            dict_mut.suggest(&word.token, &mut suggestions);
            suggestions.truncate(5);
            out.push(Problem {
                id: format!("{}:{}:{}-{}", MOD_ID, doc_uri, word.start, word.end),
                source: MOD_ID.into(),
                severity: Severity::Information,
                message: format!("Unknown word: {}", word.token),
                locator: Locator::text_bytes(word.start, word.end),
                suggestions,
                code_actions: Vec::new(),
            });
        }
        out
    }
}

struct WordSpan {
    start: u32,
    end: u32,
    token: String,
}

#[cfg(test)]
mod bundled_dict_tests {
    use super::*;

    #[test]
    fn bundled_dictionary_recognizes_common_words() {
        let checker = SpellEnNzChecker::with_bundled().expect("bundled dictionary");
        let mut dict = checker.dictionary.clone();
        assert!(dict.check("quick"));
        assert!(dict.check("paragraph"));
        assert!(!dict.check("teh"));
    }
}

fn split_words(text: &str) -> Vec<WordSpan> {
    let mut out = Vec::new();
    let mut start: Option<u32> = None;
    for (i, ch) in text.char_indices() {
        if ch.is_alphabetic() || ch == '\'' {
            if start.is_none() {
                start = Some(i as u32);
            }
        } else if let Some(s) = start {
            let end = i as u32;
            out.push(WordSpan {
                start: s,
                end,
                token: text[s as usize..end as usize].to_string(),
            });
            start = None;
        }
    }
    if let Some(s) = start {
        let end = text.len() as u32;
        out.push(WordSpan {
            start: s,
            end,
            token: text[s as usize..].to_string(),
        });
    }
    out
}
