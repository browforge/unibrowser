//! Opt-in Node sidecar for loading extension `main` modules ([docs/extension-host.md](../../docs/extension-host.md)).

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub const VSIX_NODE_HOST_PREF_OFF: &str =
    "Enable “Node extension host (experimental)” in Settings → Advanced";
pub const VSIX_NODE_HOST_NO_NODE: &str = "Node.js not found on PATH (extension host unavailable)";
pub const VSIX_NODE_HOST_NO_MAIN: &str = "Extension package.json has no main entry for the Node host";

#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum NodeHostRequest {
    Ping,
    Activate {
        extension_dir: String,
        main: String,
        extension_id: String,
    },
    Execute {
        extension_id: String,
        command_id: String,
    },
}

#[derive(Debug, Default, Deserialize)]
struct NodeHostToast {
    message: String,
    severity: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct NodeHostResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    pong: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    toast: Option<NodeHostToast>,
}

#[derive(Debug, Clone)]
pub struct NodeExecuteResult {
    pub message: String,
    pub toast: Option<String>,
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

impl NodeSidecar {
    fn spawn() -> Result<Self, String> {
        let script = host_script_path().ok_or("VSIX Node host script not found")?;
        let mut child = Command::new("node")
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to spawn node: {}", e))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "node stdout unavailable".to_string())?;
        let mut session = Self {
            child,
            reader: BufReader::new(stdout),
        };
        let resp = session.request(&NodeHostRequest::Ping)?;
        if !resp.ok || !resp.pong {
            return Err("node host ping failed".into());
        }
        Ok(session)
    }

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

static SIDECAR: Mutex<Option<NodeSidecar>> = Mutex::new(None);

fn with_sidecar<R>(f: impl FnOnce(&mut NodeSidecar) -> Result<R, String>) -> Result<R, String> {
    let mut guard = SIDECAR.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(NodeSidecar::spawn()?);
    }
    let session = guard.as_mut().unwrap();
    match f(session) {
        Ok(v) => Ok(v),
        Err(e) => {
            *guard = None;
            Err(e)
        }
    }
}

pub fn execute_extension_command(
    extension_root: &Path,
    main_rel: &str,
    extension_id: &str,
    command_id: &str,
) -> Result<NodeExecuteResult, String> {
    let extension_dir = extension_root.to_string_lossy().to_string();
    with_sidecar(|session| {
        let activate = session.request(&NodeHostRequest::Activate {
            extension_dir,
            main: main_rel.to_string(),
            extension_id: extension_id.to_string(),
        })?;
        if !activate.ok {
            return Err(activate
                .error
                .unwrap_or_else(|| "activate failed".into()));
        }
        let exec = session.request(&NodeHostRequest::Execute {
            extension_id: extension_id.to_string(),
            command_id: command_id.to_string(),
        })?;
        if !exec.ok {
            return Err(exec
                .error
                .unwrap_or_else(|| "execute failed".into()));
        }
        let toast = exec.toast.map(|t| t.message);
        let message = toast.clone().unwrap_or_else(|| {
            format!("VSIX `{}` executed via Node host", command_id)
        });
        Ok(NodeExecuteResult { message, toast })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_extension_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/vsix-sample-extension/extension")
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
        let root = sample_extension_root();
        assert!(root.join("main.js").is_file());
        let out = execute_extension_command(
            &root,
            "./main.js",
            "unibrowser.hello-sample",
            "unibrowser.helloSample.sayHello",
        )
        .expect("execute");
        assert!(
            out.toast.as_deref().unwrap_or("").contains("sample VSIX"),
            "unexpected toast: {:?}",
            out.toast
        );
    }
}
