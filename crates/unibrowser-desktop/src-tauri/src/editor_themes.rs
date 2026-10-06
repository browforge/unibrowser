//! Editor (Monaco) theme listing and VSIX theme resolution.

use std::collections::HashMap;

use serde::Serialize;
use unibrowser_extension_host::MonacoThemeDto;

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
        rules: vec![],
        colors: HashMap::from([
            ("editor.background".into(), "#FFFFFF".into()),
            ("editor.foreground".into(), "#1F2326".into()),
            ("editorLineNumber.foreground".into(), "#62686A".into()),
            ("editor.selectionBackground".into(), "#DCE9F2".into()),
            ("editor.inactiveSelectionBackground".into(), "#E7E5DF".into()),
            ("editorCursor.foreground".into(), "#1F2326".into()),
        ]),
    }
}

fn quiet_utility_dark() -> MonacoThemeDto {
    MonacoThemeDto {
        id: "vs-dark".into(),
        base: "vs-dark".into(),
        inherit: true,
        rules: vec![],
        colors: HashMap::from([
            ("editor.background".into(), "#232526".into()),
            ("editor.foreground".into(), "#F0EFEB".into()),
            ("editorLineNumber.foreground".into(), "#AEB2AF".into()),
            ("editor.selectionBackground".into(), "#253E4C".into()),
            ("editor.inactiveSelectionBackground".into(), "#2D3031".into()),
            ("editorCursor.foreground".into(), "#F0EFEB".into()),
        ]),
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
