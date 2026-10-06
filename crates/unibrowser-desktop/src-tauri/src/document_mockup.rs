//! Seeded Problems panel rows for docs screenshot parity with design mockups.

use unibrowser_diagnostics::{Locator, Problem, Severity};

pub fn seed_document_mockup_problems() {
    let problems = vec![
        Problem {
            id: "mockup-spell-summery".into(),
            source: "Spell check".into(),
            severity: Severity::Error,
            message: "\"summery\" may be a misspelling of \"summary\"".into(),
            locator: Locator::TextRange {
                start_line: 8,
                start_col: 9,
                end_line: 8,
                end_col: 16,
                start_byte: 0,
                end_byte: 0,
            },
            suggestions: vec!["summary".into()],
            code_actions: vec![],
        },
        Problem {
            id: "mockup-ts-console".into(),
            source: "TypeScript".into(),
            severity: Severity::Warning,
            message: "Promise result is written to the console only".into(),
            locator: Locator::TextRange {
                start_line: 9,
                start_col: 3,
                end_line: 9,
                end_col: 49,
                start_byte: 0,
                end_byte: 0,
            },
            suggestions: vec![],
            code_actions: vec![],
        },
        Problem {
            id: "mockup-project-path".into(),
            source: "Project".into(),
            severity: Severity::Warning,
            message: "field-notes.md is outside the saved project sources".into(),
            locator: Locator::TextRange {
                start_line: 4,
                start_col: 15,
                end_line: 4,
                end_col: 40,
                start_byte: 0,
                end_byte: 0,
            },
            suggestions: vec![],
            code_actions: vec![],
        },
    ];
    crate::problems_store::set_problems(problems);
}

#[tauri::command]
pub fn seed_document_mockup_problems_cmd() -> Result<(), String> {
    seed_document_mockup_problems();
    Ok(())
}
