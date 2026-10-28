use crate::code_regions::{code_check_segments, CodeSegmentKind};
use crate::markdown_prose::markdown_prose_ranges;
use crate::dictionary::DictionarySnapshot;
use crate::locator::Locator;
use crate::native::NativeDiagnosticsChecker;
use crate::problem::Problem;

#[derive(Debug, Clone, Default)]
pub struct TextCheckContext {
    pub language_id: String,
    pub grammar_enabled: bool,
    pub code_mode: bool,
    pub markdown_mode: bool,
}

pub struct DiagnosticsHost {
    checkers: Vec<Box<dyn NativeDiagnosticsChecker>>,
    problems: Vec<Problem>,
    cursor: usize,
    dictionary: DictionarySnapshot,
}

impl DiagnosticsHost {
    pub fn new() -> Self {
        Self {
            checkers: Vec::new(),
            problems: Vec::new(),
            cursor: 0,
            dictionary: DictionarySnapshot::default(),
        }
    }

    pub fn register_checker(&mut self, checker: Box<dyn NativeDiagnosticsChecker>) {
        self.checkers.push(checker);
    }

    pub fn set_dictionary(&mut self, snapshot: DictionarySnapshot) {
        self.dictionary = snapshot;
    }

    pub fn dictionary(&self) -> &DictionarySnapshot {
        &self.dictionary
    }

    pub fn check_text_document(
        &mut self,
        doc_uri: &str,
        text: &str,
        ctx: &TextCheckContext,
    ) -> Vec<Problem> {
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        if ctx.code_mode {
            let segments = code_check_segments(text, &ctx.language_id);
            for s in segments {
                if s.kind == CodeSegmentKind::Comment || s.kind == CodeSegmentKind::StringLiteral {
                    ranges.push((s.start, s.end));
                }
            }
        } else if ctx.markdown_mode {
            ranges = markdown_prose_ranges(text);
        }
        let mut out = Vec::new();
        for checker in &self.checkers {
            let mut batch = checker.check_text(doc_uri, text, ctx, &self.dictionary);
            if (ctx.code_mode || ctx.markdown_mode) && !ranges.is_empty() {
                batch.retain(|p| {
                    if let Locator::TextRange { start_byte, end_byte, .. } = p.locator {
                        ranges.iter().any(|(s, e)| start_byte >= *s && end_byte <= *e)
                    } else {
                        true
                    }
                });
            }
            out.extend(batch);
        }
        out.sort_by(|a, b| problem_sort_key(a).cmp(&problem_sort_key(b)));
        self.problems = out;
        self.cursor = 0;
        self.problems.clone()
    }

    pub fn check_dxf(&mut self, path: &str, content: &str) -> Vec<Problem> {
        let mut out = Vec::new();
        for checker in &self.checkers {
            out.extend(checker.check_dxf(path, content));
        }
        self.problems = out.clone();
        self.cursor = 0;
        out
    }

    pub fn check_unibrow(&mut self, project_json: &str) -> Vec<Problem> {
        let mut out = Vec::new();
        for checker in &self.checkers {
            out.extend(checker.check_unibrow(project_json));
        }
        self.problems = out.clone();
        self.cursor = 0;
        out
    }

    pub fn problems(&self) -> &[Problem] {
        &self.problems
    }

    pub fn next_problem(&mut self) -> Option<&Problem> {
        if self.problems.is_empty() {
            return None;
        }
        self.cursor = (self.cursor + 1) % self.problems.len();
        Some(&self.problems[self.cursor])
    }

    pub fn previous_problem(&mut self) -> Option<&Problem> {
        if self.problems.is_empty() {
            return None;
        }
        self.cursor = (self.problems.len() + self.cursor - 1) % self.problems.len();
        Some(&self.problems[self.cursor])
    }
}

fn problem_sort_key(p: &Problem) -> (u32, u32) {
    match &p.locator {
        Locator::TextRange { start_byte, .. } => (*start_byte, 0),
        Locator::TimelineRange { t0_ms, .. } => (0, *t0_ms as u32),
        _ => (0, 0),
    }
}
