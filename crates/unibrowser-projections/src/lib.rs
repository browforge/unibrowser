//! Document text/raster projections for AI, search, OCR, and accessibility.

mod dxf_text;
mod redact;
mod unibrow;

use std::path::Path;

use unibrowser_engine::document::DocumentSession;
use unibrowser_mod_api::{RasterProjection, TextProjection};

pub use redact::redact_sensitive_text;

/// Shell-provided text that Rust backends do not own (webview DOM, OCR, PDF.js page text).
#[derive(Debug, Clone, Default)]
pub struct ProjectionExtras {
    pub web_dom_text: Option<String>,
    pub ocr_text: Option<String>,
    pub epub_chapter_text: Option<String>,
    pub pdf_page_text: Option<String>,
    pub docx_preview_text: Option<String>,
    pub unibrow_manifest: Option<unibrowser_project::ProjectManifest>,
}

#[derive(Debug, Clone)]
pub struct ProjectionBundle {
    pub text: String,
    pub raster: Option<RasterProjection>,
}

pub fn project_document(
    doc: &mut DocumentSession,
    extras: &ProjectionExtras,
) -> ProjectionBundle {
    let text = project_text(doc, extras);
    let raster = project_raster(doc);
    ProjectionBundle { text, raster }
}

pub fn project_text(doc: &mut DocumentSession, extras: &ProjectionExtras) -> String {
    let backend = doc.backend.backend_id().to_string();
    let any = doc.backend.as_any_mut();
    let raw = match backend.as_str() {
        "base-text" => {
            if let Some(b) = any.downcast_ref::<base_text::TextBackend>() {
                b.content.clone()
            } else {
                String::new()
            }
        }
        "base-js" => {
            if let Some(b) = any.downcast_ref::<base_js::JsMonacoBackend>() {
                b.content.clone()
            } else {
                String::new()
            }
        }
        "toy-markdown" => {
            if let Some(b) = any.downcast_ref::<toy_markdown::MarkdownBackend>() {
                b.source.clone()
            } else {
                String::new()
            }
        }
        "base-web" => extras
            .web_dom_text
            .clone()
            .unwrap_or_else(|| {
                if let Some(b) = any.downcast_ref::<base_web::WebBackend>() {
                    format!("Web page: {}", b.url)
                } else {
                    String::new()
                }
            }),
        "base-docx" => extras
            .docx_preview_text
            .clone()
            .or_else(|| {
                if let Some(b) = any.downcast_ref::<base_docx::DocxBackend>() {
                    let path = Path::new(&b.path);
                    unibrowser_docx::extract_plain_text(path).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default(),
        "base-epub" => extras
            .epub_chapter_text
            .clone()
            .unwrap_or_else(|| {
                if let Some(b) = any.downcast_ref::<base_epub::EpubBackend>() {
                    format!("EPUB: {}", b.path)
                } else {
                    String::new()
                }
            }),
        "base-pdf" => extras
            .pdf_page_text
            .clone()
            .unwrap_or_else(|| {
                if let Some(b) = any.downcast_ref::<base_pdf::PdfBackend>() {
                    format!(
                        "PDF: {} (page {}, {} bytes)",
                        b.path, b.page_index, b.byte_len
                    )
                } else {
                    String::new()
                }
            }),
        "base-svg" => {
            if let Some(b) = any.downcast_ref::<base_svg::SvgBackend>() {
                strip_svg_to_text(&b.content)
            } else {
                String::new()
            }
        }
        "base-png" => {
            if let Some(b) = any.downcast_ref::<base_png::PngBackend>() {
                let mut parts = Vec::new();
                if let Some(layer) = &b.text_layer {
                    parts.push(layer.label.clone());
                }
                if let Some(ocr) = &extras.ocr_text {
                    parts.push(ocr.clone());
                }
                if parts.is_empty() {
                    format!("Image {}x{}", b.width, b.height)
                } else {
                    parts.join("\n")
                }
            } else if let Some(ocr) = &extras.ocr_text {
                ocr.clone()
            } else {
                String::new()
            }
        }
        "base-dxf" => {
            if let Some(b) = any.downcast_ref::<base_dxf::DxfBackend>() {
                let data = std::fs::read_to_string(&b.path).unwrap_or_default();
                let (text, layers) = dxf_text::dxf_text_and_layers(&data);
                if layers.is_empty() {
                    text
                } else {
                    format!("Layers: {}\n{}", layers.join(", "), text)
                }
            } else {
                String::new()
            }
        }
        "base-csv" => {
            if let Some(b) = any.downcast_ref::<base_csv::CsvBackend>() {
                std::fs::read_to_string(&b.path).unwrap_or_default()
            } else {
                String::new()
            }
        }
        "base-hex" => {
            if let Some(b) = any.downcast_ref::<base_hex::HexBackend>() {
                b.content.clone()
            } else {
                String::new()
            }
        }
        "unibrow-project" => extras
            .unibrow_manifest
            .as_ref()
            .map(unibrow::timeline_summary)
            .unwrap_or_else(|| doc.backend.status_line()),
        _ => doc.backend.status_line(),
    };
    redact_sensitive_text(&raw)
}

pub fn project_unibrow_manifest(manifest: &unibrowser_project::ProjectManifest) -> String {
    redact_sensitive_text(&unibrow::timeline_summary(manifest))
}

pub fn project_raster(doc: &mut DocumentSession) -> Option<RasterProjection> {
    let path = Path::new(&doc.path);
    let backend = doc.backend.backend_id().to_string();
    match backend.as_str() {
        "base-png" | "base-pdf" => load_image_raster(path),
        "base-svg" => {
            let any = doc.backend.as_any_mut();
            if let Some(b) = any.downcast_ref::<base_svg::SvgBackend>() {
                let p = Path::new(&b.path);
                let rgba = unibrowser_media::rasterize_svg_file_to_rgba(p, b.width.max(1), b.height.max(1))
                    .ok()?;
                Some(RasterProjection {
                    width: b.width.max(1),
                    height: b.height.max(1),
                    rgba,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn load_image_raster(path: &Path) -> Option<RasterProjection> {
    let img = image::ImageReader::open(path).ok()?.decode().ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(RasterProjection {
        width: w,
        height: h,
        rgba: rgba.into_raw(),
    })
}

fn strip_svg_to_text(svg: &str) -> String {
    let re = regex::Regex::new(r"<text[^>]*>([^<]*)</text>").unwrap();
    let parts: Vec<String> = re
        .captures_iter(svg)
        .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        svg.chars().take(4000).collect()
    } else {
        parts.join("\n")
    }
}

impl TextProjection for ProjectionBundle {
    fn text_projection(&self) -> String {
        self.text.clone()
    }
}
