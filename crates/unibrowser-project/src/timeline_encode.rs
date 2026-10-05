//! Timeline export intents (duck, fade) for ffmpeg — see docs/timeline-encode-intents.md.

use std::path::PathBuf;

use crate::{ProjectPackage, TimelineClip};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentMedia {
    Video,
    Audio,
    Other,
}

#[derive(Debug, Clone)]
pub struct EncodeSegment {
    pub clip_id: String,
    pub path: PathBuf,
    pub media: SegmentMedia,
    pub start_sec: f32,
    pub duration_sec: f32,
    pub duck_under_video: bool,
    /// Fade into the next segment when present (video chain only).
    pub fade_to_next_sec: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct TimelineEncodeIntent {
    pub segments: Vec<EncodeSegment>,
    pub notes: Vec<String>,
}

pub fn build_timeline_encode_intent(package: &ProjectPackage) -> TimelineEncodeIntent {
    let assets = package.root.join(crate::DIR_ASSETS);
    let mut clips: Vec<&TimelineClip> = package.manifest.timeline.clips.iter().collect();
    clips.sort_by(|a, b| {
        a.start_sec
            .partial_cmp(&b.start_sec)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut segments = Vec::new();
    let mut notes = Vec::new();

    for (idx, clip) in clips.iter().enumerate() {
        let node = package
            .manifest
            .nodes
            .iter()
            .find(|n| n.id == clip.node_id);
        let Some(node) = node else {
            notes.push(format!("clip {} missing node", clip.id));
            continue;
        };
        let Some(file) = node.asset.as_ref() else {
            notes.push(format!("clip {} has no asset", clip.id));
            continue;
        };
        let path = assets.join(file);
        let media = match crate::guess_asset_node_type(std::path::Path::new(file)) {
            "video" => SegmentMedia::Video,
            "audio" => SegmentMedia::Audio,
            _ => SegmentMedia::Other,
        };
        if media == SegmentMedia::Other {
            notes.push(format!(
                "clip {} ({}) skipped — export supports video/audio assets only",
                clip.label,
                file
            ));
        }
        let fade_to_next_sec = package
            .manifest
            .timeline
            .transitions
            .iter()
            .find(|t| t.after_clip_id == clip.id && t.kind == "fade")
            .map(|t| t.duration_sec.max(0.1));
        let next_is_video = clips.get(idx + 1).and_then(|next| {
            package
                .manifest
                .nodes
                .iter()
                .find(|n| n.id == next.node_id)
                .and_then(|n| n.asset.as_ref())
                .map(|a| crate::guess_asset_node_type(std::path::Path::new(a)) == "video")
        });
        let fade = if media == SegmentMedia::Video && next_is_video == Some(true) {
            fade_to_next_sec
        } else {
            if fade_to_next_sec.is_some() && media == SegmentMedia::Video {
                notes.push(format!(
                    "fade after {} ignored (next clip is not video)",
                    clip.id
                ));
            }
            None
        };
        segments.push(EncodeSegment {
            clip_id: clip.id.clone(),
            path,
            media,
            start_sec: clip.start_sec,
            duration_sec: clip.duration_sec.max(0.1),
            duck_under_video: clip.duck_under_video,
            fade_to_next_sec: fade,
        });
    }

    if segments.iter().any(|s| s.duck_under_video) {
        notes.push(
            "duck_under_video: audio tracks mixed at ~35% volume (approximate)".into(),
        );
    }
    if segments.iter().any(|s| s.fade_to_next_sec.is_some()) {
        notes.push("fade: applied between consecutive video segments when ffmpeg xfade succeeds".into());
    }

    TimelineEncodeIntent { segments, notes }
}

pub fn video_segments(intent: &TimelineEncodeIntent) -> Vec<&EncodeSegment> {
    intent
        .segments
        .iter()
        .filter(|s| s.media == SegmentMedia::Video)
        .collect()
}

pub fn audio_segments(intent: &TimelineEncodeIntent) -> Vec<&EncodeSegment> {
    intent
        .segments
        .iter()
        .filter(|s| s.media == SegmentMedia::Audio)
        .collect()
}
