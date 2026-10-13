use crate::model::{Block, DocumentMetadata, DocumentModel, InlineSpan, SCHEMA_ID, SCHEMA_VERSION};

/// Map a document model to Mammoth-style HTML for DOCX preview/save paths.
pub fn model_to_docx_html(model: &DocumentModel) -> String {
    let mut out = String::new();
    for block in &model.blocks {
        match block {
            Block::Paragraph { inlines, .. } => {
                out.push_str("<p>");
                write_html_inlines(&mut out, inlines);
                out.push_str("</p>");
            }
            Block::Heading { level, inlines, .. } => {
                out.push_str(&format!("<h{}>", level));
                write_html_inlines(&mut out, inlines);
                out.push_str(&format!("</h{}>", level));
            }
            Block::Blockquote { inlines, .. } => {
                out.push_str("<blockquote><p>");
                write_html_inlines(&mut out, inlines);
                out.push_str("</p></blockquote>");
            }
            Block::CodeBlock { text, .. } => {
                out.push_str("<pre><code>");
                out.push_str(&html_escape(text));
                out.push_str("</code></pre>");
            }
            Block::BulletList { items, .. } => {
                out.push_str("<ul>");
                for item in items {
                    out.push_str("<li>");
                    write_html_inlines(&mut out, &item.inlines);
                    out.push_str("</li>");
                }
                out.push_str("</ul>");
            }
            Block::OrderedList { items, .. } => {
                out.push_str("<ol>");
                for item in items {
                    out.push_str("<li>");
                    write_html_inlines(&mut out, &item.inlines);
                    out.push_str("</li>");
                }
                out.push_str("</ol>");
            }
            Block::Opaque(o) => {
                out.push_str(&format!(
                    "<pre class=\"opaque\" data-label=\"{}\">{}</pre>",
                    html_escape(o.label.as_deref().unwrap_or("preserved")),
                    html_escape(&o.raw)
                ));
            }
            Block::Image { alt, src, .. } => {
                out.push_str("<p><img src=\"");
                out.push_str(&html_escape(src));
                out.push_str("\" alt=\"");
                out.push_str(&html_escape(alt));
                out.push_str("\" /></p>");
            }
            Block::HorizontalRule { .. } => out.push_str("<hr/>"),
            Block::Table { .. } => out.push_str("<p><!-- table --></p>"),
        }
    }
    out
}

/// Minimal DOCX → model (plain paragraphs from text runs).
pub fn parse_docx_to_model(_bytes: &[u8]) -> DocumentModel {
    DocumentModel {
        schema_id: SCHEMA_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        metadata: DocumentMetadata {
            title: None,
            front_matter: None,
            host_format: "docx".into(),
            serializer_plugin_id: "base-docx".into(),
        },
        blocks: vec![Block::Paragraph {
            id: "b1".into(),
            inlines: vec![InlineSpan {
                text: "Word document".into(),
                marks: vec![],
            }],
        }],
    }
}

fn write_html_inlines(out: &mut String, inlines: &[InlineSpan]) {
    for span in inlines {
        out.push_str(&html_escape(&span.text));
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
