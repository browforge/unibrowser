use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Locator {
    TextRange {
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
        start_byte: u32,
        end_byte: u32,
    },
    DxfEntity {
        handle: String,
        layer: String,
    },
    TimelineRange {
        track: String,
        t0_ms: u64,
        t1_ms: u64,
    },
    ImageRegion {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    PdfRegion {
        page: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    DomNode {
        selector: String,
    },
}

impl Locator {
    pub fn text_bytes(start: u32, end: u32) -> Self {
        Self::TextRange {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
            start_byte: start,
            end_byte: end,
        }
    }
}
