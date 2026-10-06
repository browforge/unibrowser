use serde::{Deserialize, Serialize};

use crate::locator::Locator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub source: String,
    pub severity: Severity,
    pub message: String,
    pub locator: Locator,
    pub suggestions: Vec<String>,
    pub code_actions: Vec<CodeAction>,
}
