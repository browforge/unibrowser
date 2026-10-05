//! JSON-line IPC sketch for ffprobe ([`docs/ffmpeg-sidecar.md`](../../docs/ffmpeg-sidecar.md)).
//!
//! Core stays Apache-licensed; the sidecar binary may bundle LGPL ffmpeg.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::{MediaError, MediaFileProbe, find_binary};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SidecarRequest {
    ProbeFile { path: String },
    Ping,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SidecarResponse {
    #[serde(default)]
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub pong: bool,
}

/// When `UNIBROWSER_MEDIA_USE_SIDECAR=1`, route probes through the sidecar helper binary.
pub fn probe_file(path: &Path) -> Result<MediaFileProbe, MediaError> {
    let child = spawn_sidecar()?;
    let mut child = child;
    let req = SidecarRequest::ProbeFile {
        path: path.to_string_lossy().into_owned(),
    };
    let line = serde_json::to_string(&req).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    {
        let stdin = child.stdin.as_mut().ok_or_else(|| MediaError::Sidecar("stdin".into()))?;
        writeln!(stdin, "{}", line).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    }
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| MediaError::Sidecar("stdout".into()))?;
    let reader = BufReader::new(stdout);
    let response_line = reader
        .lines()
        .next()
        .ok_or_else(|| MediaError::Sidecar("no response".into()))?
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let _ = child.wait();
    let resp: SidecarResponse =
        serde_json::from_str(&response_line).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if resp.pong {
        return Err(MediaError::Sidecar("unexpected pong".into()));
    }
    if resp.ok {
        Ok(MediaFileProbe {
            path: path.to_string_lossy().into_owned(),
            ffprobe_summary: resp.summary,
        })
    } else {
        Err(MediaError::Sidecar(
            resp.error.unwrap_or_else(|| "sidecar error".into()),
        ))
    }
}

fn spawn_sidecar() -> Result<Child, MediaError> {
    let program = if let Ok(p) = std::env::var("UNIBROWSER_MEDIA_SIDECAR_BIN") {
        let path = PathBuf::from(p);
        if path.is_file() {
            path
        } else {
            return Err(MediaError::Sidecar(format!("missing {}", path.display())));
        }
    } else {
        std::env::current_exe()
            .map_err(|e| MediaError::Sidecar(e.to_string()))?
            .with_file_name("unibrowser-media-sidecar")
    };
    Command::new(&program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| MediaError::Sidecar(format!("spawn {}: {}", program.display(), e)))
}

/// Direct ffprobe (same as legacy path); used by the sidecar binary implementation.
pub fn probe_file_direct(path: &Path) -> Result<MediaFileProbe, MediaError> {
    let ffprobe = find_binary("ffprobe").ok_or(MediaError::NotFound)?;
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration,format_name",
            "-of",
            "default=noprint_wrappers=1",
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !output.status.success() {
        return Err(MediaError::Sidecar(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    let summary = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(MediaFileProbe {
        path: path.to_string_lossy().into_owned(),
        ffprobe_summary: if summary.is_empty() {
            None
        } else {
            Some(summary)
        },
    })
}
