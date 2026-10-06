//! Editor (Monaco) theme listing and VSIX theme resolution.

use std::collections::HashMap;

use serde::Serialize;
use unibrowser_extension_host::{MonacoThemeDto, MonacoTokenRule};

#[derive(Serialize)]
pub struct EditorThemeSummaryDto {
    pub id: String,
    pub label: String,
    pub source: String,
}

pub fn list_editor_themes() -> Result<Vec<EditorThemeSummaryDto>, String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    let list = unibrowser_vsix::list_editor_themes(&root).map_err(|e| e.to_string())?;
    Ok(list
        .into_iter()
        .map(|t| EditorThemeSummaryDto {
            id: t.id,
            label: t.label,
            source: t.source,
        })
        .collect())
}

fn quiet_utility_light() -> MonacoThemeDto {
    MonacoThemeDto {
        id: "vs".into(),
        base: "vs".into(),
        inherit: true,
        rules: vec![
            token_rule("comment", "#62686A", Some("italic")),
            token_rule("keyword", "#2D647D", None),
            token_rule("string", "#2F6B48", None),
            token_rule("number", "#8A5A12", None),
            token_rule("type.identifier", "#6B4F9A", None),
            token_rule("delimiter", "#4B5052", None),
        ],
        colors: HashMap::from([
            ("editor.background".into(), "#FFFFFF".into()),
            ("editor.foreground".into(), "#1F2326".into()),
            ("editorLineNumber.foreground".into(), "#62686A".into()),
            ("editor.selectionBackground".into(), "#DCE9F2".into()),
            (
                "editor.inactiveSelectionBackground".into(),
                "#E7E5DF".into(),
            ),
            ("editorCursor.foreground".into(), "#1F2326".into()),
        ]),
    }
}

fn quiet_utility_dark() -> MonacoThemeDto {
    MonacoThemeDto {
        id: "vs-dark".into(),
        base: "vs-dark".into(),
        inherit: true,
        rules: vec![
            token_rule("comment", "#AEB2AF", Some("italic")),
            token_rule("keyword", "#79B1C9", None),
            token_rule("string", "#74BE91", None),
            token_rule("number", "#E1B466", None),
            token_rule("type.identifier", "#C7A7E8", None),
            token_rule("delimiter", "#C9CBC8", None),
        ],
        colors: HashMap::from([
            ("editor.background".into(), "#232526".into()),
            ("editor.foreground".into(), "#F0EFEB".into()),
            ("editorLineNumber.foreground".into(), "#AEB2AF".into()),
            ("editor.selectionBackground".into(), "#253E4C".into()),
            (
                "editor.inactiveSelectionBackground".into(),
                "#2D3031".into(),
            ),
            ("editorCursor.foreground".into(), "#F0EFEB".into()),
        ]),
    }
}

fn token_rule(token: &str, foreground: &str, font_style: Option<&str>) -> MonacoTokenRule {
    MonacoTokenRule {
        token: token.into(),
        foreground: Some(foreground.into()),
        background: None,
        font_style: font_style.map(Into::into),
    }
}

pub fn get_editor_monaco_theme(theme_id: String) -> Result<Option<MonacoThemeDto>, String> {
    if theme_id == "vs" {
        return Ok(Some(quiet_utility_light()));
    }
    if theme_id == "vs-dark" {
        return Ok(Some(quiet_utility_dark()));
    }
    if theme_id == "auto" {
        return Ok(None);
    }
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    unibrowser_vsix::resolve_monaco_theme(&root, &theme_id).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_hex_colour(value: &str) -> bool {
        value.len() == 7
            && value.starts_with('#')
            && value[1..].chars().all(|c| c.is_ascii_hexdigit())
    }

    #[test]
    fn built_in_themes_use_valid_bases_and_hex_colours() {
        for theme in [quiet_utility_light(), quiet_utility_dark()] {
            assert!(theme.base == "vs" || theme.base == "vs-dark");
            assert!(theme.colors.values().all(|value| is_hex_colour(value)));
            assert!(theme
                .rules
                .iter()
                .filter_map(|rule| rule.foreground.as_deref())
                .all(is_hex_colour));
        }
    }
}
