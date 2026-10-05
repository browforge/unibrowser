//! Stub sidecar: one JSON request per line on stdin, one JSON response per line on stdout.

use std::io::{self, BufRead, Write};

use unibrowser_media::sidecar::{SidecarRequest, SidecarResponse};
use unibrowser_media::sidecar::probe_file_direct;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<SidecarRequest>(&line) {
            Ok(SidecarRequest::Ping) => SidecarResponse {
                ok: true,
                summary: None,
                error: None,
                pong: true,
            },
            Ok(SidecarRequest::ProbeFile { path }) => {
                let p = std::path::Path::new(&path);
                match probe_file_direct(p) {
                    Ok(probe) => SidecarResponse {
                        ok: true,
                        summary: probe.ffprobe_summary,
                        error: None,
                        pong: false,
                    },
                    Err(e) => SidecarResponse {
                        ok: false,
                        summary: None,
                        error: Some(e.to_string()),
                        pong: false,
                    },
                }
            }
            Err(e) => SidecarResponse {
                ok: false,
                summary: None,
                error: Some(format!("bad request: {}", e)),
                pong: false,
            },
        };
        let out = serde_json::to_string(&resp).unwrap();
        writeln!(io::stdout(), "{}", out)?;
    }
    Ok(())
}
