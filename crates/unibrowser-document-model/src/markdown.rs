use crate::model::{
    digest_bytes, Block, DocumentMetadata, DocumentModel, InlineMark, InlineSpan, ListItem,
    MarkKind, OpaqueBlock, SourceMap, SourceMapEntry, SCHEMA_ID, SCHEMA_VERSION,
};
use std::collections::HashSet;

pub struct MarkdownParseResult {
    pub model: DocumentModel,
    pub source_map: SourceMap,
}

pub fn parse_markdown(bytes: &[u8]) -> MarkdownParseResult {
    let text = String::from_utf8_lossy(bytes);
    let spans = split_byte_spans(bytes);
    let mut blocks = Vec::new();
    let mut entries = Vec::new();
    let mut next_id = 1usize;
    for (start, end) in spans {
        let slice = &text[start..end];
        let trimmed = slice.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }
        let id = format!("b{}", next_id);
        next_id += 1;
        let block = classify_span(trimmed, &id);
        blocks.push(block);
        push_map(&mut entries, &id, bytes, start, end);
    }
    if blocks.is_empty() {
        let id: String = "b1".into();
        blocks.push(Block::Paragraph {
            id: id.clone(),
            inlines: vec![InlineSpan {
                text: String::new(),
                marks: vec![],
            }],
        });
        push_map(&mut entries, "b1", bytes, 0, bytes.len());
    }
    let title = blocks
        .iter()
        .find_map(|b| {
            if let Block::Heading { level: 1, inlines, .. } = b {
                Some(inline_plain(inlines))
            } else {
                None
            }
        });
    let model = DocumentModel {
        schema_id: SCHEMA_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        metadata: DocumentMetadata {
            title,
            front_matter: None,
            host_format: "markdown".into(),
            serializer_plugin_id: "base-text".into(),
        },
        blocks,
    };
    MarkdownParseResult {
        model,
        source_map: SourceMap { entries },
    }
}

pub struct SerializeOptions {
    pub dirty_blocks: HashSet<String>,
    pub touched_blocks: HashSet<String>,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            dirty_blocks: HashSet::new(),
            touched_blocks: HashSet::new(),
        }
    }
}

pub fn serialize_markdown(
    original: &[u8],
    model: &DocumentModel,
    source_map: &SourceMap,
    options: &SerializeOptions,
) -> Vec<u8> {
    let mut out = Vec::new();
    let map_by_id: std::collections::HashMap<&str, &SourceMapEntry> = source_map
        .entries
        .iter()
        .map(|e| (e.block_id.as_str(), e))
        .collect();
    for block in &model.blocks {
        let id = block_id_of(block);
        let verbatim = map_by_id.get(id.as_str()).filter(|_entry| {
            !options.dirty_blocks.contains(&id) && !options.touched_blocks.contains(&id)
        });
        if let Some(entry) = verbatim {
            let end = entry.byte_end.min(bytes_len(original));
            out.extend_from_slice(&original[entry.byte_start..end]);
        } else {
            write_canonical_block(&mut out, block);
        }
    }
    out
}

fn bytes_len(original: &[u8]) -> usize {
    original.len()
}

fn split_byte_spans(bytes: &[u8]) -> Vec<(usize, usize)> {
    if bytes.is_empty() {
        return vec![(0, 0)];
    }
    let text = String::from_utf8_lossy(bytes);
    let mut spans = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut in_fence = false;
    while i < text.len() {
        if text[i..].starts_with("```") {
            in_fence = !in_fence;
            i += 3;
            continue;
        }
        if !in_fence && text[i..].starts_with("\n\n") {
            let end = i + 2;
            spans.push((start, end));
            start = end;
            i = end;
            continue;
        }
        if !in_fence && text[i..].starts_with("\r\n\r\n") {
            let end = i + 4;
            spans.push((start, end));
            start = end;
            i = end;
            continue;
        }
        i += 1;
    }
    if start < text.len() {
        spans.push((start, text.len()));
    }
    spans
}

