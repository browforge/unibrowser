use unibrowser_diagnostics::{LspClient, LspServerSpec};

#[test]
fn lsp_client_receives_publish_diagnostics_from_stub() {
    let stub = std::env::var("CARGO_BIN_EXE_unibrowser-lsp-stub")
        .expect("stub binary built");
    let spec = LspServerSpec {
        program: stub,
        args: Vec::new(),
        language_id: "plaintext".into(),
    };
    let mut client = LspClient::spawn_scrubbed(&spec).expect("spawn");
    client
        .initialize("file:///workspace")
        .expect("initialize");
    client
        .did_open("file:///test.txt", "plaintext", "wrng word")
        .expect("open");
    client
        .did_change("file:///test.txt", 2, "wrng word")
        .expect("change");
    let problems = client
        .wait_publish_diagnostics("file:///test.txt")
        .expect("diagnostics");
    assert!(!problems.is_empty());
    assert!(problems[0].message.contains("stub"));
}
