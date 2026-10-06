//! ffmpeg / ffprobe as **T3 sidecar** — never linked into Apache core ([ADR 0007]).
//!
//! See `docs/ffmpeg-sidecar.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use thiserror::Error;

pub mod audio;
pub mod encode;
pub mod frame;
pub mod midi_render;
pub mod sidecar;
mod lazy_materialize;
mod timeline_intent_encode;
pub mod svg_raster;

pub use svg_raster::{rasterize_svg_file_to_rgba, rasterize_svg_to_rgba};

pub use encode::encode_h264_timeline;
pub use midi_render::{probe_midi_audio_render, MidiAudioRenderProbe};
pub use timeline_intent_encode::{
    encode_h264_timeline_intents, EncodeIntentReport, TimelineEncodeOptions,
};

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("ffmpeg/ffprobe not found on PATH")]
    NotFound,
    #[error("sidecar failed: {0}")]
    Sidecar(String),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FfmpegProbe {
    pub available: bool,
    pub binary: Option<String>,
    pub version_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MediaFileProbe {
    pub path: String,
    pub ffprobe_summary: Option<String>,
}

/// Resolve `ffmpeg` or `ffprobe` on PATH (override with env for CI).
pub fn find_binary(name: &str) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(format!("UNIBROWSER_{}_PATH", name.to_uppercase())) {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    which_on_path(name)
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for candidate in candidate_names(name) {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn candidate_names(name: &str) -> Vec<PathBuf> {
    let has_ext = std::path::Path::new(name)
        .extension()
        .is_some_and(|e| !e.is_empty());
    let mut out = vec![PathBuf::from(name)];
    if !has_ext {
        #[cfg(windows)]
        {
            out.push(PathBuf::from(format!("{}.exe", name)));
        }
        if let Ok(pathext) = std::env::var("PATHEXT") {
            for ext in pathext.split(';').filter(|s| !s.is_empty()) {
                let ext = ext.strip_prefix('.').unwrap_or(ext);
                out.push(PathBuf::from(format!("{}.{}", name, ext)));
            }
        }
    }
    out
}

pub fn probe_ffmpeg_install() -> FfmpegProbe {
    let binary = find_binary("ffmpeg");
    if binary.is_none() {
        return FfmpegProbe {
            available: false,
            binary: None,
            version_line: None,
        };
    }
    let bin = binary.unwrap();
    let output = Command::new(&bin)
        .arg("-version")
        .output()
        .map_err(|e| e.to_string());
    let version_line = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .map(String::from),
        Err(_) => None,
    };
    FfmpegProbe {
        available: version_line.is_some(),
        binary: Some(bin.to_string_lossy().into_owned()),
        version_line,
    }
}

pub fn probe_media_file(path: &Path) -> Result<MediaFileProbe, MediaError> {
    if std::env::var("UNIBROWSER_MEDIA_USE_SIDECAR").as_deref() == Ok("1") {
        return sidecar::probe_file_pooled(path);
    }
    sidecar::probe_file_direct(path)
}

/// Warm the sidecar pool (call from desktop setup when sidecar mode is enabled).
pub fn warm_sidecar_pool() -> Result<(), MediaError> {
    if std::env::var("UNIBROWSER_MEDIA_USE_SIDECAR").as_deref() != Ok("1") {
        return Ok(());
    }
    sidecar::init_sidecar_pool()
}

pub fn video_frame_thumbnail(
    path: &Path,
    time_sec: f64,
) -> Result<frame::VideoFrameThumbnail, MediaError> {
    frame::video_frame_thumbnail(path, time_sec)
}

pub fn concat_mp4_videos(inputs: &[PathBuf], dest: &Path) -> Result<(), MediaError> {
    frame::concat_mp4_videos(inputs, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn probe_install_does_not_panic() {
        let _ = probe_ffmpeg_install();
    }

}
