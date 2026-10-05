//! H.264 export honouring timeline duck/fade intents (approximate).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use unibrowser_project::{audio_segments, video_segments, EncodeSegment, SegmentMedia, TimelineEncodeIntent};

use crate::encode::encode_h264_file;
use crate::frame::concat_mp4_videos;
use crate::lazy_materialize::{materialize_visual_segment, resolve_audio_path};
use crate::{find_binary, MediaError};

#[derive(Debug, Clone, Default)]
pub struct TimelineEncodeOptions {
    /// User-configured FluidSynth soundfont (`.sf2`). When unset, common system paths are tried.
    pub soundfont_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EncodeIntentReport {
    pub honored_fade: bool,
    pub honored_clip_fade: bool,
    pub honored_duck: bool,
    pub ignored: Vec<String>,
}

struct ResolvedAudio {
    path: PathBuf,
    start_sec: f32,
    duck_under_video: bool,
    cleanup: bool,
}

pub fn encode_h264_timeline_intents(
    intent: &TimelineEncodeIntent,
    dest: &Path,
    preset: &str,
    options: &TimelineEncodeOptions,
    progress: &mut dyn FnMut(u8, &str),
) -> Result<EncodeIntentReport, MediaError> {
    let videos = video_segments(intent);
    if videos.is_empty() {
        return Err(MediaError::Sidecar("no visual clips to export".into()));
    }

    let mut report = EncodeIntentReport {
        honored_fade: false,
        honored_clip_fade: false,
        honored_duck: false,
        ignored: intent.notes.clone(),
    };

    let work_dir = dest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".ub-lazy-encode");
    fs::create_dir_all(&work_dir).map_err(|e| MediaError::Sidecar(e.to_string()))?;

    let width = intent.export_width.max(320);
    let height = intent.export_height.max(240);
    let soundfont = resolve_soundfont(options);

    progress(5, "materialize");
    let mut lazy_cleanup: Vec<PathBuf> = Vec::new();
    let mut sources: Vec<PathBuf> = Vec::with_capacity(videos.len());
    for (i, seg) in videos.iter().enumerate() {
        if seg.media == SegmentMedia::Video {
            if !seg.path.is_file() {
                return Err(MediaError::Sidecar(format!("missing {}", seg.path.display())));
            }
            let ext = seg
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext != "mp4" {
                return Err(MediaError::Sidecar(
                    "H.264 export needs MP4 video assets".into(),
                ));
            }
            sources.push(seg.path.clone());
            continue;
        }
        if seg.media != SegmentMedia::TitleCard && !seg.path.is_file() {
            return Err(MediaError::Sidecar(format!("missing {}", seg.path.display())));
        }
        let mp4 = materialize_visual_segment(seg, width, height, &work_dir, i)?;
        if mp4 != seg.path {
            lazy_cleanup.push(mp4.clone());
        }
        sources.push(mp4);
    }

    progress(8, "trim");
    let mut trimmed: Vec<PathBuf> = Vec::with_capacity(videos.len());
    for (i, seg) in videos.iter().enumerate() {
        let out = trim_video_segment(
            &sources[i],
            seg.duration_sec,
            seg.fade_in_sec,
            seg.fade_out_sec,
            dest,
            i,
        )?;
        if seg.fade_in_sec > 0.0 || seg.fade_out_sec > 0.0 {
            report.honored_clip_fade = true;
        }
        trimmed.push(out);
    }
    for p in lazy_cleanup {
        let _ = fs::remove_file(p);
    }

    progress(25, "video");
    let video_temp = dest.with_extension("ub-video.mp4");
    let fade_pair = videos.len() == 2 && videos[0].fade_to_next_sec.is_some();
    if fade_pair {
        let fade_dur = videos[0].fade_to_next_sec.unwrap_or(0.5);
        let offset = (videos[0].duration_sec - fade_dur).max(0.1);
        if xfade_two(&trimmed[0], &trimmed[1], fade_dur, offset, &video_temp).is_ok() {
            report.honored_fade = true;
        } else {
            report.ignored.push("xfade failed; used plain concat".into());
            concat_mp4_videos(&trimmed, &video_temp)?;
        }
    } else {
        if videos.iter().any(|v| v.fade_to_next_sec.is_some()) {
            report.ignored.push("fade only honored for exactly two video clips".into());
        }
        concat_mp4_videos(&trimmed, &video_temp)?;
    }

    for t in &trimmed {
        let _ = fs::remove_file(t);
    }

    let audios = audio_segments(intent);
    let final_temp = if audios.is_empty() {
        video_temp.clone()
    } else {
        progress(55, "audio");
        let mixed = dest.with_extension("ub-mix.mp4");
        let duck_any = audios.iter().any(|a| a.duck_under_video);
        let resolved = resolve_audio_tracks(audios, soundfont.as_deref(), &work_dir, &mut report)?;
        mix_audio_over_video(&video_temp, &resolved, &mixed)?;
        for r in resolved {
            if r.cleanup {
                let _ = fs::remove_file(r.path);
            }
        }
        let _ = fs::remove_file(&video_temp);
        if duck_any {
            report.honored_duck = true;
        }
        mixed
    };

    progress(70, "encode");
    encode_h264_file(&final_temp, dest, preset)?;
    let _ = fs::remove_file(&final_temp);
    let _ = fs::remove_dir_all(&work_dir);
    progress(100, "done");
    Ok(report)
}

