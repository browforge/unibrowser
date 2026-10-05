//! Opt-in Node sidecar for loading extension `main` modules ([docs/extension-host.md](../../docs/extension-host.md)).

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::text_edits::TextEditDto;

pub const VSIX_NODE_HOST_PREF_OFF: &str =
    "Enable “Node extension host (experimental)” in Settings → Advanced";
pub const VSIX_NODE_HOST_NO_NODE: &str = "Node.js not found on PATH (extension host unavailable)";
pub const VSIX_NODE_HOST_NO_MAIN: &str = "Extension package.json has no main entry for the Node host";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSnapshot {
    pub language_id: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct VsixInvokeContext {
    pub install_dir: PathBuf,
    pub extension_root: PathBuf,
    pub main_rel: String,
    pub extension_id: String,
    pub command_id: String,
    pub editor: Option<EditorSnapshot>,
    pub configuration: Value,
}

#[derive(Debug, Clone)]
pub struct NodeExecuteResult {
    pub message: String,
    pub toast: Option<String>,
    pub toast_severity: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeHostRuntimeStatus {
    pub pid: Option<u32>,
    pub alive: bool,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum NodeHostRequest {
    Ping,
    Activate {
        install_dir: String,
        extension_dir: String,
        main: String,
        extension_id: String,
    },
    Deactivate {
        extension_id: String,
    },
    Execute {
        execute_token: u64,
        install_dir: String,
        extension_dir: String,
        main: String,
        extension_id: String,
        command_id: String,
        editor: Option<EditorSnapshot>,
        configuration: Value,
    },
    HostResults {
        execute_token: u64,
        results: Vec<HostCallResult>,
    },
    FormatDocument {
        language_id: String,
        text: String,
    },
}

#[derive(Debug, Clone)]
pub struct ExtensionHostCall {
    pub id: String,
    pub kind: String,
    pub command_id: Option<String>,
    pub edits: Vec<TextEditDto>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HostCallResult {
    id: String,
    ok: bool,
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct NodeHostToast {
    message: String,
    #[serde(default)]
    severity: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct HostCallOut {
    id: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    command_id: String,
    #[serde(default)]
    edits: Vec<TextEditDto>,
}

#[derive(Debug, Default, Deserialize)]
struct NodeHostResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    pong: bool,
    #[serde(default)]
    complete: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    toast: Option<NodeHostToast>,
    #[serde(default)]
    host_calls: Vec<HostCallOut>,
    #[serde(default)]
    execute_token: u64,
    #[serde(default)]
    edits: Vec<TextEditDto>,
}

static EXECUTE_TOKEN: Mutex<u64> = Mutex::new(1);

/// After `env_clear()`, Node on Windows needs SystemRoot (and friends) to resolve `node.exe`.
fn apply_minimal_child_env(cmd: &mut Command) {
    const KEYS: &[&str] = &[
        "SystemRoot",
        "WINDIR",
        "ComSpec",
        "TEMP",
        "TMP",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "PATHEXT",
        "HOME",
    ];
    for key in KEYS {
        if let Ok(val) = std::env::var(key) {
            cmd.env(key, val);
        }
    }
}

fn next_execute_token() -> u64 {
    let mut guard = EXECUTE_TOKEN.lock().unwrap();
    *guard += 1;
    *guard
}

pub fn node_on_path() -> bool {
    Command::new("node")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn host_script_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("UNIBROWSER_VSIX_NODE_HOST") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest.join("../../examples/vsix-node-host/host.mjs");
    if candidate.is_file() {
        return Some(candidate);
    }
    None
}

pub fn vsix_palette_enablement(pref_enabled: bool, node_ok: bool) -> (bool, Option<String>) {
    if !pref_enabled {
        return (false, Some(VSIX_NODE_HOST_PREF_OFF.into()));
    }
    if !node_ok {
        return (false, Some(VSIX_NODE_HOST_NO_NODE.into()));
    }
    (true, None)
}

struct NodeSidecar {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
}

struct SidecarPool {
    session: Option<NodeSidecar>,
    restart_count: u32,
    backoff_ms: u64,
    last_error: Option<String>,
}

impl SidecarPool {
    fn spawn_child() -> Result<NodeSidecar, String> {
        let script = host_script_path().ok_or("VSIX Node host script not found")?;
        let path_env = std::env::var("PATH").unwrap_or_default();
        let mut cmd = Command::new("node");
        cmd.arg(&script)
            .env_clear()
            .env("PATH", path_env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        apply_minimal_child_env(&mut cmd);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn node: {}", e))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "node stdout unavailable".to_string())?;
        let mut session = NodeSidecar {
            child,
            reader: BufReader::new(stdout),
        };
        let resp = session.request(&NodeHostRequest::Ping)?;
        if !resp.ok || !resp.pong {
            return Err("node host ping failed".into());
        }
        Ok(session)
    }

    fn ensure(&mut self) -> Result<&mut NodeSidecar, String> {
        if let Some(session) = self.session.as_mut() {
            if let Ok(Some(status)) = session.child.try_wait() {
                self.last_error = Some(format!("node host exited: {}", status));
                self.session = None;
            }
        }
        if self.session.is_none() {
            if self.backoff_ms > 0 {
                thread::sleep(Duration::from_millis(self.backoff_ms));
            }
            match Self::spawn_child() {
                Ok(s) => {
                    self.session = Some(s);
                    self.restart_count += 1;
                    self.backoff_ms = 200;
                }
                Err(e) => {
                    self.last_error = Some(e.clone());
                    self.backoff_ms = (self.backoff_ms * 2).min(8_000);
                    return Err(e);
                }
            }
        }
        Ok(self.session.as_mut().unwrap())
    }

    fn drop_session(&mut self) {
        if let Some(mut s) = self.session.take() {
            let _ = s.child.kill();
            let _ = s.child.wait();
        }
    }
}

static SIDECAR: Mutex<SidecarPool> = Mutex::new(SidecarPool {
    session: None,
    restart_count: 0,
    backoff_ms: 200,
    last_error: None,
});

impl NodeSidecar {
    fn request(&mut self, req: &NodeHostRequest) -> Result<NodeHostResponse, String> {
        let line = serde_json::to_string(req).map_err(|e| e.to_string())?;
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| "node stdin unavailable".to_string())?;
        writeln!(stdin, "{}", line).map_err(|e| e.to_string())?;
        let mut response_line = String::new();
        self.reader
            .read_line(&mut response_line)
            .map_err(|e| e.to_string())?;
        if response_line.trim().is_empty() {
            return Err("node host returned no response".into());
        }
        serde_json::from_str(response_line.trim()).map_err(|e| e.to_string())
    }
}

pub fn runtime_status() -> NodeHostRuntimeStatus {
    let mut guard = SIDECAR.lock().unwrap();
    let pid = guard.session.as_ref().map(|s| s.child.id());
    let alive = guard
        .session
        .as_mut()
        .map(|s| s.child.try_wait().ok().flatten().is_none())
        .unwrap_or(false);
    NodeHostRuntimeStatus {
        pid,
        alive,
        restart_count: guard.restart_count,
        last_error: guard.last_error.clone(),
    }
}

pub fn shutdown_node_host() {
    let mut guard = SIDECAR.lock().unwrap();
    guard.drop_session();
}

pub fn deactivate_extension(extension_id: &str) -> Result<(), String> {
    let mut guard = SIDECAR.lock().map_err(|e| e.to_string())?;
    if guard.session.is_none() {
        return Ok(());
    }
    match guard.ensure() {
        Ok(session) => {
            let resp = session.request(&NodeHostRequest::Deactivate {
                extension_id: extension_id.to_string(),
            })?;
            if !resp.ok {
                return Err(resp
                    .error
                    .unwrap_or_else(|| "deactivate failed".into()));
            }
        }
        Err(_) => {}
    }
    Ok(())
}

pub fn format_document_via_node(editor: EditorSnapshot) -> Result<Vec<TextEditDto>, String> {
    let mut guard = SIDECAR.lock().map_err(|e| e.to_string())?;
    let session = guard.ensure()?;
    let resp = session.request(&NodeHostRequest::FormatDocument {
        language_id: editor.language_id,
        text: editor.text,
    })?;
    if !resp.ok {
        guard.drop_session();
        return Err(resp
            .error
            .unwrap_or_else(|| "format_document failed".into()));
    }
    Ok(resp.edits)
}

pub fn execute_extension_command<F>(
    ctx: VsixInvokeContext,
    mut on_host_call: F,
) -> Result<NodeExecuteResult, String>
where
    F: FnMut(ExtensionHostCall) -> Result<String, String>,
{
    let install_dir = ctx.install_dir.to_string_lossy().to_string();
    let extension_dir = ctx.extension_root.to_string_lossy().to_string();
    let token = next_execute_token();

    let mut guard = SIDECAR.lock().map_err(|e| e.to_string())?;
    let session = guard.ensure()?;

    let activate = session.request(&NodeHostRequest::Activate {
        install_dir: install_dir.clone(),
        extension_dir: extension_dir.clone(),
        main: ctx.main_rel.clone(),
        extension_id: ctx.extension_id.clone(),
    })?;
    if !activate.ok {
        guard.drop_session();
        return Err(activate
            .error
            .unwrap_or_else(|| "activate failed".into()));
    }

    let mut resp = session.request(&NodeHostRequest::Execute {
        execute_token: token,
        install_dir,
        extension_dir,
        main: ctx.main_rel,
        extension_id: ctx.extension_id.clone(),
        command_id: ctx.command_id.clone(),
        editor: ctx.editor,
        configuration: ctx.configuration,
    })?;
    if !resp.ok {
        guard.drop_session();
        return Err(resp
            .error
            .unwrap_or_else(|| "execute failed".into()));
    }

    while !resp.complete {
        let mut results = Vec::new();
        for call in &resp.host_calls {
            let host_call = ExtensionHostCall {
                id: call.id.clone(),
                kind: if call.kind.is_empty() {
                    "builtin".into()
                } else {
                    call.kind.clone()
                },
                command_id: if call.command_id.is_empty() {
                    None
                } else {
                    Some(call.command_id.clone())
                },
                edits: call.edits.clone(),
            };
            match on_host_call(host_call) {
                Ok(msg) => results.push(HostCallResult {
                    id: call.id.clone(),
                    ok: true,
                    result: Some(msg),
                    error: None,
                }),
                Err(e) => results.push(HostCallResult {
                    id: call.id.clone(),
                    ok: false,
                    result: None,
                    error: Some(e),
                }),
            }
        }
        resp = session.request(&NodeHostRequest::HostResults {
            execute_token: resp.execute_token,
            results,
        })?;
        if !resp.ok {
            guard.drop_session();
            return Err(resp
                .error
                .unwrap_or_else(|| "host results failed".into()));
        }
    }

    let toast = resp.toast.as_ref().map(|t| t.message.clone());
    let toast_severity = resp.toast.as_ref().and_then(|t| t.severity.clone());
    let message = toast.clone().unwrap_or_else(|| {
        format!("VSIX `{}` executed via Node host", ctx.command_id)
    });
    Ok(NodeExecuteResult {
        message,
        toast,
        toast_severity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_paths() -> (PathBuf, PathBuf) {
        let install = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/vsix-sample-extension");
        let extension = install.join("extension");
        (install, extension)
    }

    #[test]
    fn node_host_runs_sample_extension_command() {
        if !node_on_path() {
            eprintln!("skip node_host_runs_sample_extension_command: node not on PATH");
            return;
        }
        if host_script_path().is_none() {
            panic!("host.mjs missing");
        }
        let (install, extension) = sample_paths();
        let out = execute_extension_command(
            VsixInvokeContext {
                install_dir: install,
                extension_root: extension,
                main_rel: "./main.js".into(),
                extension_id: "unibrowser.hello-sample".into(),
                command_id: "unibrowser.helloSample.sayHello".into(),
                editor: None,
                configuration: Value::Object(serde_json::Map::new()),
            },
            |_| Ok("noop".into()),
        )
        .expect("execute");
        assert!(
            out.toast.as_deref().unwrap_or("").contains("sample VSIX"),
            "unexpected toast: {:?}",
            out.toast
        );
    }

    #[test]
    fn node_host_probe_editor_and_builtin_command() {
        if !node_on_path() {
            eprintln!("skip node_host_probe_editor_and_builtin_command: node not on PATH");
            return;
        }
        let (install, extension) = sample_paths();
        let mut builtin_ran = false;
        let out = execute_extension_command(
            VsixInvokeContext {
                install_dir: install,
                extension_root: extension,
                main_rel: "./main.js".into(),
                extension_id: "unibrowser.hello-sample".into(),
                command_id: "unibrowser.helloSample.probeEditor".into(),
                editor: Some(EditorSnapshot {
                    language_id: "markdown".into(),
                    text: "# Title\n\nBody text for the sample extension.".into(),
                }),
                configuration: serde_json::json!({ "helloSample": { "previewChars": 12 } }),
            },
            |call| {
                if call.kind == "workspace_edit" {
                    return Ok("edits".into());
                }
                let cmd = call.command_id.as_deref().unwrap_or("");
                if cmd == "unibrowser.font.size.increase" {
                    builtin_ran = true;
                    return Ok("font increased".into());
                }
                Err(format!("unexpected builtin {}", cmd))
            },
        )
        .expect("execute");
        assert!(builtin_ran, "builtin command bus not invoked");
        let toast = out.toast.as_deref().unwrap_or("");
        assert!(toast.contains("markdown"), "toast: {}", toast);
        assert!(toast.contains("Title"), "toast: {}", toast);
    }

    #[test]
    fn node_host_format_document_provider() {
        if !node_on_path() {
            eprintln!("skip node_host_format_document_provider: node not on PATH");
            return;
        }
        let (install, extension) = sample_paths();
        let activate = execute_extension_command(
            VsixInvokeContext {
                install_dir: install.clone(),
                extension_root: extension.clone(),
                main_rel: "./main.js".into(),
                extension_id: "unibrowser.hello-sample".into(),
                command_id: "unibrowser.helloSample.sayHello".into(),
                editor: None,
                configuration: Value::Object(serde_json::Map::new()),
            },
            |_| Ok("noop".into()),
        );
        assert!(activate.is_ok(), "activate via execute: {:?}", activate.err());
        let edits = format_document_via_node(EditorSnapshot {
            language_id: "markdown".into(),
            text: "line one\nline two".into(),
        })
        .expect("format");
        assert!(!edits.is_empty(), "expected formatting edits from sample provider");
        let out = crate::text_edits::apply_text_edits("line one\nline two".into(), &edits)
            .expect("apply");
        assert!(out.ends_with('\n'), "formatted output should gain trailing newline: {:?}", out);
    }

    #[test]
    fn node_host_workspace_edit_via_command() {
        if !node_on_path() {
            eprintln!("skip node_host_workspace_edit_via_command: node not on PATH");
            return;
        }
        let (install, extension) = sample_paths();
        let mut applied = false;
        let _ = execute_extension_command(
            VsixInvokeContext {
                install_dir: install,
                extension_root: extension,
                main_rel: "./main.js".into(),
                extension_id: "unibrowser.hello-sample".into(),
                command_id: "unibrowser.helloSample.formatViaEdit".into(),
                editor: Some(EditorSnapshot {
                    language_id: "markdown".into(),
                    text: "hello".into(),
                }),
                configuration: Value::Object(serde_json::Map::new()),
            },
            |call| {
                if call.kind == "workspace_edit" {
                    applied = true;
                    assert!(!call.edits.is_empty());
                    return Ok("ok".into());
                }
                Ok("noop".into())
            },
        )
        .expect("execute");
        assert!(applied, "workspace_edit host call expected");
    }
}
