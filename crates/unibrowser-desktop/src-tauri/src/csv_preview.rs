use std::fs;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CsvPreviewDto {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub truncated: bool,
    pub delimiter: char,
}

pub fn preview_file(path: &Path, max_rows: usize) -> Result<CsvPreviewDto, String> {
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let delimiter = if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("tsv"))
        .unwrap_or(false)
    {
        '\t'
    } else {
        ','
    };
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut truncated = false;
    for (i, line) in data.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cells = parse_line(line, delimiter);
        if i == 0 && headers.is_empty() {
            headers = cells;
            continue;
        }
        if rows.len() >= max_rows {
            truncated = true;
            break;
        }
        rows.push(cells);
    }
    if headers.is_empty() && !rows.is_empty() {
        let n = rows[0].len();
        headers = (1..=n).map(|i| format!("Column {}", i)).collect();
    }
    Ok(CsvPreviewDto {
        headers,
        rows,
        truncated,
        delimiter,
    })
}

fn parse_line(line: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                cur.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == delimiter {
            out.push(cur);
            cur = String::new();
        } else {
            cur.push(ch);
        }
    }
    out.push(cur);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_quoted_csv() {
        let path = std::env::temp_dir().join("unibrowser-csv-test.csv");
        let mut f = std::fs::File::create(&path).expect("create");
        writeln!(f, "a,b").unwrap();
        writeln!(f, "1,\"2,3\"").unwrap();
        let preview = preview_file(&path, 10).expect("preview");
        assert_eq!(preview.headers, vec!["a", "b"]);
        assert_eq!(preview.rows[0], vec!["1", "2,3"]);
        let _ = std::fs::remove_file(path);
    }
}
