//! Map diagnostics [`Locator`] values to shell navigation targets (ADR 0021).

use serde::{Deserialize, Serialize};

use crate::locator::Locator;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NavigationTarget {
    TextLineCol {
        line: u32,
        col: u32,
        end_line: u32,
        end_col: u32,
    },
    TextByteRange {
        start_byte: u32,
        end_byte: u32,
    },
    DocxParagraph {
        paragraph_index: u32,
    },
    SvgTextOffset {
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
    PdfRegion {
        page: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    ImageRegion {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    DomNode {
        selector: String,
    },
}

pub fn navigation_target(locator: &Locator) -> NavigationTarget {
    match locator {
        Locator::TextRange {
            start_line,
            start_col,
            end_line,
            end_col,
            start_byte,
            end_byte,
        } => {
            if *start_line > 0 || *start_col > 0 || *end_line > 0 || *end_col > 0 {
                NavigationTarget::TextLineCol {
                    line: *start_line,
                    col: *start_col,
                    end_line: *end_line,
                    end_col: *end_col,
                }
            } else {
                NavigationTarget::TextByteRange {
                    start_byte: *start_byte,
                    end_byte: *end_byte,
                }
            }
        }
        Locator::DxfEntity { handle, layer } => NavigationTarget::DxfEntity {
            handle: handle.clone(),
            layer: layer.clone(),
        },
        Locator::TimelineRange { track, t0_ms, t1_ms } => NavigationTarget::TimelineRange {
            track: track.clone(),
            t0_ms: *t0_ms,
            t1_ms: *t1_ms,
        },
        Locator::ImageRegion { x, y, w, h } => NavigationTarget::ImageRegion {
            x: *x,
            y: *y,
            w: *w,
            h: *h,
        },
        Locator::PdfRegion { page, x, y, w, h } => NavigationTarget::PdfRegion {
            page: *page,
            x: *x,
            y: *y,
            w: *w,
            h: *h,
        },
        Locator::DomNode { selector } => NavigationTarget::DomNode {
            selector: selector.clone(),
        },
    }
}

/// DOCX preview: map a text line number to a paragraph index (0-based).
pub fn docx_paragraph_from_line(line: u32) -> NavigationTarget {
    NavigationTarget::DocxParagraph {
        paragraph_index: line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_text_line_col() {
        let loc = Locator::TextRange {
            start_line: 2,
            start_col: 1,
            end_line: 2,
            end_col: 5,
            start_byte: 0,
            end_byte: 0,
        };
        let t = navigation_target(&loc);
        assert_eq!(
            t,
            NavigationTarget::TextLineCol {
                line: 2,
                col: 1,
                end_line: 2,
                end_col: 5,
            }
        );
    }

    #[test]
    fn dispatch_timeline() {
        let loc = Locator::TimelineRange {
            track: "video".into(),
            t0_ms: 100,
            t1_ms: 200,
        };
        let t = navigation_target(&loc);
        assert!(matches!(
            t,
            NavigationTarget::TimelineRange {
                track,
                t0_ms: 100,
                t1_ms: 200,
            } if track == "video"
        ));
    }

    #[test]
    fn dispatch_dxf_pdf_dom() {
        assert!(matches!(
            navigation_target(&Locator::DxfEntity {
                handle: "A1".into(),
                layer: "0".into(),
            }),
            NavigationTarget::DxfEntity { .. }
        ));
        assert!(matches!(
            navigation_target(&Locator::PdfRegion {
                page: 3,
                x: 0.1,
                y: 0.2,
                w: 0.3,
                h: 0.4,
            }),
            NavigationTarget::PdfRegion { page: 3, .. }
        ));
        assert!(matches!(
            navigation_target(&Locator::DomNode {
                selector: "#email".into(),
            }),
            NavigationTarget::DomNode { .. }
        ));
    }
}