fn resolve_soundfont(options: &TimelineEncodeOptions) -> Option<PathBuf> {
    if let Some(p) = &options.soundfont_path {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    if let Ok(p) = std::env::var("UNIBROWSER_SOUNDFONT_PATH") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    for candidate in [
        "/usr/share/sounds/sf2/FluidR3_GM.sf2",
        "/usr/share/soundfonts/FluidR3_GM.sf2",
        "/usr/share/sounds/sf2/default.sf2",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn resolve_audio_tracks(
    audios: Vec<&EncodeSegment>,
    soundfont: Option<&Path>,
    work_dir: &Path,
    report: &mut EncodeIntentReport,
) -> Result<Vec<ResolvedAudio>, MediaError> {
    let mut out = Vec::new();
    for (i, seg) in audios.into_iter().enumerate() {
        if seg.media == SegmentMedia::Audio {
            if !seg.path.is_file() {
                report
                    .ignored
                    .push(format!("audio clip {} missing file; skipped", seg.clip_id));
                continue;
            }
            out.push(ResolvedAudio {
                path: seg.path.clone(),
                start_sec: seg.start_sec,
                duck_under_video: seg.duck_under_video,
                cleanup: false,
            });
            continue;
        }
        match resolve_audio_path(seg, soundfont, work_dir, i) {
            Ok(wav) => {
                out.push(ResolvedAudio {
                    path: wav,
                    start_sec: seg.start_sec,
                    duck_under_video: seg.duck_under_video,
                    cleanup: true,
                });
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("MIDI mix skipped") {
                    report.ignored.push(msg);
                } else {
                    report
                        .ignored
                        .push(format!("midi clip {}: {}", seg.clip_id, msg));
                }
            }
        }
    }
    Ok(out)
}

fn trim_video_segment(
    input: &Path,
    duration_sec: f32,
    fade_in_sec: f32,
    fade_out_sec: f32,
    dest_hint: &Path,
    index: usize,
) -> Result<PathBuf, MediaError> {
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let out = dest_hint.with_extension(format!("ub-trim-{}.mp4", index));
    let dur = duration_sec.max(0.1);
    let dur_s = format!("{:.3}", dur);
    let fade_in = fade_in_sec.max(0.0);
    let fade_out = fade_out_sec.max(0.0);
    let needs_reencode = fade_in > 0.01 || fade_out > 0.01;
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-i".to_string(),
        input.to_string_lossy().into_owned(),
        "-t".to_string(),
        dur_s,
    ];
    if needs_reencode {
        let mut filters = Vec::new();
        if fade_in > 0.01 {
            filters.push(format!("fade=t=in:st=0:d={:.3}", fade_in.min(dur)));
        }
        if fade_out > 0.01 {
            let st = (dur - fade_out).max(0.0);
            filters.push(format!("fade=t=out:st={:.3}:d={:.3}", st, fade_out.min(dur)));
        }
        args.push("-vf".to_string());
        args.push(filters.join(","));
        args.push("-c:v".to_string());
        args.push("libx264".to_string());
        args.push("-preset".to_string());
        args.push("veryfast".to_string());
        args.push("-an".to_string());
    } else {
        args.push("-c".to_string());
        args.push("copy".to_string());
    }
    args.push("-y".to_string());
    args.push(out.to_string_lossy().into_owned());
    let status = Command::new(&ffmpeg)
        .args(&args)
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !out.is_file() {
        return Err(MediaError::Sidecar("ffmpeg trim failed".into()));
    }
    Ok(out)
}

fn xfade_two(
    a: &Path,
    b: &Path,
    fade_sec: f32,
    offset_sec: f32,
    dest: &Path,
) -> Result<(), MediaError> {
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let fade = format!("{:.3}", fade_sec.max(0.1));
    let offset = format!("{:.3}", offset_sec.max(0.0));
    let filter = format!(
        "[0:v][1:v]xfade=transition=fade:duration={}:offset={}[v]",
        fade, offset
    );
    let status = Command::new(&ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            a.to_string_lossy().as_ref(),
            "-i",
            b.to_string_lossy().as_ref(),
            "-filter_complex",
            &filter,
            "-map",
            "[v]",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-an",
            "-y",
            dest.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !dest.is_file() {
        return Err(MediaError::Sidecar("xfade failed".into()));
    }
    Ok(())
}

fn mix_audio_over_video(video: &Path, audios: &[ResolvedAudio], dest: &Path) -> Result<(), MediaError> {
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    if audios.is_empty() {
        fs::copy(video, dest).map_err(|e| MediaError::Sidecar(e.to_string()))?;
        return Ok(());
    }
    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-i".into(),
        video.to_string_lossy().into_owned(),
    ];
    for a in audios {
        args.push("-i".into());
        args.push(a.path.to_string_lossy().into_owned());
    }
    let mut filter_parts = Vec::new();
    let mut mix_inputs = String::new();
    for (i, a) in audios.iter().enumerate() {
        let idx = i + 1;
        let vol = if a.duck_under_video { "0.35" } else { "1.0" };
        let delay_ms = (a.start_sec.max(0.0) * 1000.0).round() as i64;
        filter_parts.push(format!(
            "[{idx}:a]volume={vol},adelay={delay_ms}|{delay_ms}[a{idx}]"
        ));
        mix_inputs.push_str(&format!("[a{idx}]"));
    }
    let filter = format!(
        "{};{}amix=inputs={}:duration=longest:dropout_transition=0[aout]",
        filter_parts.join(";"),
        mix_inputs,
        audios.len()
    );
    args.push("-filter_complex".into());
    args.push(filter);
    args.push("-map".into());
    args.push("0:v".into());
    args.push("-map".into());
    args.push("[aout]".into());
    args.push("-c:v".into());
    args.push("copy".into());
    args.push("-c:a".into());
    args.push("aac".into());
    args.push("-shortest".into());
    args.push("-y".into());
    args.push(dest.to_string_lossy().into_owned());
    let status = Command::new(&ffmpeg)
        .args(&args)
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !dest.is_file() {
        return Err(MediaError::Sidecar("audio mix failed".into()));
    }
    Ok(())
}