fn classify_span(slice: &str, id: &str) -> Block {
    if slice.trim() == "---" {
        return Block::HorizontalRule { id: id.to_string() };
    }
    if slice.starts_with("```") {
        let first_line_end = slice.find('\n').unwrap_or(slice.len());
        let opener = slice[..first_line_end].trim_end();
        let fence = opener.chars().take(3).collect::<String>();
        let language = opener[fence.len()..].trim();
        let lang = if language.is_empty() {
            None
        } else {
            Some(language.to_string())
        };
        let close = slice.rfind(&fence).unwrap_or(slice.len());
        let body = if close > first_line_end {
            slice[first_line_end + 1..close].trim_end_matches(['\r', '\n'])
        } else {
            ""
        };
        return Block::CodeBlock {
            id: id.to_string(),
            language: lang,
            text: body.to_string(),
            fence: Some(fence),
        };
    }
    if slice.starts_with('<') && !slice.starts_with("<!--") {
        return Block::Opaque(OpaqueBlock {
            id: id.to_string(),
            raw: slice.to_string(),
            language: Some("html".into()),
            label: Some("Raw Markdown (preserved)".into()),
        });
    }
    if let Some(level) = heading_level_line(slice) {
        let body = slice.trim_start_matches('#').trim_start();
        return Block::Heading {
            id: id.to_string(),
            level,
            inlines: parse_inline(body),
        };
    }
    if slice.starts_with('>') {
        let body = slice
            .lines()
            .map(|l| l.trim_start_matches('>').trim_start())
            .collect::<Vec<_>>()
            .join(" ");
        return Block::Blockquote {
            id: id.to_string(),
            inlines: parse_inline(&body),
        };
    }
    if is_list_start_str(slice) {
        return parse_list_span(slice, id);
    }
    if let Some(img) = parse_image_line(slice.lines().next().unwrap_or(slice)) {
        return Block::Image {
            id: id.to_string(),
            src: img.0,
            alt: img.1,
            title: img.2,
        };
    }
    Block::Paragraph {
        id: id.to_string(),
        inlines: parse_inline(slice),
    }
}

fn heading_level_line(line: &str) -> Option<u8> {
    let first = line.lines().next().unwrap_or(line);
    heading_level_at(first, 0)
}

fn is_list_start_str(slice: &str) -> bool {
    let line = slice.lines().next().unwrap_or(slice);
    line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("+ ")
        || ordered_prefix(line).is_some()
}

fn parse_list_span(slice: &str, id: &str) -> Block {
    let first = slice.lines().next().unwrap_or(slice);
    let ordered = ordered_prefix(first);
    let mut items = Vec::new();
    let mut item_index = 1usize;
    for line in slice.lines() {
        let (prefix_len, checked) = if ordered.is_some() {
            (line.find('.').map(|i| i + 2).unwrap_or(0), None)
        } else if line.starts_with("- [ ] ") {
            (6, Some(false))
        } else if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            (6, Some(true))
        } else if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
            (2, None)
        } else {
            continue;
        };
        let body = line[prefix_len.min(line.len())..].trim();
        items.push(ListItem {
            id: format!("{}-{}", id, item_index),
            checked,
            inlines: parse_inline(body),
        });
        item_index += 1;
    }
    if ordered.is_some() {
        Block::OrderedList {
            id: id.to_string(),
            start: ordered.unwrap_or(1),
            items,
        }
    } else {
        Block::BulletList {
            id: id.to_string(),
            items,
        }
    }
}

fn write_canonical_block(out: &mut Vec<u8>, block: &Block) {
    match block {
        Block::Paragraph { inlines, .. } => {
            write_inlines(out, inlines);
        }
        Block::Heading { level, inlines, .. } => {
            for _ in 0..*level {
                out.push(b'#');
            }
            out.push(b' ');
            write_inlines(out, inlines);
        }
        Block::Blockquote { inlines, .. } => {
            out.extend_from_slice(b"> ");
            write_inlines(out, inlines);
        }
        Block::CodeBlock {
            text, language, fence, ..
        } => {
            let fence = fence.as_deref().unwrap_or("```");
            out.extend_from_slice(fence.as_bytes());
            if let Some(lang) = language {
                out.push(b' ');
                out.extend_from_slice(lang.as_bytes());
            }
            out.extend_from_slice(b"\n");
            out.extend_from_slice(text.as_bytes());
            if !text.ends_with('\n') {
                out.push(b'\n');
            }
            out.extend_from_slice(fence.as_bytes());
        }
        Block::BulletList { items, .. } => {
            for item in items {
                out.extend_from_slice(b"- ");
                write_inlines(out, &item.inlines);
                out.push(b'\n');
            }
            if out.last() == Some(&b'\n') {
                out.pop();
            }
        }
        Block::OrderedList { start, items, .. } => {
            let mut n = *start;
            for item in items {
                out.extend_from_slice(format!("{}. ", n).as_bytes());
                write_inlines(out, &item.inlines);
                out.push(b'\n');
                n += 1;
            }
            if out.last() == Some(&b'\n') {
                out.pop();
            }
        }
        Block::HorizontalRule { .. } => {
            out.extend_from_slice(b"---");
        }
        Block::Image { alt, src, title, .. } => {
            out.push(b'!');
            out.push(b'[');
            out.extend_from_slice(alt.as_bytes());
            out.push(b']');
            if let Some(t) = title {
                out.push(b' ');
                out.push(b'"');
                out.extend_from_slice(t.as_bytes());
                out.push(b'"');
            }
            out.push(b'(');
            out.extend_from_slice(src.as_bytes());
            out.push(b')');
        }
        Block::Opaque(o) => {
            out.extend_from_slice(o.raw.as_bytes());
        }
        Block::Table { .. } => {
            out.extend_from_slice(b"<!-- table omitted -->");
        }
    }
}

