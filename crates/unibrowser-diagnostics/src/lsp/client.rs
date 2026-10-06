use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde::Deserialize;
use serde_json::Value;

use crate::locator::Locator;
use crate::problem::{Problem, Severity};

#[derive(Debug, Clone)]
pub struct LspServerSpec {
    pub program: String,
    pub args: Vec<String>,
    pub language_id: String,
}

pub struct LspClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl LspClient {
    pub fn spawn_scrubbed(spec: &LspServerSpec) -> Result<Self, String> {
        let mut cmd = Command::new(&spec.program);
        cmd.args(&spec.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        cmd.env_clear();
        cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
        let mut child = cmd.spawn().map_err(|e| format!("spawn LSP: {}", e))?;
        let stdin = child.stdin.take().ok_or("LSP stdin")?;
        let stdout = child.stdout.take().ok_or("LSP stdout")?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    pub fn initialize(&mut self, root_uri: &str) -> Result<(), String> {
        let id = self.next_id;
        self.next_id += 1;
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {}
            }
        });
        self.send(&req)?;
        let _ = self.read_message_until_id(id)?;
        self.notify("initialized", serde_json::json!({}))?;
        Ok(())
    }

    pub fn did_open(&mut self, uri: &str, language_id: &str, text: &str) -> Result<(), String> {
        self.notify(
            "textDocument/didOpen",
            serde_json::json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text
                }
            }),
        )?;
        Ok(())
    }

    pub fn did_change(&mut self, uri: &str, version: i64, text: &str) -> Result<(), String> {
        self.notify(
            "textDocument/didChange",
            serde_json::json!({
                "textDocument": { "uri": uri, "version": version },
                "contentChanges": [{ "text": text }]
            }),
        )?;
        Ok(())
    }

    pub fn wait_publish_diagnostics(&mut self, uri: &str) -> Result<Vec<Problem>, String> {
        for _ in 0..32 {
            let msg = self.read_message()?;
            if msg.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
                return Ok(parse_publish_diagnostics(&msg, uri));
            }
        }
        Err("no publishDiagnostics".into())
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "shutdown",
            "params": null
        }))?;
        let _ = self.read_message_until_id(id)?;
        self.notify("exit", serde_json::json!({}))?;
        Ok(())
    }

    pub fn hover(&mut self, uri: &str, line: u32, character: u32) -> Result<Option<String>, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }
        }))?;
        let msg = self.read_message_until_id(id)?;
        let contents = msg
            .pointer("/result/contents/value")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(contents)
    }

    pub fn code_action_quick_fix(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<String>, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": { "uri": uri },
                "range": {
                    "start": { "line": line, "character": character },
                    "end": { "line": line, "character": character }
                },
                "context": { "diagnostics": [] }
            }
        }))?;
        let msg = self.read_message_until_id(id)?;
        let mut titles = Vec::new();
        if let Some(arr) = msg.pointer("/result").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(title) = item.get("title").and_then(|t| t.as_str()) {
                    titles.push(title.to_string());
                }
            }
        }
        Ok(titles)
    }

    fn send(&mut self, value: &Value) -> Result<(), String> {
        let body = serde_json::to_string(value).map_err(|e| e.to_string())?;
        write_message(&mut self.stdin, &body)?;
        Ok(())
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }))
    }

    fn read_message_until_id(&mut self, id: i64) -> Result<Value, String> {
        for _ in 0..32 {
            let msg = self.read_message()?;
            if msg.get("id").and_then(|v| v.as_i64()) == Some(id) {
                return Ok(msg);
            }
        }
        Err("response timeout".into())
    }

    fn read_message(&mut self) -> Result<Value, String> {
        let body = read_lsp_body(&mut self.stdout)?;
        serde_json::from_str(&body).map_err(|e| e.to_string())
    }
}

fn write_message(w: &mut impl Write, body: &str) -> Result<(), String> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    w.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
    w.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn read_lsp_body(r: &mut impl BufRead) -> Result<String, String> {
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        r.read_line(&mut line).map_err(|e| e.to_string())?;
        if line == "\r\n" {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            len = rest.trim().parse().map_err(|_| "bad Content-Length")?;
        }
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(String::from_utf8(buf).map_err(|e| e.to_string())?)
}

fn parse_publish_diagnostics(msg: &Value, uri: &str) -> Vec<Problem> {
    let params = msg.get("params").unwrap_or(msg);
    let doc_uri = params
        .get("uri")
        .and_then(|u| u.as_str())
        .unwrap_or(uri);
    let diags = params.get("diagnostics").and_then(|d| d.as_array());
    let mut out = Vec::new();
    for (i, d) in diags.into_iter().flatten().enumerate() {
        let range = d.get("range").unwrap_or(&Value::Null);
        let start = range.get("start").unwrap_or(&Value::Null);
        let end = range.get("end").unwrap_or(&Value::Null);
        let sl = start.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let sc = start.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let el = end
            .get("line")
            .and_then(|v| v.as_u64())
            .unwrap_or(sl as u64) as u32;
        let ec = end
            .get("character")
            .and_then(|v| v.as_u64())
            .unwrap_or(sc as u64) as u32;
        let message = d
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("diagnostic")
            .to_string();
        let sev = match d.get("severity").and_then(|s| s.as_u64()) {
            Some(1) => Severity::Error,
            Some(2) => Severity::Warning,
            Some(3) => Severity::Information,
            _ => Severity::Hint,
        };
        out.push(Problem {
            id: format!("lsp:{}:{}", doc_uri, i),
            source: "lsp".into(),
            severity: sev,
            message,
            locator: Locator::TextRange {
                start_line: sl,
                start_col: sc,
                end_line: el,
                end_col: ec,
                start_byte: 0,
                end_byte: 0,
            },
            suggestions: Vec::new(),
            code_actions: Vec::new(),
        });
    }
    out
}

#[derive(Debug, Deserialize)]
struct StubDiag {
    range: StubRange,
    message: String,
}

#[derive(Debug, Deserialize)]
struct StubRange {
    start: StubPos,
    end: StubPos,
}

#[derive(Debug, Deserialize)]
struct StubPos {
    line: u32,
    character: u32,
}
