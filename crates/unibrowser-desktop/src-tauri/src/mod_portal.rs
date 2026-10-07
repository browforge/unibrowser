use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModSuggestionDto {
    pub mod_id: String,
    pub title: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_summary: Option<String>,
    /// Optional Open VSX extension id for in-portal search (stub catalog).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_vsx_id: Option<String>,
    /// Optional local `.ubmod` package id when a community bundle exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ubmod_id: Option<String>,
}

struct SuggestionSeed {
    mod_id: String,
    title: String,
    reason: String,
    licence: Option<String>,
    permission_summary: Option<String>,
    open_vsx_id: Option<String>,
    ubmod_id: Option<String>,
}

fn seed_to_dto(seed: SuggestionSeed) -> ModSuggestionDto {
    ModSuggestionDto {
        mod_id: seed.mod_id,
        title: seed.title,
        reason: seed.reason,
        licence: seed.licence,
        permission_summary: seed.permission_summary,
        open_vsx_id: seed.open_vsx_id,
        ubmod_id: seed.ubmod_id,
    }
}

fn cad_file_support(ubmod_id: &str) -> SuggestionSeed {
    SuggestionSeed {
        mod_id: "cad-support".into(),
        title: "CAD file support".into(),
        reason: "Open, measure, and preview exchange CAD files in Browse.".into(),
        licence: Some("Apache-2.0".into()),
        permission_summary: Some("Reads local files only".into()),
        open_vsx_id: None,
        ubmod_id: Some(ubmod_id.into()),
    }
}

/// Static index until native `.ubmod` registry exists ([ADR 0014]).
pub fn suggest_mods_for_extension(ext: &str) -> Vec<ModSuggestionDto> {
    let ext = ext.to_ascii_lowercase();
    let seeds: Vec<SuggestionSeed> = match ext.as_str() {
        "docx" | "doc" => vec![SuggestionSeed {
            mod_id: "base-docx".into(),
            title: "Open Word documents".into(),
            reason: "Preview DOCX with layout and styles preserved.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-docx".into()),
        }],
        "pdf" => vec![SuggestionSeed {
            mod_id: "base-pdf".into(),
            title: "View PDF documents".into(),
            reason: "Vector and raster PDF viewing with annotations.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-pdf".into()),
        }],
        "dxf" => vec![cad_file_support("unibrowser.base-dxf")],
        "step" | "stp" | "3mf" | "iges" | "igs" => vec![cad_file_support("unibrowser.base-cad")],
        "mp4" | "webm" | "mkv" => vec![SuggestionSeed {
            mod_id: "base-video".into(),
            title: "Play and edit video".into(),
            reason: "Timeline playback and export for common containers.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-video".into()),
        }],
        "wav" | "mp3" | "flac" => vec![SuggestionSeed {
            mod_id: "base-audio".into(),
            title: "Play and edit audio".into(),
            reason: "Waveform preview and clip editing.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-audio".into()),
        }],
        "mid" | "midi" => vec![SuggestionSeed {
            mod_id: "base-midi".into(),
            title: "MIDI clips and preview".into(),
            reason: "Piano-roll preview for MIDI programme lanes.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: None,
            ubmod_id: Some("unibrowser.base-midi".into()),
        }],
        "js" | "mjs" | "cjs" | "jsx" => vec![SuggestionSeed {
            mod_id: "base-js".into(),
            title: "Run user scripts safely".into(),
            reason: "Lint and run JavaScript with sandboxed tooling.".into(),
            licence: Some("MIT".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: Some("dbaeumer.vscode-eslint".into()),
            ubmod_id: None,
        }],
        "ts" | "tsx" => vec![SuggestionSeed {
            mod_id: "base-js".into(),
            title: "Script and TypeScript tooling".into(),
            reason: "Typecheck and prove TypeScript in the editor.".into(),
            licence: Some("MIT".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: Some("ms-vscode.vscode-typescript-next".into()),
            ubmod_id: None,
        }],
        "psd" => vec![SuggestionSeed {
            mod_id: "base-psd".into(),
            title: "Photoshop documents".into(),
            reason: "Community reader for layered PSD previews.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: Some("ms-vscode.vscode-typescript-next".into()),
            ubmod_id: Some("community.psd-reader".into()),
        }],
        "xlsx" | "xls" => vec![SuggestionSeed {
            mod_id: "base-spreadsheet".into(),
            title: "Spreadsheets".into(),
            reason: "Preview workbook sheets and cell values.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: Some("GrapeCity.gc-excelviewer".into()),
            ubmod_id: Some("unibrowser.base-csv".into()),
        }],
        "" => vec![],
        _ => vec![],
    };
    seeds.into_iter().map(seed_to_dto).collect()
}

pub fn suggest_mods_for_unknown_path(path: &str) -> Vec<ModSuggestionDto> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let list = suggest_mods_for_extension(ext);
    if list.is_empty() && !ext.is_empty() {
        return vec![seed_to_dto(SuggestionSeed {
            mod_id: "format-support".into(),
            title: "Format support".into(),
            reason: "Install a handler to open this file type in Browse.".into(),
            licence: Some("Apache-2.0".into()),
            permission_summary: Some("Reads local files only".into()),
            open_vsx_id: None,
            ubmod_id: Some(format!("community.handler-{}", ext)),
        })];
    }
    if list.is_empty() {
        return vec![seed_to_dto(SuggestionSeed {
            mod_id: "mod-portal".into(),
            title: "Browse installable mods".into(),
            reason: "Pick a format handler from the catalogue.".into(),
            licence: None,
            permission_summary: None,
            open_vsx_id: None,
            ubmod_id: None,
        })];
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
        assert_eq!(list[0].title, "Format support");
    }

    #[test]
    fn step_suggests_cad_support() {
        let list = suggest_mods_for_extension("step");
        assert_eq!(list[0].title, "CAD file support");
        assert_eq!(list[0].licence.as_deref(), Some("Apache-2.0"));
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
