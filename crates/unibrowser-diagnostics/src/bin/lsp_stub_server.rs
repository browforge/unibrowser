//! Minimal LSP server for diagnostics client tests (stdio JSON-RPC).

use std::io::{BufRead, BufReader, Read, Write};

fn main() {
    let stdin = BufReader::new(std::io::stdin());
    let mut stdout = std::io::stdout();
    let mut lines = stdin;
    let mut buf = String::new();
    let mut opened_text = String::new();
    while read_lsp_body(&mut lines, &mut buf) {
        let msg: serde_json::Value = serde_json::from_str(&buf).unwrap_or(serde_json::json!({}));
        if let Some(id) = msg.get("id").and_then(|v| v.as_i64()) {
            let method = msg.get("method").and_then(|m| m.as_str());
            let result = if method == Some("shutdown") {
                serde_json::json!(null)
            } else if method == Some("textDocument/hover") {
                serde_json::json!({
                    "contents": { "kind": "plaintext", "value": format!("stub hover: {}", opened_text.chars().take(40).collect::<String>()) }
                })
            } else if method == Some("textDocument/codeAction") {
                serde_json::json!([{
                    "title": "Stub quick fix",
                    "kind": "quickfix",
                    "edit": { "changes": {} }
                }])
            } else {
                serde_json::json!({ "capabilities": {} })
            };
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            });
            write_message(&mut stdout, &resp.to_string());
        }
        if msg.get("method").and_then(|m| m.as_str()) == Some("textDocument/didOpen") {
            opened_text = msg
                .pointer("/params/textDocument/text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
        }
        if msg.get("method").and_then(|m| m.as_str()) == Some("textDocument/didChange") {
            opened_text = msg
                .pointer("/params/contentChanges/0/text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let uri = msg
                .pointer("/params/textDocument/uri")
                .and_then(|t| t.as_str())
                .unwrap_or("file:///test.txt");
            let note = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": uri,
                    "diagnostics": [{
                        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 4 } },
                        "message": "stub: typo",
                        "severity": 2
                    }]
                }
            });
            write_message(&mut stdout, &note.to_string());
        }
        buf.clear();
    }
}

fn read_lsp_body(r: &mut BufReader<impl Read>, buf: &mut String) -> bool {
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).unwrap_or(0) == 0 {
            return false;
        }
        if line == "\r\n" {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            len = rest.trim().parse().unwrap_or(0);
        }
    }
    buf.clear();
    let mut bytes = vec![0u8; len];
    if r.read_exact(&mut bytes).is_err() {
        return false;
    }
    *buf = String::from_utf8(bytes).unwrap_or_default();
    true
}

fn write_message(w: &mut impl Write, body: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let _ = w.write_all(header.as_bytes());
    let _ = w.write_all(body.as_bytes());
    let _ = w.flush();
}
