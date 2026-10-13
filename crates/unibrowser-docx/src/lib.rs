//! Write a **subset** of Mammoth-style HTML back into an existing `.docx`.
//!
//! Tables, images, fields, and other OOXML outside the supported HTML subset are
//! detected on the original file; save refuses rather than dropping them silently.

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use docx_rs::{Docx, Paragraph, Run};
use quick_xml::events::Event;
use quick_xml::Reader;
use scraper::{ElementRef, Html, Node};
use serde::Serialize;
use thiserror::Error;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Error)]
pub enum DocxSaveError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML: {0}")]
    Xml(String),
    #[error("unsupported content in original document: {0}")]
    UnsupportedOriginal(String),
    #[error("unsupported HTML in editor: {0}")]
    UnsupportedHtml(String),
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocxSaveReport {
    pub warnings: Vec<String>,
    pub paragraphs_written: usize,
}

#[derive(Debug, Clone)]
struct InlineRun {
    text: String,
    bold: bool,
    italic: bool,
}

#[derive(Debug, Clone)]
enum Block {
    Paragraph(Vec<InlineRun>),
    Heading(u8, Vec<InlineRun>),
}

const ORIGINAL_UNSUPPORTED: &[&str] = &[
    "tbl",
    "drawing",
    "pict",
    "object",
    "fldSimple",
    "instrText",
    "sdt",
];

const HTML_ALLOWED: &[&str] = &[
    "p", "h1", "h2", "h3", "h4", "h5", "h6", "strong", "b", "em", "i", "u", "br", "span", "a",
    "ul", "ol", "li",
];

/// Scan `word/document.xml` for OOXML this writer cannot preserve.
pub fn scan_original_fidelity(docx_path: &Path) -> Result<Vec<String>, DocxSaveError> {
    let xml = read_zip_entry(docx_path, "word/document.xml")?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut issues = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if ORIGINAL_UNSUPPORTED.contains(&local.as_str()) {
                    issues.push(format!("w:{}", local));
                }
            }
            Ok(_) => {}
            Err(e) => return Err(DocxSaveError::Xml(e.to_string())),
        }
        buf.clear();
    }
    Ok(issues)
}

/// Save using the shared document model (base-docx serialiser path).
pub fn save_docx_from_document_model(
    docx_path: &Path,
    model: &unibrowser_document_model::DocumentModel,
) -> Result<DocxSaveReport, DocxSaveError> {
    let html = unibrowser_document_model::model_to_docx_html(model);
    save_docx_from_html(docx_path, &html)
}

/// Replace document body from HTML produced by the Mammoth preview (supported subset only).
pub fn save_docx_from_html(docx_path: &Path, html: &str) -> Result<DocxSaveReport, DocxSaveError> {
    let mut warnings = Vec::new();
    let original_issues = scan_original_fidelity(docx_path)?;
    if !original_issues.is_empty() {
        warnings.push(format!(
            "Original used unsupported Word features ({}); body will be replaced with the preview subset.",
            original_issues.join(", ")
        ));
    }
    let html_issues = unsupported_html_tags(html);
    if !html_issues.is_empty() {
        warnings.push(format!(
            "Preview HTML contained unsupported tags ({}); those regions may be omitted.",
            html_issues.join(", ")
        ));
    }
    let blocks = parse_html_blocks(html)?;
    let blocks = if blocks.is_empty() {
        warnings.push("Preview had no paragraphs; writing a single empty paragraph.".into());
        vec![Block::Paragraph(Vec::new())]
    } else {
        blocks
    };
    let sect = extract_sect_pr(&read_zip_entry(docx_path, "word/document.xml")?)?;
    let new_document_xml = build_document_xml(&blocks, sect.as_deref())?;
    rewrite_document_xml_in_place(docx_path, &new_document_xml)?;
    warnings.push(
        "Body replaced with supported paragraph/heading markup only; styles and numbering from Word are not round-tripped.".into(),
    );
    Ok(DocxSaveReport {
        warnings,
        paragraphs_written: blocks.len(),
    })
}

fn read_zip_entry(path: &Path, name: &str) -> Result<String, DocxSaveError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut entry = archive.by_name(name)?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf)?;
    Ok(buf)
}

