use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModSuggestionDto {
    pub mod_id: String,
    pub title: String,
    pub reason: String,
}

/// Static index until native `.ubmod` registry exists ([ADR 0014]).
pub fn suggest_mods_for_extension(ext: &str) -> Vec<ModSuggestionDto> {
    let ext = ext.to_ascii_lowercase();
    let suggestions = match ext.as_str() {
        "docx" | "doc" => vec![("base-docx", "Open Word documents")],
        "pdf" => vec![("base-pdf", "View PDF (vector + raster)")],
        "dxf" => vec![("base-dxf", "View CAD drawings")],
        "mp4" | "webm" | "mkv" => vec![("base-video", "Play and edit video")],
        "wav" | "mp3" | "flac" => vec![("base-audio", "Play and edit audio")],
        "mid" | "midi" => vec![("base-midi", "MIDI clips and preview")],
        "js" | "mjs" => vec![("base-js", "Run user scripts safely")],
        "psd" => vec![("base-psd", "Photoshop documents (community)")],
        "" => vec![],
        _ => vec![],
    };
    suggestions
        .into_iter()
        .map(|(id, title)| ModSuggestionDto {
            mod_id: id.into(),
            title: title.into(),
            reason: format!("No bundled handler for .{}", ext),
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
        }];
    }
    list
}
