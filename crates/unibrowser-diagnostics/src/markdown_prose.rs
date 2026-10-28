/// Byte ranges of markdown prose suitable for spell/grammar (skip fences and syntax-only spans).

pub fn markdown_prose_ranges(source: &str) -> Vec<(u32, u32)> {
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    let mut i = 0usize;
    let bytes = source.as_bytes();
    while i < bytes.len() {
        if source[i..].starts_with("```") {
            let fence_start = i;
            i += 3;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            while i < bytes.len() {
                if source[i..].starts_with("```") {
                    i += 3;
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                    break;
                }
                i += 1;
            }
            let _ = fence_start;
            continue;
        }
        let line_start = i;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        let line_end = i;
        if i < bytes.len() {
            i += 1;
        }
        let line = &source[line_start..line_end];
        if let Some((content_start, content_end)) = prose_within_line(line) {
            let start = line_start + content_start;
            let end = line_start + content_end;
            if end > start {
                ranges.push((start as u32, end as u32));
            }
        }
    }
    merge_ranges(ranges)
}

fn prose_within_line(line: &str) -> Option<(usize, usize)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("```") {
        return None;
    }
    let mut start = 0usize;
    let bytes = line.as_bytes();
    while start < bytes.len() && bytes[start] == b' ' {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    if bytes[start] == b'#' {
        while start < bytes.len() && bytes[start] == b'#' {
            start += 1;
        }
        while start < bytes.len() && bytes[start] == b' ' {
            start += 1;
        }
    } else if bytes[start] == b'>' {
        start += 1;
        while start < bytes.len() && bytes[start] == b' ' {
            start += 1;
        }
    } else if bytes[start] == b'-' || bytes[start] == b'*' || bytes[start] == b'+' {
        if start + 1 < bytes.len() && bytes[start + 1] == b' ' {
            start += 2;
        }
    } else if bytes[start].is_ascii_digit() {
        let mut j = start;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'.' && j + 1 < bytes.len() && bytes[j + 1] == b' ' {
            start = j + 2;
        }
    }
    if start >= line.len() {
        return None;
    }
    let mut end = line.len();
    while end > start && line.as_bytes()[end - 1] == b' ' {
        end -= 1;
    }
    if end <= start {
        return None;
    }
    Some((start, end))
}

fn merge_ranges(mut ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort_by_key(|r| r.0);
    let mut out = vec![ranges[0]];
    for (s, e) in ranges.into_iter().skip(1) {
        let last = out.last_mut().unwrap();
        if s <= last.1 {
            last.1 = last.1.max(e);
        } else {
            out.push((s, e));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_fenced_code_and_checks_heading_prose() {
        let md = "# Title\n\nPara word\n\n```rust\nfn main() {}\n```";
        let ranges = markdown_prose_ranges(md);
        assert!(ranges.iter().any(|(s, e)| &md[*s as usize..*e as usize] == "Title"));
        assert!(ranges.iter().any(|(s, e)| &md[*s as usize..*e as usize] == "Para word"));
        assert!(!ranges.iter().any(|(s, e)| md[*s as usize..*e as usize].contains("fn main")));
    }
}
