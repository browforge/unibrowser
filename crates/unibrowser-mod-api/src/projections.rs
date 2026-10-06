//! Text and raster projections for any document type (ADR 0025).

/// UTF-8 (or lossy) text surface for search, AI, a11y.
pub trait TextProjection {
    fn text_projection(&self) -> String;
}

/// RGBA raster for vision models / thumbnails (optional per backend).
#[derive(Debug, Clone)]
pub struct RasterProjection {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub trait RasterProjectionSource {
    fn raster_projection(&self) -> Option<RasterProjection>;
}
