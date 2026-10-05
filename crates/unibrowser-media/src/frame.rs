//! Single-frame thumbnails via ffmpeg (direct or sidecar).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::{MediaError, find_binary};
use crate::sidecar::SidecarRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoFrameThumbnail {
    pub path: String,
    pub mime: String,
}

fn frame_cache_dir() -> Result<PathBuf, MediaError> {
    let dir = std::env::temp_dir().join("unibrowser-video-frames");
    fs::create_dir_all(&dir).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    Ok(dir)
}

fn cache_key(path: &Path, time_sec: f64) -> String {
    let stem = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "video".into());
    let t_ms = (time_sec.max(0.0) * 1000.0).round() as u64;
    format!("{}-{}-{}.jpg", stem, path.metadata().map(|m| m.len()).unwrap_or(0), t_ms)
}

pub fn video_frame_thumbnail(path: &Path, time_sec: f64) -> Result<VideoFrameThumbnail, MediaError> {
    if std::env::var("UNIBROWSER_MEDIA_USE_SIDECAR").as_deref() == Ok("1") {
        return frame_thumbnail_pooled(path, time_sec);
    }
    frame_thumbnail_direct(path, time_sec)
}

pub fn frame_thumbnail_direct(path: &Path, time_sec: f64) -> Result<VideoFrameThumbnail, MediaError> {
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let cache = frame_cache_dir()?;
    let out = cache.join(cache_key(path, time_sec));
    if out.is_file() {
        return Ok(VideoFrameThumbnail {
            path: out.to_string_lossy().into_owned(),
            mime: "image/jpeg".into(),
        });
    }
    let time = format!("{:.3}", time_sec.max(0.0));
    let status = Command::new(&ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &time,
            "-i",
            path.to_string_lossy().as_ref(),
            "-frames:v",
            "1",
            "-q:v",
            "4",
            "-y",
            out.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !out.is_file() {
        return Err(MediaError::Sidecar("ffmpeg frame extract failed".into()));
    }
    Ok(VideoFrameThumbnail {
        path: out.to_string_lossy().into_owned(),
        mime: "image/jpeg".into(),
    })
}

pub fn frame_thumbnail_pooled(path: &Path, time_sec: f64) -> Result<VideoFrameThumbnail, MediaError> {
    let cache = frame_cache_dir()?;
    let out = cache.join(cache_key(path, time_sec));
    if out.is_file() {
        return Ok(VideoFrameThumbnail {
            path: out.to_string_lossy().into_owned(),
            mime: "image/jpeg".into(),
        });
    }
    let req = SidecarRequest::FrameAt {
        path: path.to_string_lossy().into_owned(),
        time_sec,
        dest_path: out.to_string_lossy().into_owned(),
    };
    let resp = crate::sidecar::sidecar_request_pooled(&req)?;
    if resp.ok {
        Ok(VideoFrameThumbnail {
            path: out.to_string_lossy().into_owned(),
            mime: resp
                .frame_mime
                .unwrap_or_else(|| "image/jpeg".into()),
        })
    } else {
        Err(MediaError::Sidecar(
            resp.error.unwrap_or_else(|| "frame request failed".into()),
        ))
    }
}

/// Concat demuxer copy when every input is an on-disk `.mp4`.
pub fn concat_mp4_videos(inputs: &[PathBuf], dest: &Path) -> Result<(), MediaError> {
    if inputs.is_empty() {
        return Err(MediaError::Sidecar("no inputs".into()));
    }
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let list_path = dest.with_extension("concat.txt");
    let mut list = String::new();
    for p in inputs {
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "mp4" {
            return Err(MediaError::Sidecar("concat preview requires mp4 clips".into()));
        }
        if !p.is_file() {
            return Err(MediaError::Sidecar(format!("missing {}", p.display())));
        }
        let escaped = p.to_string_lossy().replace('\'', "'\\''");
        list.push_str(&format!("file '{}'\n", escaped));
    }
    fs::write(&list_path, list).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let status = Command::new(&ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            list_path.to_string_lossy().as_ref(),
            "-c",
            "copy",
            "-y",
            dest.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let _ = fs::remove_file(&list_path);
    if !status.success() || !dest.is_file() {
        return Err(MediaError::Sidecar("ffmpeg concat failed".into()));
    }
    Ok(())
}

/// Extract one frame to an explicit path (sidecar `FrameAt`).
pub fn frame_thumbnail_to_dest(path: &Path, time_sec: f64, dest: &Path) -> Result<(), MediaError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    }
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let time = format!("{:.3}", time_sec.max(0.0));
    let status = Command::new(&ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &time,
            "-i",
            path.to_string_lossy().as_ref(),
            "-frames:v",
            "1",
            "-q:v",
            "4",
            "-y",
            dest.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !dest.is_file() {
        return Err(MediaError::Sidecar("ffmpeg frame extract failed".into()));
    }
    Ok(())
}