fn unsupported_html_tags(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let mut bad = HashSet::new();
    for el in doc.root_element().descendants() {
        if let Node::Element(ref e) = el.value() {
            let tag = e.name();
            if tag == "html" || tag == "body" || tag == "head" {
                continue;
            }
            if !HTML_ALLOWED.contains(&tag) {
                bad.insert(format!("<{}>", tag));
            }
        }
    }
    bad.into_iter().collect()
}

fn parse_html_blocks(html: &str) -> Result<Vec<Block>, DocxSaveError> {
    let trimmed = html.trim();
    let fragment = !trimmed.to_ascii_lowercase().starts_with("<html")
        && !trimmed.to_ascii_lowercase().starts_with("<body");
    if fragment {
        let frag = Html::parse_fragment(html);
        return blocks_from_root(frag.root_element());
    }
    let doc = Html::parse_document(html);
    let body_sel = scraper::Selector::parse("body").unwrap();
    let article_sel = scraper::Selector::parse("article").unwrap();
    let root = doc
        .select(&body_sel)
        .next()
        .or_else(|| doc.select(&article_sel).next())
        .ok_or_else(|| DocxSaveError::Msg("HTML has no body".into()))?;
    blocks_from_root(root)
}

fn blocks_from_root(root: ElementRef<'_>) -> Result<Vec<Block>, DocxSaveError> {
    let mut blocks = Vec::new();
    for child in root.children().filter(|n| n.value().is_element()) {
        let el = ElementRef::wrap(child).ok_or_else(|| DocxSaveError::Msg("element".into()))?;
        let tag = el.value().name();
        match tag {
            "p" => blocks.push(Block::Paragraph(inline_runs_from(el))),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse().unwrap_or(1);
                blocks.push(Block::Heading(level, inline_runs_from(el)));
            }
            "ul" | "ol" => {
                let li_sel = scraper::Selector::parse("li").unwrap();
                for li in el.select(&li_sel) {
                    blocks.push(Block::Paragraph(inline_runs_from(li)));
                }
            }
            _ => {
                return Err(DocxSaveError::UnsupportedHtml(format!("<{}>", tag)));
            }
        }
    }
    if blocks.is_empty() && root.value().name() != "html" {
        blocks.push(Block::Paragraph(inline_runs_from(root)));
    }
    Ok(blocks)
}

fn inline_runs_from(root: ElementRef<'_>) -> Vec<InlineRun> {
    let mut runs = Vec::new();
    collect_runs(root, false, false, &mut runs);
    if runs.is_empty() {
        runs.push(InlineRun {
            text: String::new(),
            bold: false,
            italic: false,
        });
    }
    runs
}

fn collect_runs(node: ElementRef<'_>, bold: bool, italic: bool, out: &mut Vec<InlineRun>) {
    for child in node.children() {
        match child.value() {
            Node::Text(t) => {
                let text = t.trim();
                if !text.is_empty() {
                    out.push(InlineRun {
                        text: text.to_string(),
                        bold,
                        italic,
                    });
                }
            }
            Node::Element(_) => {
                let el = ElementRef::wrap(child).expect("element");
                let tag = el.value().name();
                let b = bold || tag == "strong" || tag == "b";
                let i = italic || tag == "em" || tag == "i";
                if tag == "br" {
                    out.push(InlineRun {
                        text: "\n".into(),
                        bold,
                        italic,
                    });
                } else {
                    collect_runs(el, b, i, out);
                }
            }
            _ => {}
        }
    }
}

fn build_document_xml(blocks: &[Block], sect_pr: Option<&str>) -> Result<Vec<u8>, DocxSaveError> {
    let mut docx = Docx::new();
    for block in blocks {
        let para = match block {
            Block::Paragraph(runs) => paragraph_from_runs(runs),
            Block::Heading(level, runs) => {
                paragraph_from_runs(runs).style(&format!("Heading{}", level))
            }
        };
        docx = docx.add_paragraph(para);
    }
    let built = docx.build();
    let mut xml =
        String::from_utf8(built.document).map_err(|e| DocxSaveError::Msg(e.to_string()))?;
    if let Some(sect) = sect_pr {
        if !xml.contains("w:sectPr") {
            xml = xml.replace("</w:body>", &format!("{sect}</w:body>"));
        }
    }
    Ok(xml.into_bytes())
}

