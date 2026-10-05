use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Serialize;

const MAX_PREVIEW_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct CsvPreviewDto {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub truncated: bool,
    pub delimiter: char,
    pub total_data_rows: usize,
    pub rows_returned: usize,
    pub file_bytes: u64,
    pub read_capped: bool,
}

pub fn preview_file(path: &Path, max_rows: usize) -> Result<CsvPreviewDto, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let file_bytes = meta.len();
    let read_capped = file_bytes > MAX_PREVIEW_BYTES;
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
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut truncated = false;
    let mut total_data_rows = 0usize;
    let mut line_index = 0usize;
    let max_rows = max_rows.min(200);
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let cells = parse_line(&line, delimiter);
        if line_index == 0 {
            headers = cells;
            line_index += 1;
            continue;
        }
        total_data_rows += 1;
        if rows.len() < max_rows {
            rows.push(cells);
        } else {
            truncated = true;
        }
        line_index += 1;
        if read_capped && line_index > 50_000 {
            truncated = true;
            break;
        }
    }
    if headers.is_empty() && !rows.is_empty() {
        let n = rows[0].len();
        headers = (1..=n).map(|i| format!("Column {}", i)).collect();
    }
    let rows_returned = rows.len();
    Ok(CsvPreviewDto {
        headers,
        rows,
        truncated,
        delimiter,
        total_data_rows,
        rows_returned,
        file_bytes,
        read_capped,
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
        assert_eq!(preview.total_data_rows, 1);
        let _ = std::fs::remove_file(path);
    }
}
