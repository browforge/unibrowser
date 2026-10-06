use crate::dictionaries::DictionaryStore;
use crate::engine::ProofingLanguage;

pub struct SpellingHit {
    pub start: u32,
    pub end: u32,
    pub word: String,
    pub suggestions: Vec<String>,
}

pub struct HunspellSpelling {
    dictionary: spellbook::Dictionary,
}

impl HunspellSpelling {
    pub fn with_bundled() -> Result<Self, String> {
        let (aff, dic) = crate::dictionaries::bundled_aff_dic(ProofingLanguage::EnNz)?;
        let dictionary = spellbook::Dictionary::new(&aff, &dic)
            .map_err(|e| format!("dictionary parse: {}", e))?;
        Ok(Self { dictionary })
    }

    pub fn apply_stores(&mut self, global: &DictionaryStore, workspace: &DictionaryStore) {
        for w in global.added.iter().chain(workspace.added.iter()) {
            let _ = self.dictionary.add(w);
        }
    }

    pub fn check_slice(&self, text: &str) -> Vec<SpellingHit> {
        let mut out = Vec::new();
        for word in split_words(text) {
            if should_skip_token(&word.token) {
                continue;
            }
            if self.dictionary.check(&word.token) {
                continue;
            }
            let mut suggestions = Vec::new();
            self.dictionary.suggest(&word.token, &mut suggestions);
            suggestions.truncate(5);
            out.push(SpellingHit {
                start: word.start,
                end: word.end,
                word: word.token,
                suggestions,
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
            let token = text[s as usize..end as usize].to_string();
            out.push(WordSpan { start: s, end, token });
            start = None;
        }
    }
    if let Some(s) = start {
        let end = text.len() as u32;
        let token = text[s as usize..].to_string();
        out.push(WordSpan { start: s, end, token });
    }
    out
}

fn should_skip_token(token: &str) -> bool {
    if token.len() <= 1 {
        return true;
    }
    if token.starts_with("http://") || token.starts_with("https://") {
        return true;
    }
    if token.contains('/') || token.contains('\\') {
        return true;
    }
    if token.chars().all(|c| c.is_ascii_hexdigit()) && token.len() >= 4 {
        return true;
    }
    false
}
