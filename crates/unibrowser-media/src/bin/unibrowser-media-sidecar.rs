//! Stub sidecar: one JSON request per line on stdin, one JSON response per line on stdout.

use std::io::{self, BufRead, Write};

use unibrowser_media::encode::encode_h264_file;
use unibrowser_media::frame::frame_thumbnail_to_dest;
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
                frame_mime: None,
                pong: true,
            },
            Ok(SidecarRequest::ProbeFile { path }) => {
                let p = std::path::Path::new(&path);
                match probe_file_direct(p) {
                    Ok(probe) => SidecarResponse {
                        ok: true,
                        summary: probe.ffprobe_summary,
                        error: None,
                        frame_mime: None,
                        pong: false,
                    },
                    Err(e) => SidecarResponse {
                        ok: false,
                        summary: None,
                        error: Some(e.to_string()),
                        frame_mime: None,
                        pong: false,
                    },
                }
            }
            Ok(SidecarRequest::EncodeH264 {
                input_path,
                dest_path,
                preset,
            }) => {
                let input = std::path::Path::new(&input_path);
                let dest = std::path::Path::new(&dest_path);
                match encode_h264_file(input, dest, &preset) {
                    Ok(()) => SidecarResponse {
                        ok: true,
                        summary: None,
                        error: None,
                        frame_mime: None,
                        pong: false,
                    },
                    Err(e) => SidecarResponse {
                        ok: false,
                        summary: None,
                        error: Some(e.to_string()),
                        frame_mime: None,
                        pong: false,
                    },
                }
            }
            Ok(SidecarRequest::FrameAt {
                path,
                time_sec,
                dest_path,
            }) => {
                let p = std::path::Path::new(&path);
                let dest = std::path::Path::new(&dest_path);
                match frame_thumbnail_to_dest(p, time_sec, dest) {
                    Ok(()) => SidecarResponse {
                        ok: true,
                        summary: None,
                        error: None,
                        frame_mime: Some("image/jpeg".into()),
                        pong: false,
                    },
                    Err(e) => SidecarResponse {
                        ok: false,
                        summary: None,
                        error: Some(e.to_string()),
                        frame_mime: None,
                        pong: false,
                    },
                }
            }
            Err(e) => SidecarResponse {
                ok: false,
                summary: None,
                error: Some(format!("bad request: {}", e)),
                frame_mime: None,
                pong: false,
            },
        };
        let out = serde_json::to_string(&resp).unwrap();
        writeln!(io::stdout(), "{}", out)?;
    }
    Ok(())
}
