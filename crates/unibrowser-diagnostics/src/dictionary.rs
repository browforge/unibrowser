use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

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
}

#[derive(Debug, Clone, Default)]
pub struct DictionarySnapshot {
    pub global: DictionaryStore,
    pub workspace: DictionaryStore,
}

impl DictionarySnapshot {
    pub fn should_suppress_spelling(&self, word: &str) -> bool {
        let w = normalise_word(word);
        if self.workspace.added.contains(&w) || self.global.added.contains(&w) {
            return true;
        }
        if self.workspace.ignored.contains(&w) || self.global.ignored.contains(&w) {
            return true;
        }
        false
    }

    /// Workspace overrides global for the same token.
    pub fn workspace_wins_over_global_added(&self, word: &str) -> bool {
        let w = normalise_word(word);
        self.workspace.added.contains(&w) && !self.global.added.contains(&w)
    }
}

fn normalise_word(w: &str) -> String {
    w.to_lowercase()
}
