//! PDF multi-engine proving stub ([ADR 0011](../../docs/adr/0011-lazy-conversion-native-until-encode.md)).

use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PdfEngineLineDto {
    pub engine: String,
    pub status: String,
    pub detail: String,
}

pub fn pdf_engine_lines(path: &Path) -> Vec<PdfEngineLineDto> {
    let raster_status = match crate::pdf_preview::pdf_first_page_preview(path) {
        Ok(p) if p.is_file() => ("proxy", "derived/ or temp JPEG page-1 proxy on disk"),
        Ok(_) => ("lazy", "proxy path missing — UI rasterises on demand"),
        Err(_) => ("lazy", "pdftoppm/ffmpeg absent — pdf.js raster in shell"),
    };
    vec![
        PdfEngineLineDto {
            engine: "document".into(),
            status: "active".into(),
            detail: "pdf.js canvas + text layer; font commands when text selected".into(),
        },
        PdfEngineLineDto {
            engine: "raster".into(),
            status: raster_status.0.into(),
            detail: raster_status.1.into(),
        },
        PdfEngineLineDto {
            engine: "vector".into(),
            status: "planned".into(),
            detail: "path/text graph at encode time; not extracted on open".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_lines_have_three_slots() {
        let lines = pdf_engine_lines(Path::new("/nonexistent/file.pdf"));
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].engine, "document");
        assert_eq!(lines[2].engine, "vector");
    }
}
