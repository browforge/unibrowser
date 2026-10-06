//! ADR 0026 proving: mock OpenAI server, read grant, write grant, Ask pending.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use unibrowser_ai_assistant::{
    ActiveDocContext, AiPermission, AiScope, AssistantSession, ChatClientConfig, ChatMessage,
    PermissionStore, chat_completions, default_read_grant,
};
use unibrowser_mod_api::TextProjection;

struct StubDoc(&'static str);
impl TextProjection for StubDoc {
    fn text_projection(&self) -> String {
        self.0.to_string()
    }
}

fn spawn_mock_openai() -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).expect("read");
        let req = String::from_utf8_lossy(&buf[..n]);
        if req.contains("POST") {
            let _ = tx.send(req.to_string());
        }
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"mock-reply"}}]}"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).expect("write");
    });
    thread::sleep(Duration::from_millis(20));
    (format!("http://127.0.0.1:{}/v1", port), rx)
}

#[test]
fn mock_server_chat_round_trip() {
    let (base, rx) = spawn_mock_openai();
    let cfg = ChatClientConfig {
        base_url: Some(base),
        api_key: None,
    };
    let reply = chat_completions(
        &cfg,
        "test-model",
        &[ChatMessage {
            role: "user".into(),
            content: "hello".into(),
        }],
    )
    .expect("chat");
    assert_eq!(reply, "mock-reply");
    let req = rx.recv_timeout(Duration::from_secs(2)).expect("req");
    assert!(req.contains("/chat/completions"));
}

#[test]
fn read_projection_requires_grant() {
    let mut perms = PermissionStore::default();
    let mut session = AssistantSession::new(perms);
    let ctx = ActiveDocContext {
        workspace_root: None,
        file_ext: Some("txt".into()),
        site_origin: None,
    };
    session.user_invoke();
    let doc = StubDoc("hello doc");
    assert!(session.read_active_projection(&doc, &ctx).is_err());
    default_read_grant(session.permissions_mut());
    let text = session.read_active_projection(&doc, &ctx).expect("read");
    assert_eq!(text, "hello doc");
}

#[test]
fn write_requires_grant() {
    let perms = PermissionStore::default();
    let mut session = AssistantSession::new(perms);
    session.user_invoke();
    let ctx = ActiveDocContext {
        workspace_root: None,
        file_ext: None,
        site_origin: None,
    };
    assert!(session
        .propose_write(&ctx, "edit", "new")
        .unwrap_err()
        .to_string()
        .contains("write"));
}

#[test]
fn ask_mode_pending_until_approved() {
    let mut perms = PermissionStore::default();
    perms.grant(AiPermission::Ask, AiScope::Global);
    let mut session = AssistantSession::new(perms);
    session.user_invoke();
    let ctx = ActiveDocContext {
        workspace_root: None,
        file_ext: None,
        site_origin: None,
    };
    let pending_id = session
        .propose_write(&ctx, "replace intro", "edited")
        .expect("pending")
        .expect("id");
    assert_eq!(session.pending_actions().len(), 1);
    let applied = session.approve_pending(pending_id).expect("approve");
    assert_eq!(applied, "edited");
    assert!(session.pending_actions().is_empty());
}
