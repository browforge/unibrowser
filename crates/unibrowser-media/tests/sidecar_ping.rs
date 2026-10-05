use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

use unibrowser_media::sidecar::{SidecarRequest, SidecarSession};

#[test]
fn sidecar_ping_roundtrip() {
    let exe = std::env::var("CARGO_BIN_EXE_unibrowser-media-sidecar")
        .expect("sidecar binary should be built for this integration test");
    std::env::set_var("UNIBROWSER_MEDIA_SIDECAR_BIN", exe);
    let mut session = SidecarSession::spawn().expect("spawn sidecar");
    let resp = session.request(&SidecarRequest::Ping).expect("ping");
    assert!(resp.pong);
}

#[test]
fn sidecar_one_shot_stdio() {
    let exe = std::env::var("CARGO_BIN_EXE_unibrowser-media-sidecar").expect("sidecar bin");
    let mut child = Command::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn");
    let line = serde_json::to_string(&SidecarRequest::Ping).unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(format!("{}\n", line).as_bytes()).unwrap();
    }
    child.stdin.take();
    let mut output = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut output)
        .unwrap();
    let _ = child.wait();
    let resp: unibrowser_media::sidecar::SidecarResponse =
        serde_json::from_str(output.trim()).unwrap();
    assert!(resp.pong);
}
