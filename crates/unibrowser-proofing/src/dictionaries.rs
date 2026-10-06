use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::engine::ProofingLanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgnoreScope {
    Global,
    Workspace,
    Once,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DictionaryStore {
    pub added: HashSet<String>,
    pub ignored: HashSet<String>,
}

impl DictionaryStore {
    pub fn load_plain_file(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Ok(Self::from_lines(&text))
    }

    pub fn from_lines(text: &str) -> Self {
        let mut added = HashSet::new();
        for line in text.lines() {
            let w = line.trim();
            if w.is_empty() || w.starts_with('#') {
                continue;
            }
            added.insert(normalise_word(w));
        }
        Self {
            added,
            ignored: HashSet::new(),
        }
    }

    pub fn save_plain_file(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut words: Vec<&String> = self.added.iter().chain(self.ignored.iter()).collect();
        words.sort();
        let body = words
            .iter()
            .map(|w| w.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, body).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProofingDictionaries {
    pub global: DictionaryStore,
    pub workspace: DictionaryStore,
}

impl ProofingDictionaries {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn should_ignore(&self, word: &str, _start: u32, _end: u32) -> bool {
        let w = normalise_word(word);
        if self.workspace.added.contains(&w) || self.global.added.contains(&w) {
            return true;
        }
        if self.workspace.ignored.contains(&w) || self.global.ignored.contains(&w) {
            return true;
        }
        false
    }

    pub fn add_word(&mut self, scope: IgnoreScope, word: &str, ignore: bool) {
        self.add_word_inner(scope, word, ignore);
    }
}

impl ProofingDictionaries {
    pub fn add_word_inner(&mut self, scope: IgnoreScope, word: &str, ignore: bool) {
        let w = normalise_word(word);
        match scope {
            IgnoreScope::Global => {
                if ignore {
                    self.global.ignored.insert(w);
                } else {
                    self.global.added.insert(w);
                }
            }
            IgnoreScope::Workspace => {
                if ignore {
                    self.workspace.ignored.insert(w);
                } else {
                    self.workspace.added.insert(w);
                }
            }
            IgnoreScope::Once => {}
        }
    }
}

pub struct WordListFile;

fn normalise_word(w: &str) -> String {
    w.to_lowercase()
}

/// Minimal bundled Hunspell-compatible aff/dic (FOSS word list for CI and offline default).
pub fn bundled_aff_dic(lang: ProofingLanguage) -> Result<(String, String), String> {
    let aff = include_str!("../dictionaries/en.aff");
    let dic = match lang {
        ProofingLanguage::EnUs => include_str!("../dictionaries/en-US.dic"),
        ProofingLanguage::EnGb | ProofingLanguage::EnNz => include_str!("../dictionaries/en-GB.dic"),
    };
    Ok((aff.to_string(), dic.to_string()))
}
