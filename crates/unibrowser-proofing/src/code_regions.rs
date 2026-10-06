use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeSegmentKind {
    Comment,
    StringLiteral,
    Identifier,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSegment {
    pub start: u32,
    pub end: u32,
    pub kind: CodeSegmentKind,
}

/// Lightweight lexer for proofing: comments and string literals only (Monaco/tokeniser parity goal).
pub fn code_check_segments(source: &str, language_id: &str) -> Vec<CodeSegment> {
    let mut segments = Vec::new();
    if language_id == "rust" {
        segments.extend(scan_rust_like(source));
    } else {
        segments.extend(scan_c_like(source));
    }
    segments
}

pub fn proofable_ranges(segments: &[CodeSegment]) -> Vec<(u32, u32)> {
    segments
        .iter()
        .filter(|s| {
            s.kind == CodeSegmentKind::Comment || s.kind == CodeSegmentKind::StringLiteral
        })
        .map(|s| (s.start, s.end))
        .collect()
}

fn scan_c_like(source: &str) -> Vec<CodeSegment> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            out.push(CodeSegment {
                start: start as u32,
                end: i as u32,
                kind: CodeSegmentKind::Comment,
            });
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            out.push(CodeSegment {
                start: start as u32,
                end: i as u32,
                kind: CodeSegmentKind::Comment,
            });
            continue;
        }
        if bytes[i] == b'"' {
            let (seg, next) = read_string(source, i, b'"');
            out.push(seg);
            i = next;
            continue;
        }
        if bytes[i] == b'\'' {
            let (seg, next) = read_string(source, i, b'\'');
            out.push(seg);
            i = next;
            continue;
        }
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            out.push(CodeSegment {
                start: start as u32,
                end: i as u32,
                kind: CodeSegmentKind::Identifier,
            });
            continue;
        }
        i += 1;
    }
    out
}

fn scan_rust_like(source: &str) -> Vec<CodeSegment> {
    scan_c_like(source)
}

fn read_string(source: &str, start: usize, quote: u8) -> (CodeSegment, usize) {
    let bytes = source.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            i += 1;
            break;
        }
        i += 1;
    }
    (
        CodeSegment {
            start: start as u32,
            end: i as u32,
            kind: CodeSegmentKind::StringLiteral,
        },
        i,
    )
}
