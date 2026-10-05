//! JSON-line IPC sketch for ffprobe ([`docs/ffmpeg-sidecar.md`](../../docs/ffmpeg-sidecar.md)).
//!
//! Core stays Apache-licensed; the sidecar binary may bundle LGPL ffmpeg.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::{MediaError, MediaFileProbe, find_binary};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SidecarRequest {
    ProbeFile { path: String },
    FrameAt {
        path: String,
        time_sec: f64,
        dest_path: String,
    },
    EncodeH264 {
        input_path: String,
        dest_path: String,
        preset: String,
    },
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_mime: Option<String>,
    #[serde(default)]
    pub pong: bool,
}

/// Long-lived sidecar process (JSON line protocol on stdin/stdout).
pub struct SidecarSession {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl SidecarSession {
    pub fn spawn() -> Result<Self, MediaError> {
        let mut child = spawn_sidecar()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| MediaError::Sidecar("stdout".into()))?;
        Ok(Self {
            child,
            reader: BufReader::new(stdout),
        })
    }

    pub fn request(&mut self, req: &SidecarRequest) -> Result<SidecarResponse, MediaError> {
        let line = serde_json::to_string(req).map_err(|e| MediaError::Sidecar(e.to_string()))?;
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| MediaError::Sidecar("stdin".into()))?;
        writeln!(stdin, "{}", line).map_err(|e| MediaError::Sidecar(e.to_string()))?;
        let mut response_line = String::new();
        self.reader
            .read_line(&mut response_line)
            .map_err(|e| MediaError::Sidecar(e.to_string()))?;
        if response_line.trim().is_empty() {
            return Err(MediaError::Sidecar("no response".into()));
        }
        serde_json::from_str(response_line.trim()).map_err(|e| MediaError::Sidecar(e.to_string()))
    }
}

fn response_to_probe(path: &Path, resp: SidecarResponse) -> Result<MediaFileProbe, MediaError> {
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

static SIDECAR_POOL: Mutex<Option<SidecarSession>> = Mutex::new(None);

/// Start the pooled sidecar process if sidecar mode is on.
pub fn init_sidecar_pool() -> Result<(), MediaError> {
    let mut guard = SIDECAR_POOL
        .lock()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if guard.is_none() {
        *guard = Some(SidecarSession::spawn()?);
    }
    Ok(())
}

/// Reuse one sidecar process for multiple probes (desktop / `UNIBROWSER_MEDIA_USE_SIDECAR=1`).
pub fn sidecar_request_pooled(req: &SidecarRequest) -> Result<SidecarResponse, MediaError> {
    let mut guard = SIDECAR_POOL
        .lock()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if guard.is_none() {
        *guard = Some(SidecarSession::spawn()?);
    }
    let session = guard.as_mut().unwrap();
    match session.request(req) {
        Ok(resp) => Ok(resp),
        Err(e) => {
            *guard = None;
            Err(e)
        }
    }
}

pub fn probe_file_pooled(path: &Path) -> Result<MediaFileProbe, MediaError> {
    let mut guard = SIDECAR_POOL
        .lock()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if guard.is_none() {
        *guard = Some(SidecarSession::spawn()?);
    }
    let req = SidecarRequest::ProbeFile {
        path: path.to_string_lossy().into_owned(),
    };
    let session = guard.as_mut().unwrap();
    match session.request(&req) {
        Ok(resp) => response_to_probe(path, resp),
        Err(e) => {
            *guard = None;
            Err(e)
        }
    }
}

/// When `UNIBROWSER_MEDIA_USE_SIDECAR=1`, route probes through the pooled sidecar helper binary.
pub fn probe_file(path: &Path) -> Result<MediaFileProbe, MediaError> {
    probe_file_pooled(path)
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
pub fn sidecar_encode_h264(input: &Path, dest: &Path, preset: &str) -> Result<(), MediaError> {
    let resp = sidecar_request_pooled(&SidecarRequest::EncodeH264 {
        input_path: input.to_string_lossy().into_owned(),
        dest_path: dest.to_string_lossy().into_owned(),
        preset: preset.to_string(),
    })?;
    if resp.ok {
        Ok(())
    } else {
        Err(MediaError::Sidecar(
            resp.error.unwrap_or_else(|| "encode failed".into()),
        ))
    }
}

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
