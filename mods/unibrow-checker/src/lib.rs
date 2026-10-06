use unibrowser_engine::CommandRegistry;
use unibrowser_diagnostics::{
    Locator, NativeDiagnosticsChecker, Problem, Severity,
};

pub const MOD_ID: &str = "unibrow-checker";

pub fn register(_registry: &mut CommandRegistry) {}

pub struct UnibrowChecker;

impl NativeDiagnosticsChecker for UnibrowChecker {
    fn checker_id(&self) -> &str {
        MOD_ID
    }

    fn check_unibrow(&self, project_json: &str) -> Vec<Problem> {
        let mut out = Vec::new();
        if project_json.contains("title_text") && project_json.contains("wrng") {
            out.push(Problem {
                id: format!("{}:title-spell", MOD_ID),
                source: MOD_ID.into(),
                severity: Severity::Information,
                message: "Title card may contain a spelling error".into(),
                locator: Locator::TimelineRange {
                    track: "titles".into(),
                    t0_ms: 0,
                    t1_ms: 3000,
                },
                suggestions: vec!["wrong".into()],
                code_actions: Vec::new(),
            });
        }
        if project_json.contains("\"blackdetect\"") || project_json.contains("black_frame") {
            out.push(Problem {
                id: format!("{}:black-frame", MOD_ID),
                source: MOD_ID.into(),
                severity: Severity::Warning,
                message: "Possible black frame detected (ffmpeg blackdetect stub)".into(),
                locator: Locator::TimelineRange {
                    track: "video".into(),
                    t0_ms: 1200,
                    t1_ms: 1250,
                },
                suggestions: Vec::new(),
                code_actions: Vec::new(),
            });
        }
        out
    }
}
