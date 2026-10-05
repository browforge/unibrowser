//! H.264 export via ffmpeg (direct subprocess; optional sidecar op when pooled).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{find_binary, MediaError};
use crate::frame::concat_mp4_videos;
use crate::sidecar::sidecar_encode_h264;

pub fn h264_quality(preset: &str) -> (&'static str, &'static str) {
    match preset.to_ascii_lowercase().as_str() {
        "balanced" => ("medium", "23"),
        _ => ("veryfast", "28"),
    }
}

pub fn encode_h264_timeline(
    inputs: &[PathBuf],
    dest: &Path,
    preset: &str,
    progress: &mut dyn FnMut(u8, &str),
) -> Result<(), MediaError> {
    if inputs.is_empty() {
        return Err(MediaError::Sidecar("no video inputs".into()));
    }
    progress(5, "prepare");
    let temp = dest.with_extension("ub-concat.mp4");
    progress(15, "concat");
    concat_mp4_videos(inputs, &temp)?;
    progress(45, "encode");
    if std::env::var("UNIBROWSER_MEDIA_USE_SIDECAR").as_deref() == Ok("1") {
        sidecar_encode_h264(&temp, dest, preset)?;
    } else {
        encode_h264_file(&temp, dest, preset)?;
    }
    let _ = fs::remove_file(&temp);
    progress(100, "done");
    Ok(())
}

pub fn encode_h264_file(input: &Path, dest: &Path, preset: &str) -> Result<(), MediaError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    }
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let (ffmpeg_preset, crf) = h264_quality(preset);
    let status = Command::new(&ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            input.to_string_lossy().as_ref(),
            "-c:v",
            "libx264",
            "-preset",
            ffmpeg_preset,
            "-crf",
            crf,
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-movflags",
            "+faststart",
            "-y",
            dest.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !dest.is_file() {
        return Err(MediaError::Sidecar("ffmpeg H.264 encode failed".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::h264_quality;

    #[test]
    fn preset_mapping() {
        assert_eq!(h264_quality("fast"), ("veryfast", "28"));
        assert_eq!(h264_quality("balanced"), ("medium", "23"));
    }
}
