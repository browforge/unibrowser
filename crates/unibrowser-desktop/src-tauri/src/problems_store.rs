//! Aggregated problems for the dockable Problems panel (proofing, LSP, project checkers).

use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use unibrowser_diagnostics::{Locator, NavigationTarget, Problem, Severity, navigation_target};

static PROBLEMS: OnceLock<Mutex<Vec<Problem>>> = OnceLock::new();

fn store() -> &'static Mutex<Vec<Problem>> {
    PROBLEMS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn set_problems(items: Vec<Problem>) {
    if let Ok(mut g) = store().lock() {
        *g = items;
    }
}

pub fn append_problems(items: Vec<Problem>) {
    if let Ok(mut g) = store().lock() {
        g.extend(items);
    }
}

pub fn clear_problems() {
    if let Ok(mut g) = store().lock() {
        g.clear();
    }
}

pub fn all_problems() -> Vec<Problem> {
    store().lock().map(|g| g.clone()).unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub struct ProblemDto {
    pub id: String,
    pub source: String,
    pub severity: String,
    pub message: String,
    pub locator_kind: String,
    pub navigate_hint: Option<String>,
    pub navigation: NavigationTarget,
}

pub fn problem_to_dto(p: &Problem) -> ProblemDto {
    let (locator_kind, navigate_hint) = match &p.locator {
        Locator::TextRange {
            start_line,
            start_col,
            end_line,
            end_col,
            ..
        } => (
            "text".into(),
            Some(format!("{}:{}-{}:{}", start_line, start_col, end_line, end_col)),
        ),
        Locator::TimelineRange { track, t0_ms, t1_ms } => (
            "timeline".into(),
            Some(format!("{}@{}-{}ms", track, t0_ms, t1_ms)),
        ),
        Locator::DxfEntity { handle, .. } => ("dxf".into(), Some(handle.clone())),
        Locator::ImageRegion { x, y, w, h } => (
            "image".into(),
            Some(format!("{},{},{},{}", x, y, w, h)),
        ),
        Locator::PdfRegion { page, .. } => ("pdf".into(), Some(format!("page {}", page))),
        Locator::DomNode { selector } => ("dom".into(), Some(selector.clone())),
    };
    let severity = match p.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Information => "info",
        Severity::Hint => "hint",
    };
    ProblemDto {
        id: p.id.clone(),
        source: p.source.clone(),
        severity: severity.into(),
        message: p.message.clone(),
        locator_kind,
        navigate_hint,
        navigation: navigation_target(&p.locator),
    }
}
