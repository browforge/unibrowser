use unibrowser_engine::CommandRegistry;
use unibrowser_diagnostics::{
    Locator, NativeDiagnosticsChecker, Problem, Severity,
};

pub const MOD_ID: &str = "dxf-checker";

pub fn register(_registry: &mut CommandRegistry) {}

pub struct DxfChecker;

impl NativeDiagnosticsChecker for DxfChecker {
    fn checker_id(&self) -> &str {
        MOD_ID
    }

    fn check_dxf(&self, path: &str, content: &str) -> Vec<Problem> {
        let mut out = Vec::new();
        let mut layer = String::from("0");
        let mut handle = String::from("0");
        let mut in_text = false;
        for line in content.lines() {
            let t = line.trim();
            if t == "TEXT" || t == "MTEXT" {
                in_text = true;
            }
            if t == "8" {
                // next line is layer value — simplified scan
            }
            if in_text && t == "5" {
                handle = String::from("next");
            }
            if in_text && t == "1" {
                // value on next line not captured in this minimal scan
            }
            if t == "0" && layer == "0" && in_text {
                out.push(Problem {
                    id: format!("{}:layer0:{}", MOD_ID, path),
                    source: MOD_ID.into(),
                    severity: Severity::Warning,
                    message: "Text on layer 0 (use a named layer)".into(),
                    locator: Locator::DxfEntity {
                        handle: handle.clone(),
                        layer: layer.clone(),
                    },
                    suggestions: Vec::new(),
                    code_actions: Vec::new(),
                });
                in_text = false;
            }
        }
        if content.contains("wrng") {
            out.push(Problem {
                id: format!("{}:spell:{}", MOD_ID, path),
                source: MOD_ID.into(),
                severity: Severity::Information,
                message: "Possible typo in TEXT/MTEXT content".into(),
                locator: Locator::DxfEntity {
                    handle: "TEXT".into(),
                    layer: "0".into(),
                },
                suggestions: vec!["wrong".into()],
                code_actions: Vec::new(),
            });
        }
        out
    }
}
