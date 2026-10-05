//! Known `vscode.*` API surface implemented by the Node host shim.

use std::collections::BTreeSet;

/// Normalised paths without the `vscode.` prefix (e.g. `window.showInformationMessage`).
pub fn shimmed_api_paths() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "commands.registerCommand",
        "commands.executeCommand",
        "window.showInformationMessage",
        "window.showWarningMessage",
        "window.showErrorMessage",
        "window.activeTextEditor",
        "workspace.getConfiguration",
        "workspace.applyEdit",
        "languages.registerDocumentFormattingEditProvider",
        "WorkspaceEdit",
        "Range",
        "Position",
        "Selection",
        "TextEdit",
    ])
}

/// `TextEditor.edit` is exposed on the editor object, not as `vscode.TextEditor.edit` in source.
pub fn is_shimmed_usage(api_path: &str) -> bool {
    let path = api_path.strip_prefix("vscode.").unwrap_or(api_path);
    if path == "TextEditor.edit" || path.ends_with(".edit") {
        return true;
    }
    if path.starts_with("TextEdit.") {
        return true;
    }
    shimmed_api_paths().contains(path)
}

/// Scan extension JavaScript for `vscode.` identifier chains.
pub fn scan_vscode_api_usages(source: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut i = 0;
    let bytes = source.as_bytes();
    while i < bytes.len() {
        if i + 6 < bytes.len() && bytes[i..].starts_with(b"vscode.") {
            i += 7;
            let start = i;
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' {
                    i += 1;
                } else {
                    break;
                }
            }
            if i > start {
                let path = source[start..i].trim_end_matches('.');
                if !path.is_empty() {
                    out.insert(path.to_string());
                }
            }
            continue;
        }
        i += 1;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatibilityLevel {
    Works,
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompatibilityReport {
    pub level: CompatibilityLevel,
    pub used_apis: Vec<String>,
    pub missing_apis: Vec<String>,
    pub shimmed_apis: Vec<String>,
}

pub fn compatibility_report(source: &str) -> CompatibilityReport {
    let used = scan_vscode_api_usages(source);
    let mut missing = Vec::new();
    let mut shimmed = Vec::new();
    for api in &used {
        if is_shimmed_usage(api) {
            shimmed.push(api.clone());
        } else {
            missing.push(api.clone());
        }
    }
    let level = if used.is_empty() {
        CompatibilityLevel::Works
    } else if missing.is_empty() {
        CompatibilityLevel::Works
    } else if shimmed.is_empty() {
        CompatibilityLevel::Unsupported
    } else {
        CompatibilityLevel::Partial
    };
    CompatibilityReport {
        level,
        used_apis: used.into_iter().collect(),
        missing_apis: missing,
        shimmed_apis: shimmed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_finds_nested_calls() {
        let src = r#"
          vscode.commands.registerCommand("x", () => {
            vscode.window.showInformationMessage("hi");
            foo(vscode.workspace.getConfiguration("a"));
          });
        "#;
        let used = scan_vscode_api_usages(src);
        assert!(used.contains("commands.registerCommand"));
        assert!(used.contains("window.showInformationMessage"));
        assert!(used.contains("workspace.getConfiguration"));
    }

    #[test]
    fn sample_extension_is_works() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/vsix-sample-extension/extension/main.js");
        let src = std::fs::read_to_string(&root).expect("read sample");
        let report = compatibility_report(&src);
        assert_eq!(report.level, CompatibilityLevel::Works);
    }
}
