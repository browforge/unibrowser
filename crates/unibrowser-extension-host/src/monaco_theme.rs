//! Convert VS Code color theme JSON into a Monaco `IStandaloneThemeData`-compatible shape.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonacoThemeDto {
    pub id: String,
    pub base: String,
    pub inherit: bool,
    pub rules: Vec<MonacoTokenRule>,
    pub colors: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonacoTokenRule {
    pub token: String,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VsCodeThemeFile {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type", default)]
    theme_type: Option<String>,
    #[serde(default)]
    colors: HashMap<String, String>,
    #[serde(default, rename = "tokenColors")]
    token_colors: Vec<TokenColorEntry>,
}

#[derive(Debug, Deserialize)]
struct TokenColorEntry {
    #[serde(default)]
    scope: Option<serde_json::Value>,
    #[serde(default)]
    settings: TokenSettings,
}

fn scopes_from_value(scope: &Option<serde_json::Value>) -> Vec<String> {
    match scope {
        None => Vec::new(),
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

#[derive(Debug, Default, Deserialize)]
struct TokenSettings {
    #[serde(default)]
    foreground: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(rename = "fontStyle", default)]
    font_style: Option<String>,
}

fn normalise_hex(color: &str) -> Option<String> {
    let c = color.trim();
    if c.is_empty() {
        return None;
    }
    if c.starts_with('#') {
        return Some(c.to_string());
    }
    if c.len() == 8 {
        return Some(format!("#{}", c));
    }
    None
}

fn monaco_base(theme_type: &str) -> &'static str {
    match theme_type {
        "light" => "vs",
        "hcLight" => "hc-light",
        "hcDark" => "hc-black",
        _ => "vs-dark",
    }
}

pub fn from_vscode_theme_json(id: &str, data: &str) -> Result<MonacoThemeDto, String> {
    let parsed: VsCodeThemeFile = serde_json::from_str(data).map_err(|e| e.to_string())?;
    let base = monaco_base(parsed.theme_type.as_deref().unwrap_or("dark"));
    let mut colors = HashMap::new();
    for (k, v) in parsed.colors {
        if let Some(hex) = normalise_hex(&v) {
            colors.insert(k, hex);
        }
    }
    let mut rules = Vec::new();
    for entry in parsed.token_colors {
        let scopes = scopes_from_value(&entry.scope);
        if scopes.is_empty() {
            continue;
        }
        for scope in scopes {
            rules.push(MonacoTokenRule {
                token: scope,
                foreground: entry.settings.foreground.as_ref().and_then(|c| normalise_hex(c)),
                background: entry.settings.background.as_ref().and_then(|c| normalise_hex(c)),
                font_style: entry.settings.font_style.clone(),
            });
        }
    }
    if rules.is_empty() {
        rules.push(MonacoTokenRule {
            token: String::new(),
            foreground: None,
            background: None,
            font_style: None,
        });
    }
    Ok(MonacoThemeDto {
        id: id.to_string(),
        base: base.to_string(),
        inherit: true,
        rules,
        colors,
    })
}

pub fn from_vscode_theme_file(id: &str, path: &Path) -> Result<MonacoThemeDto, String> {
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    from_vscode_theme_json(id, &data)
}

#[cfg(test)]
mod tests {
    use super::from_vscode_theme_json;

    #[test]
    fn converts_minimal_dark_theme() {
        let json = r##"{
          "name": "Test",
          "type": "dark",
          "colors": { "editor.background": "#1e1e1e" },
          "tokenColors": [
            { "scope": "keyword", "settings": { "foreground": "#569CD6" } }
          ]
        }"##;
        let theme = from_vscode_theme_json("test-theme", json).expect("convert");
        assert_eq!(theme.base, "vs-dark");
        assert_eq!(theme.colors.get("editor.background").map(String::as_str), Some("#1e1e1e"));
        assert_eq!(theme.rules[0].token, "keyword");
    }
}
