//! Launch desktop with `--open` under xvfb; fail if shell bootstrap does not ping in time.
//! Run: UNIBROWSER_SHELL_BOOTSTRAP_IT=1 cargo test -p unibrowser-desktop shell_bootstrap_launch -- --ignored --nocapture

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[ignore = "requires xvfb and release binary; set UNIBROWSER_SHELL_BOOTSTRAP_IT=1"]
fn shell_bootstrap_launch_open_reaches_ping() {
    if std::env::var("UNIBROWSER_SHELL_BOOTSTRAP_IT").as_deref() != Ok("1") {
        return;
    }
    let root = PathBuf::from(
        std::env::var("UNIBROWSER_REPO_ROOT").unwrap_or_else(|_| "../..".into()),
    );
    let root = root.canonicalize().expect("repo root");
    let bin = root.join("target/release/unibrowser-desktop");
    assert!(bin.is_file(), "missing release binary: {}", bin.display());
    let fixture = root.join("e2e/fixtures/spell-test.txt");
    assert!(fixture.is_file());

    let timeout = Duration::from_secs(
        std::env::var("SHELL_BOOTSTRAP_PROBE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(120),
    );

    let mut child = Command::new("xvfb-run")
        .args(["-a", "-s", "-screen 0 1280x800x24"])
        .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
        .env("WEBKIT_DISABLE_COMPOSITING_MODE", "1")
        .env("UNIBROWSER_REPO_ROOT", &root)
        .env("UNIBROWSER_SAFE_MODE", "1")
        .arg(&bin)
        .args(["--open", fixture.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn desktop");

    let stderr = child.stderr.take().expect("stderr");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if line.contains("[self-test] ping shell-bootstrap-done") {
                let _ = tx.send(());
                break;
            }
        }
    });

    let start = Instant::now();
    let got = rx.recv_timeout(timeout);
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        got.is_ok(),
        "shell-bootstrap-done not seen within {:?} (elapsed {:?})",
        timeout,
        start.elapsed()
    );
}
