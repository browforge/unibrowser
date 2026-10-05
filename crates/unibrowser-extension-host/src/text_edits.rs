//! Apply VS Code-style text edits to a UTF-8 document (line + character = char index within line).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDto {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeDto {
    pub start: PositionDto,
    pub end: PositionDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditDto {
    pub range: RangeDto,
    #[serde(alias = "newText")]
    pub new_text: String,
}

fn line_start(lines: &[&str], line: u32) -> Option<usize> {
    if line as usize > lines.len() {
        return None;
    }
    let mut offset = 0usize;
    for (i, line_str) in lines.iter().enumerate() {
        if i == line as usize {
            return Some(offset);
        }
        offset += line_str.len() + 1;
    }
    if line as usize == lines.len() {
        return Some(offset);
    }
    None
}

fn position_to_offset(text: &str, pos: &PositionDto) -> Option<usize> {
    let lines: Vec<&str> = text.split('\n').collect();
    let base = line_start(&lines, pos.line)?;
    let line_str = if (pos.line as usize) < lines.len() {
        lines[pos.line as usize]
    } else if pos.line as usize == lines.len() {
        ""
    } else {
        return None;
    };
    let char_idx = pos.character.min(line_str.chars().count() as u32) as usize;
    let within = line_str.chars().take(char_idx).map(|c| c.len_utf8()).sum::<usize>();
    Some(base + within)
}

pub fn apply_text_edits(mut text: String, edits: &[TextEditDto]) -> Result<String, String> {
    let mut indexed: Vec<(usize, usize, String)> = Vec::new();
    for edit in edits {
        let start = position_to_offset(&text, &edit.range.start)
            .ok_or_else(|| "edit start out of range".to_string())?;
        let end = position_to_offset(&text, &edit.range.end)
            .ok_or_else(|| "edit end out of range".to_string())?;
        if end < start {
            return Err("invalid edit range".into());
        }
        indexed.push((start, end, edit.new_text.clone()));
    }
    indexed.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end, new_text) in indexed {
        if end > text.len() || start > end {
            return Err("edit out of bounds".into());
        }
        text.replace_range(start..end, &new_text);
    }
    Ok(text)
}
