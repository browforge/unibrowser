use unibrowser_mod_api::capability_surface::ids;

/// Host routing from sniffed / file extension to a surface capability id.
pub fn capability_for_effective_extension(effective_ext: &str) -> Option<&'static str> {
    match effective_ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" => Some(ids::RENDER_IMAGE),
        "svg" => Some(ids::RENDER_SVG),
        "docx" => Some(ids::OPEN_DOCX),
        "pdf" => Some(ids::RENDER_PDF),
        "dxf" => Some(ids::OPEN_DXF),
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" => Some(ids::OPEN_SCRIPT),
        "rhai" => Some(ids::OPEN_TEXT),
        "csv" | "tsv" => Some(ids::OPEN_CSV),
        "txt" | "md" | "json" | "rs" | "toml" | "html" | "css" => Some(ids::OPEN_TEXT),
        "mp4" | "webm" | "avi" => Some(ids::OPEN_VIDEO),
        "wav" | "mp3" | "ogg" | "flac" => Some(ids::OPEN_AUDIO),
        "mid" | "midi" => None,
        "epub" => Some(ids::OPEN_EPUB),
        "zip" | "7z" | "rar" | "tar" | "gz" => None,
        _ => None,
    }
}