fn write_inlines(out: &mut Vec<u8>, inlines: &[InlineSpan]) {
    for span in inlines {
        let mut prefix = String::new();
        let mut suffix = String::new();
        for mark in &span.marks {
            match mark.kind {
                MarkKind::Bold => {
                    prefix.push_str("**");
                    suffix = format!("**{}", suffix);
                }
                MarkKind::Italic => {
                    prefix.push('*');
                    suffix = format!("*{}", suffix);
                }
                MarkKind::Strikethrough => {
                    prefix.push_str("~~");
                    suffix = format!("~~{}", suffix);
                }
                MarkKind::Code => {
                    prefix.push('`');
                    suffix = format!("`{}", suffix);
                }
                MarkKind::Link => {
                    let href = mark.href.as_deref().unwrap_or("");
                    prefix.push('[');
                    suffix = format!("]({}){}", href, suffix);
                }
                MarkKind::Underline => {
                    prefix.push_str("<u>");
                    suffix = format!("</u>{}", suffix);
                }
                MarkKind::Subscript | MarkKind::Superscript => {}
            }
        }
        out.extend_from_slice(prefix.as_bytes());
        out.extend_from_slice(span.text.as_bytes());
        out.extend_from_slice(suffix.as_bytes());
    }
}

fn push_map(entries: &mut Vec<SourceMapEntry>, id: &str, bytes: &[u8], start: usize, end: usize) {
    let slice = &bytes[start..end.min(bytes.len())];
    entries.push(SourceMapEntry {
        block_id: id.to_string(),
        byte_start: start,
        byte_end: end,
        digest: digest_bytes(slice),
    });
}

fn block_id_of(block: &Block) -> String {
    match block {
        Block::Paragraph { id, .. } => id.clone(),
        Block::Heading { id, .. } => id.clone(),
        Block::Blockquote { id, .. } => id.clone(),
        Block::CodeBlock { id, .. } => id.clone(),
        Block::BulletList { id, .. } => id.clone(),
        Block::OrderedList { id, .. } => id.clone(),
        Block::HorizontalRule { id } => id.clone(),
        Block::Image { id, .. } => id.clone(),
        Block::Opaque(o) => o.id.clone(),
        Block::Table { id, .. } => id.clone(),
    }
}

fn find_line_end(text: &str, offset: usize) -> usize {
    match text[offset..].find('\n') {
        Some(i) => offset + i + 1,
        None => text.len(),
    }
}

fn skip_blank_lines(text: &str, offset: usize) -> usize {
    let mut o = offset;
    while o < text.len() {
        if text[o..].starts_with("\r\n") {
            o += 2;
            continue;
        }
        if text[o..].starts_with('\n') {
            o += 1;
            continue;
        }
        break;
    }
    o
}

fn find_paragraph_end(text: &str, offset: usize) -> usize {
    let mut o = offset;
    while o < text.len() {
        if text[o..].starts_with("\n\n") || text[o..].starts_with("\r\n\r\n") {
            return o;
        }
        if let Some(i) = text[o..].find('\n') {
            let next = o + i + 1;
            if next < text.len() && text.as_bytes()[next] == b'\n' {
                return o + i;
            }
            o = next;
        } else {
            return text.len();
        }
    }
    text.len()
}

fn heading_level_at(text: &str, offset: usize) -> Option<u8> {
    let line = text[offset..].lines().next()?;
    let mut hashes = 0u8;
    for ch in line.chars() {
        if ch == '#' {
            hashes += 1;
        } else {
            break;
        }
    }
    if hashes == 0 || hashes > 6 {
        return None;
    }
    if line.len() > hashes as usize && line.as_bytes()[hashes as usize] != b' ' {
        return None;
    }
    Some(hashes)
}

fn find_blockquote_end(text: &str, offset: usize) -> usize {
    let mut o = offset;
    while o < text.len() {
        let line_end = find_line_end(text, o);
        let line = text[o..line_end].trim_end_matches(['\r', '\n']);
        if !line.starts_with('>') && !line.is_empty() {
            return o;
        }
        o = line_end;
        if o < text.len() && text[o..].starts_with("\n\n") {
            break;
        }
    }
    o
}

fn is_list_start(text: &str, offset: usize) -> bool {
    let line = text[offset..].lines().next().unwrap_or("");
    line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") || ordered_prefix(line).is_some()
}

