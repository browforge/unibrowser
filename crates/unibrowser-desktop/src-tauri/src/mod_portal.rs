use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModSuggestionDto {
    pub mod_id: String,
    pub title: String,
    pub reason: String,
    /// Optional Open VSX extension id for in-portal search (stub catalog).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_vsx_id: Option<String>,
    /// Optional local `.ubmod` package id when a community bundle exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ubmod_id: Option<String>,
}

/// Static index until native `.ubmod` registry exists ([ADR 0014]).
pub fn suggest_mods_for_extension(ext: &str) -> Vec<ModSuggestionDto> {
    let ext = ext.to_ascii_lowercase();
    let suggestions: Vec<(String, String, Option<String>, Option<String>)> = match ext.as_str() {
        "docx" | "doc" => vec![(
            "base-docx".into(),
            "Open Word documents".into(),
            None,
            Some("unibrowser.base-docx".into()),
        )],
        "pdf" => vec![(
            "base-pdf".into(),
            "View PDF (vector + raster)".into(),
            None,
            Some("unibrowser.base-pdf".into()),
        )],
        "dxf" => vec![(
            "base-dxf".into(),
            "View CAD drawings".into(),
            None,
            Some("unibrowser.base-dxf".into()),
        )],
        "mp4" | "webm" | "mkv" => vec![(
            "base-video".into(),
            "Play and edit video".into(),
            None,
            Some("unibrowser.base-video".into()),
        )],
        "wav" | "mp3" | "flac" => vec![(
            "base-audio".into(),
            "Play and edit audio".into(),
            None,
            Some("unibrowser.base-audio".into()),
        )],
        "mid" | "midi" => vec![(
            "base-midi".into(),
            "MIDI clips and preview".into(),
            None,
            Some("unibrowser.base-midi".into()),
        )],
        "js" | "mjs" => vec![(
            "base-js".into(),
            "Run user scripts safely".into(),
            Some("dbaeumer.vscode-eslint".into()),
            None,
        )],
        "psd" => vec![(
            "base-psd".into(),
            "Photoshop documents (community)".into(),
            Some("ms-vscode.vscode-typescript-next".into()),
            Some("community.psd-reader".into()),
        )],
        "xlsx" | "xls" => vec![(
            "base-spreadsheet".into(),
            "Spreadsheets".into(),
            Some("GrapeCity.gc-excelviewer".into()),
            Some("unibrowser.base-csv".into()),
        )],
        "" => vec![],
        _ => vec![],
    };
    suggestions
        .into_iter()
        .map(|(id, title, open_vsx_id, ubmod_id)| ModSuggestionDto {
            mod_id: id,
            title,
            reason: format!("No bundled handler for .{}", ext),
            open_vsx_id,
            ubmod_id,
        })
        .collect()
}

pub fn suggest_mods_for_unknown_path(path: &str) -> Vec<ModSuggestionDto> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let list = suggest_mods_for_extension(ext);
    if list.is_empty() && !ext.is_empty() {
        return vec![ModSuggestionDto {
            mod_id: "mod-portal".into(),
            title: "Browse installable mods".into(),
            reason: format!("'.{}' is not supported yet", ext),
            open_vsx_id: None,
            ubmod_id: Some(format!("community.handler-{}", ext)),
        }];
    }
    if list.is_empty() {
        return vec![ModSuggestionDto {
            mod_id: "mod-portal".into(),
            title: "Browse installable mods".into(),
            reason: "Pick a format handler from the catalog".into(),
            open_vsx_id: None,
            ubmod_id: None,
        }];
    }
    list
}

#[derive(Debug, Clone, Serialize)]
pub struct ModCatalogEntryDto {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub blurb: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_vsx_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ubmod_id: Option<String>,
}

fn catalog_seed() -> Vec<ModCatalogEntryDto> {
    vec![
        ModCatalogEntryDto {
            id: "base-docx".into(),
            title: "Word (DOCX)".into(),
            kind: "ubmod".into(),
            blurb: "Bundled Mammoth preview lane".into(),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-docx".into()),
        },
        ModCatalogEntryDto {
            id: "base-pdf".into(),
            title: "PDF viewer".into(),
            kind: "ubmod".into(),
            blurb: "Raster + vector proving".into(),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-pdf".into()),
        },
        ModCatalogEntryDto {
            id: "eslint".into(),
            title: "ESLint (Open VSX)".into(),
            kind: "open_vsx".into(),
            blurb: "Example VSIX stub install".into(),
            open_vsx_id: Some("dbaeumer.vscode-eslint".into()),
            ubmod_id: None,
        },
        ModCatalogEntryDto {
            id: "excel-viewer".into(),
            title: "Excel viewer (Open VSX)".into(),
            kind: "open_vsx".into(),
            blurb: "Spreadsheet preview extension".into(),
            open_vsx_id: Some("GrapeCity.gc-excelviewer".into()),
            ubmod_id: None,
        },
        ModCatalogEntryDto {
            id: "hello-ubmod".into(),
            title: "Hello world .ubmod".into(),
            kind: "ubmod".into(),
            blurb: "Sample native mod package".into(),
            open_vsx_id: None,
            ubmod_id: Some("mod.hello".into()),
        },
        ModCatalogEntryDto {
            id: "base-dxf".into(),
            title: "DXF proving".into(),
            kind: "ubmod".into(),
            blurb: "Wireframe CAD stub".into(),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-dxf".into()),
        },
    ]
}

pub fn filter_mod_catalog(query: &str) -> Vec<ModCatalogEntryDto> {
    let q = query.trim().to_ascii_lowercase();
    let all = catalog_seed();
    if q.is_empty() {
        return all;
    }
    all.into_iter()
        .filter(|e| {
            e.id.to_ascii_lowercase().contains(&q)
                || e.title.to_ascii_lowercase().contains(&q)
                || e.blurb.to_ascii_lowercase().contains(&q)
                || e.open_vsx_id
                    .as_ref()
                    .map(|s| s.to_ascii_lowercase().contains(&q))
                    .unwrap_or(false)
                || e.ubmod_id
                    .as_ref()
                    .map(|s| s.to_ascii_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_extension_suggests_portal() {
        let list = suggest_mods_for_unknown_path("file.xyz");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].mod_id, "mod-portal");
    }

    #[test]
    fn pdf_suggests_ubmod() {
        let list = suggest_mods_for_extension("pdf");
        assert_eq!(list[0].mod_id, "base-pdf");
        assert!(list[0].ubmod_id.is_some());
    }

    #[test]
    fn catalog_filter_matches_ubmod_id() {
        let hits = filter_mod_catalog("hello");
        assert!(hits.iter().any(|e| e.id == "hello-ubmod"));
    }
}
