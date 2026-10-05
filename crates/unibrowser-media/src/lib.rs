//! ffmpeg / ffprobe as **T3 sidecar** — never linked into Apache core ([ADR 0007]).
//!
//! See `docs/ffmpeg-sidecar.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use thiserror::Error;

pub mod sidecar;

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
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn probe_install_does_not_panic() {
        let _ = probe_ffmpeg_install();
    }

}