fn ordered_prefix(line: &str) -> Option<u32> {
    let mut digits = String::new();
    for ch in line.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if ch == '.' && !digits.is_empty() {
            return digits.parse().ok();
        } else {
            break;
        }
    }
    None
}

fn parse_list_block(
    text: &str,
    _start: usize,
    offset: usize,
    next_id: &mut usize,
) -> (Block, usize) {
    let mut o = offset;
    let first = text[offset..].lines().next().unwrap_or("");
    let ordered = ordered_prefix(first);
    let mut items = Vec::new();
    while o < text.len() {
        let line_end = find_line_end(text, o);
        let line = text[o..line_end].trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        let (prefix_len, checked) = if let Some(_) = ordered {
            (first.find('.').map(|i| i + 2).unwrap_or(0), None)
        } else if line.starts_with("- [ ] ") {
            (6, Some(false))
        } else if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            (6, Some(true))
        } else if line.starts_with("- ") {
            (2, None)
        } else if line.starts_with("* ") || line.starts_with("+ ") {
            (2, None)
        } else {
            break;
        };
        let body = line[prefix_len.min(line.len())..].trim();
        let id = format!("b{}", *next_id);
        *next_id += 1;
        items.push(ListItem {
            id,
            checked,
            inlines: parse_inline(body),
        });
        o = line_end;
    }
    let id = format!("b{}", *next_id);
    *next_id += 1;
    let block = if let Some(start) = ordered {
        Block::OrderedList {
            id,
            start,
            items,
        }
    } else {
        Block::BulletList { id, items }
    };
    (block, o)
}

fn parse_fenced_code(text: &str, start: usize, next_id: &mut usize) -> (Block, usize) {
    let first_line_end = find_line_end(text, start);
    let opener = text[start..first_line_end].trim_end_matches(['\r', '\n']);
    let fence = opener.chars().take(3).collect::<String>();
    let language = opener[fence.len()..].trim();
    let lang = if language.is_empty() {
        None
    } else {
        Some(language.to_string())
    };
    let mut o = first_line_end;
    let body_start = o;
    while o < text.len() {
        let line_end = find_line_end(text, o);
        let line = text[o..line_end].trim_end_matches(['\r', '\n']);
        if line.starts_with(&fence) && line.trim() == fence {
            let body = text[body_start..o].trim_end_matches(['\r', '\n']).to_string();
            let id = format!("b{}", *next_id);
            *next_id += 1;
            return (
                Block::CodeBlock {
                    id,
                    language: lang,
                    text: body,
                    fence: Some(fence),
                },
                line_end,
            );
        }
        o = line_end;
    }
    let id = format!("b{}", *next_id);
    *next_id += 1;
    (
        Block::CodeBlock {
            id,
            language: lang,
            text: text[start..].to_string(),
            fence: Some(fence),
        },
        text.len(),
    )
}

fn is_html_opaque_line(text: &str, offset: usize) -> bool {
    let line = text[offset..].lines().next().unwrap_or("");
    line.starts_with('<') && !line.starts_with("<!--")
}

fn find_opaque_end(text: &str, offset: usize) -> usize {
    let mut o = offset;
    while o < text.len() {
        let line_end = find_line_end(text, o);
        let line = text[o..line_end].trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            return line_end;
        }
        if !line.starts_with('<') && heading_level_at(text, o).is_none() {
            return o;
        }
        o = line_end;
    }
    text.len()
}

fn is_image_line(text: &str, offset: usize) -> bool {
    text[offset..].trim_start().starts_with("![")
}

fn parse_image_line(line: &str) -> Option<(String, String, Option<String>)> {
    let line = line.trim();
    if !line.starts_with("![") {
        return None;
    }
    let after = &line[2..];
    let alt_end = after.find(']')?;
    let alt = after[..alt_end].to_string();
    let rest = &after[alt_end + 1..];
    let (title, rest) = if rest.starts_with(" \"") {
        let end = rest.find('"')?;
        (Some(rest[2..end].to_string()), &rest[end + 1..])
    } else {
        (None, rest)
    };
    if !rest.starts_with('(') || !rest.ends_with(')') {
        return None;
    }
    let src = rest[1..rest.len() - 1].to_string();
    Some((src, alt, title))
}

pub fn parse_inline(text: &str) -> Vec<InlineSpan> {
    if text.is_empty() {
        return vec![InlineSpan {
            text: String::new(),
            marks: vec![],
        }];
    }
    vec![InlineSpan {
        text: text.to_string(),
        marks: parse_marks_simple(text),
    }]
}

fn parse_marks_simple(_text: &str) -> Vec<InlineMark> {
    vec![]
}

fn inline_plain(inlines: &[InlineSpan]) -> String {
    inlines.iter().map(|s| s.text.as_str()).collect::<String>()
}
