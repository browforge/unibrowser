//! VSIX-contributed color themes → Monaco definitions.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use unibrowser_extension_host::MonacoThemeDto;

use crate::{InstalledVsixRecord, VsixError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsixThemeEntry {
    pub theme_id: String,
    pub label: String,
    pub extension_id: String,
    /// Path relative to `extension/` inside the install dir.
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EditorThemeSummary {
    pub id: String,
    pub label: String,
    pub source: String,
}

pub fn theme_id_for(extension_id: &str, label: &str) -> String {
    let slug = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let slug = if slug.is_empty() { "theme".into() } else { slug };
    format!("vsix:{}:{}", extension_id, slug)
}

pub fn discover_themes_on_install(
    extension_id: &str,
    install_dir: &Path,
    theme_contribs: &[(Option<String>, Option<String>)],
) -> (Vec<VsixThemeEntry>, Vec<String>) {
    let mut themes = Vec::new();
    let mut notes = Vec::new();
    for t in theme_contribs {
        let label = t.0.clone().unwrap_or_else(|| "Theme".into());
        let rel = t.1.clone().unwrap_or_default();
        if rel.is_empty() {
            notes.push(format!("{label} — recognised, not applied (no path)"));
            continue;
        }
        let path = install_dir.join("extension").join(&rel);
        let theme_id = theme_id_for(extension_id, &label);
        match unibrowser_extension_host::from_vscode_theme_file(&theme_id, &path) {
            Ok(_) => {
                notes.push(format!(
                    "{label} — editor theme (Settings → Appearance → Editor theme)"
                ));
                themes.push(VsixThemeEntry {
                    theme_id,
                    label,
                    extension_id: extension_id.to_string(),
                    path: rel,
                });
            }
            Err(e) => {
                notes.push(format!("{label} ({rel}) — recognised, not applied ({e})"));
            }
        }
    }
    (themes, notes)
}

pub fn list_editor_themes(root: &Path) -> Result<Vec<EditorThemeSummary>, VsixError> {
    let mut out = vec![
        EditorThemeSummary {
            id: "auto".into(),
            label: "Match app theme".into(),
            source: "builtin".into(),
        },
        EditorThemeSummary {
            id: "vs".into(),
            label: "Monaco light".into(),
            source: "builtin".into(),
        },
        EditorThemeSummary {
            id: "vs-dark".into(),
            label: "Monaco dark".into(),
            source: "builtin".into(),
        },
    ];
    for rec in crate::list_installed(root)? {
        if !rec.enabled {
            continue;
        }
        for t in &rec.themes {
            out.push(EditorThemeSummary {
                id: t.theme_id.clone(),
                label: format!("{} ({})", t.label, rec.extension_id),
                source: rec.extension_id.clone(),
            });
        }
    }
    Ok(out)
}

pub fn resolve_monaco_theme(root: &Path, theme_id: &str) -> Result<Option<MonacoThemeDto>, VsixError> {
    if theme_id == "auto" || theme_id == "vs" || theme_id == "vs-dark" {
        return Ok(None);
    }
    if !theme_id.starts_with("vsix:") {
        return Err(VsixError::Msg(format!("unknown editor theme: {}", theme_id)));
    }
    for rec in crate::list_installed(root)? {
        for t in &rec.themes {
            if t.theme_id != theme_id {
                continue;
            }
            let path = PathBuf::from(&rec.install_dir).join("extension").join(&t.path);
            let dto = unibrowser_extension_host::from_vscode_theme_file(&theme_id, &path)
                .map_err(|e| VsixError::Msg(e))?;
            return Ok(Some(dto));
        }
    }
    Err(VsixError::Msg(format!("editor theme not found: {}", theme_id)))
}
