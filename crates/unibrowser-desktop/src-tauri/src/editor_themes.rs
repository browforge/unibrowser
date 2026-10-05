//! Editor (Monaco) theme listing and VSIX theme resolution.

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

pub fn get_editor_monaco_theme(theme_id: String) -> Result<Option<MonacoThemeDto>, String> {
    let root = crate::config::vsix_extensions_dir().ok_or("config dir unavailable")?;
    unibrowser_vsix::resolve_monaco_theme(&root, &theme_id).map_err(|e| e.to_string())
}
