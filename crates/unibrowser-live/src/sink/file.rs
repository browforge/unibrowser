//! Record sink: raw RGBA frames → H.264 file via ffmpeg (shared encoder lane as export).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::source::MediaFrame;

#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("ffmpeg not on PATH")]
    FfmpegMissing,
    #[error("encode failed: {0}")]
    Encode(String),
}

pub struct FileVideoSink {
    dest: PathBuf,
    width: u32,
    height: u32,
    fps: f32,
    child: Option<Child>,
}

impl FileVideoSink {
    pub fn open(dest: impl AsRef<Path>, width: u32, height: u32, fps: f32) -> Result<Self, SinkError> {
        let dest = dest.as_ref().to_path_buf();
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| SinkError::Encode(e.to_string()))?;
        }
        let ffmpeg = find_ffmpeg().ok_or(SinkError::FfmpegMissing)?;
        let child = Command::new(&ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "-s",
                &format!("{}x{}", width, height),
                "-r",
                &format!("{:.3}", fps),
                "-i",
                "pipe:0",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "28",
                "-movflags",
                "+faststart",
                "-y",
                dest.to_string_lossy().as_ref(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| SinkError::Encode(e.to_string()))?;
        Ok(Self {
            dest,
            width,
            height,
            fps,
            child: Some(child),
        })
    }

    pub fn write_frame(&mut self, frame: &MediaFrame) -> Result<(), SinkError> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| SinkError::Encode("sink closed".into()))?;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| SinkError::Encode("no stdin".into()))?;
        if frame.width != self.width || frame.height != self.height {
            return Err(SinkError::Encode("frame size mismatch".into()));
        }
        stdin
            .write_all(&frame.rgba)
            .map_err(|e| SinkError::Encode(e.to_string()))?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<PathBuf, SinkError> {
        let mut child = self
            .child
            .take()
            .ok_or_else(|| SinkError::Encode("already finished".into()))?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.flush();
        }
        let out = child
            .wait_with_output()
            .map_err(|e| SinkError::Encode(e.to_string()))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(SinkError::Encode(err.to_string()));
        }
        if !self.dest.is_file() {
            return Err(SinkError::Encode("output missing".into()));
        }
        Ok(self.dest)
    }
}

fn find_ffmpeg() -> Option<PathBuf> {
    std::env::var_os("UNIBROWSER_FFMPEG_PATH")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| which_simple("ffmpeg"))
}

fn which_simple(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn ffprobe_duration_sec(path: &Path) -> Option<f64> {
    let ffprobe = which_simple("ffprobe")?;
    let out = Command::new(&ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}
