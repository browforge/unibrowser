//! Optional stdio LSP servers (stub, harper-ls when installed).

use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use unibrowser_diagnostics::{LspClient, LspServerSpec, Problem};

static STUB_CLIENT: Mutex<Option<LspClient>> = Mutex::new(None);

pub fn harper_ls_available() -> bool {
    Command::new("harper-ls")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn resolve_lsp_spec(language_id: &str) -> Option<LspServerSpec> {
    if harper_ls_available() && (language_id == "plaintext" || language_id.starts_with("en")) {
        return Some(LspServerSpec {
            program: "harper-ls".into(),
            args: vec![],
            language_id: language_id.to_string(),
        });
    }
    let stub = std::env::var("UNIBROWSER_LSP_STUB")
        .ok()
        .or_else(|| {
            std::env::var("CARGO_BIN_EXE_unibrowser-lsp-stub").ok()
        });
    stub.map(|program| LspServerSpec {
        program,
        args: vec![],
        language_id: language_id.to_string(),
    })
}

pub fn lsp_check_text(
    workspace_root: &Path,
    doc_uri: &str,
    language_id: &str,
    text: &str,
) -> Result<Vec<Problem>, String> {
    let spec = resolve_lsp_spec(language_id).ok_or("no LSP server configured")?;
    let root_uri = format!("file://{}", workspace_root.display());
    let mut client = LspClient::spawn_scrubbed(&spec)?;
    client.initialize(&root_uri)?;
    client.did_open(doc_uri, language_id, text)?;
    client.did_change(doc_uri, 2, text)?;
    let problems = client.wait_publish_diagnostics(doc_uri)?;
    client.shutdown()?;
    Ok(problems)
}

pub fn lsp_check_text_cached(
    workspace_root: &Path,
    doc_uri: &str,
    language_id: &str,
    text: &str,
) -> Result<Vec<Problem>, String> {
    let stub_only = std::env::var("CARGO_BIN_EXE_unibrowser-lsp-stub").ok();
    if stub_only.is_some() {
        let mut guard = STUB_CLIENT.lock().map_err(|e| e.to_string())?;
        if guard.is_none() {
            let spec = resolve_lsp_spec(language_id).ok_or("no LSP")?;
            let mut client = LspClient::spawn_scrubbed(&spec)?;
            let root_uri = format!("file://{}", workspace_root.display());
            client.initialize(&root_uri)?;
            *guard = Some(client);
        }
        let client = guard.as_mut().ok_or("lsp client")?;
        client.did_open(doc_uri, language_id, text)?;
        client.did_change(doc_uri, 2, text)?;
        return client.wait_publish_diagnostics(doc_uri);
    }
    lsp_check_text(workspace_root, doc_uri, language_id, text)
}
