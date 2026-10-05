//! Mono waveform peaks via ffmpeg PCM decode (proving lane for exotic codecs).

use std::path::Path;
use std::process::{Command, Stdio};

use crate::{find_binary, MediaError};

pub fn waveform_peaks(path: &Path, points: usize) -> Result<Vec<f32>, MediaError> {
    let points = points.clamp(32, 2048);
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let output = Command::new(&ffmpeg)
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &path.to_string_lossy(),
            "-ac",
            "1",
            "-ar",
            "8000",
            "-f",
            "f32le",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !output.status.success() {
        return Err(MediaError::Sidecar("ffmpeg audio decode failed".into()));
    }
    let bytes = &output.stdout;
    if bytes.len() < 4 {
        return Err(MediaError::Sidecar("no audio samples".into()));
    }
    let mut samples = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let v = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if v.is_finite() {
            samples.push(v);
        }
    }
    if samples.is_empty() {
        return Err(MediaError::Sidecar("empty waveform".into()));
    }
    Ok(downsample_peaks(&samples, points))
}

fn downsample_peaks(samples: &[f32], points: usize) -> Vec<f32> {
    let block = (samples.len() / points).max(1);
    let mut out = Vec::with_capacity(points);
    for i in 0..points {
        let start = i * block;
        if start >= samples.len() {
            break;
        }
        let end = (start + block).min(samples.len());
        let peak = samples[start..end]
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);
        out.push(peak);
    }
    out
}