fn paragraph_from_runs(runs: &[InlineRun]) -> Paragraph {
    let mut p = Paragraph::new();
    for r in runs {
        let mut run = Run::new().add_text(&r.text);
        if r.bold {
            run = run.bold();
        }
        if r.italic {
            run = run.italic();
        }
        p = p.add_run(run);
    }
    p
}

fn rewrite_document_xml_in_place(docx_path: &Path, document_xml: &[u8]) -> Result<(), DocxSaveError> {
    let temp = docx_path.with_extension("docx.ub-save-tmp");
    repack_docx(docx_path, &temp, "word/document.xml", document_xml)?;
    std::fs::rename(&temp, docx_path)?;
    Ok(())
}

fn extract_sect_pr(xml: &str) -> Result<Option<String>, DocxSaveError> {
    let start = xml.rfind("<w:sectPr");
    if start.is_none() {
        return Ok(None);
    }
    let start = start.unwrap();
    let end = xml[start..]
        .find("</w:sectPr>")
        .map(|i| start + i + "</w:sectPr>".len())
        .ok_or_else(|| DocxSaveError::Xml("sectPr close missing".into()))?;
    Ok(Some(xml[start..end].to_string()))
}

fn repack_docx(
    src: &Path,
    dest: &Path,
    replace_name: &str,
    replace_bytes: &[u8],
) -> Result<(), DocxSaveError> {
    let in_file = File::open(src)?;
    let mut archive = ZipArchive::new(in_file)?;
    let out_file = File::create(dest)?;
    let mut writer = ZipWriter::new(out_file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        writer.start_file(name.clone(), opts)?;
        if name == replace_name {
            writer.write_all(replace_bytes)?;
        } else {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            writer.write_all(&buf)?;
        }
    }
    writer.finish()?;
    Ok(())
}

/// Plain text from all `w:t` nodes (tests / verification).
pub fn extract_plain_text(docx_path: &Path) -> Result<String, DocxSaveError> {
    let xml = read_zip_entry(docx_path, "word/document.xml")?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut out = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Text(t)) => {
                out.push_str(&t.unescape().unwrap_or_default());
                out.push(' ');
            }
            Ok(_) => {}
            Err(e) => return Err(DocxSaveError::Xml(e.to_string())),
        }
        buf.clear();
    }
    Ok(out.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn write_minimal_docx(path: &Path, text: &str) {
        let doc = Docx::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)));
        let mut buf = Vec::new();
        doc.build().pack(Cursor::new(&mut buf)).expect("pack");
        // pack writes to buf via Seek
        std::fs::write(path, buf).expect("write");
    }

    #[test]
    fn minimal_pack_writes_bytes() {
        let dir = std::env::temp_dir().join("unibrowser-docx-pack");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tiny.docx");
        write_minimal_docx(&path, "Hi");
        assert!(path.metadata().unwrap().len() > 100);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn roundtrip_simple_paragraph() {
        let dir = std::env::temp_dir().join("unibrowser-docx-rt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.docx");
        write_minimal_docx(&path, "Hello world");
        let html = "<p>Hello <strong>edited</strong></p>";
        let report = save_docx_from_html(&path, html).expect("save");
        assert_eq!(report.paragraphs_written, 1);
        let text = extract_plain_text(&path).expect("read");
        assert!(text.contains("Hello"));
        assert!(text.contains("edited"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sacred_save_despite_original_table() {
        let dir = std::env::temp_dir().join("unibrowser-docx-tbl");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tbl.docx");
        write_minimal_docx(&path, "before");
        let xml = read_zip_entry(&path, "word/document.xml").unwrap();
        let injected = xml.replace(
            "</w:body>",
            "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body>",
        );
        repack_docx(&path, &path.with_extension("tmp.docx"), "word/document.xml", injected.as_bytes())
            .unwrap();
        std::fs::rename(path.with_extension("tmp.docx"), &path).unwrap();
        let report = save_docx_from_html(&path, "<p>ok</p>").expect("save must not refuse");
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("unsupported Word features")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
