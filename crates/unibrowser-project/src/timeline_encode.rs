//! Timeline export intents (duck, fade) for ffmpeg — see docs/timeline-encode-intents.md.

use std::path::PathBuf;

use crate::{DIR_DERIVED, ProjectPackage, TimelineClip};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentMedia {
    Video,
    Audio,
    /// PDF asset; rasterize `pdf_page` at encode time (ADR 0011).
    PdfPage,
    /// SVG asset; vector until encode (`resvg`).
    SvgStill,
    /// On-screen title card (`title_text` / `title_font_pt` on segment).
    TitleCard,
    /// SMF asset; render to WAV at encode when fluidsynth is available.
    Midi,
    Other,
}

impl SegmentMedia {
    pub fn is_visual_bus(&self) -> bool {
        matches!(
            self,
            SegmentMedia::Video
                | SegmentMedia::PdfPage
                | SegmentMedia::SvgStill
                | SegmentMedia::TitleCard
        )
    }

    pub fn is_audio_mix(&self) -> bool {
        matches!(self, SegmentMedia::Audio | SegmentMedia::Midi)
    }
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
    pub fade_in_sec: f32,
    pub fade_out_sec: f32,
    /// 1-based PDF page when `media == PdfPage`.
    pub pdf_page: u32,
    pub title_text: String,
    pub title_font_pt: f32,
}

#[derive(Debug, Clone, Default)]
pub struct TimelineEncodeIntent {
    pub segments: Vec<EncodeSegment>,
    pub notes: Vec<String>,
    pub export_width: u32,
    pub export_height: u32,
}

pub const DEFAULT_EXPORT_WIDTH: u32 = 1920;
pub const DEFAULT_EXPORT_HEIGHT: u32 = 1080;

fn segment_media_for_clip(package: &ProjectPackage, clip: &TimelineClip) -> Option<SegmentMedia> {
    let node = package.manifest.nodes.iter().find(|n| n.id == clip.node_id)?;
    if clip.title_text.is_some() || node.node_type == "title" {
        return Some(SegmentMedia::TitleCard);
    }
    let file = node.asset.as_ref()?;
    let ext = std::path::Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "pdf" || node.node_type == "pdf" {
        return Some(SegmentMedia::PdfPage);
    }
    if ext == "svg" {
        return Some(SegmentMedia::SvgStill);
    }
    match crate::guess_asset_node_type(std::path::Path::new(file)) {
        "video" => Some(SegmentMedia::Video),
        "audio" => Some(SegmentMedia::Audio),
        "midi" => Some(SegmentMedia::Midi),
        _ => Some(SegmentMedia::Other),
    }
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
        let media = if clip.title_text.is_some() || node.node_type == "title" {
            SegmentMedia::TitleCard
        } else {
            segment_media_for_clip(package, *clip).unwrap_or(SegmentMedia::Other)
        };
        let path = match node.asset.as_ref() {
            Some(file) => assets.join(file),
            None if media == SegmentMedia::TitleCard => package.root.join(DIR_DERIVED),
            None => {
                notes.push(format!("clip {} has no asset", clip.id));
                continue;
            }
        };
        if media == SegmentMedia::Other {
            notes.push(format!(
                "clip {} skipped — no encode lane for this asset type",
                clip.label
            ));
        } else if media != SegmentMedia::TitleCard && node.asset.is_some() && !path.is_file() {
            notes.push(format!("clip {} missing file {}", clip.label, path.display()));
        }
        let fade_to_next_sec = package
            .manifest
            .timeline
            .transitions
            .iter()
            .find(|t| t.after_clip_id == clip.id && t.kind == "fade")
            .map(|t| t.duration_sec.max(0.1));
        let next_on_visual_bus = clips.get(idx + 1).map(|next| {
            segment_media_for_clip(package, next).map(|m| m.is_visual_bus()) == Some(true)
        });
        let visual = media.is_visual_bus();
        let fade = if visual && next_on_visual_bus == Some(true) {
            fade_to_next_sec
        } else {
            if fade_to_next_sec.is_some() && visual {
                notes.push(format!(
                    "fade after {} ignored (next clip is not on the visual bus)",
                    clip.id
                ));
            }
            None
        };
        let title_text = clip
            .title_text
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| clip.label.clone());
        let title_font_pt = clip.title_font_pt.unwrap_or(48.0).max(8.0);
        segments.push(EncodeSegment {
            clip_id: clip.id.clone(),
            path,
            media,
            start_sec: clip.start_sec,
            duration_sec: clip.duration_sec.max(0.1),
            duck_under_video: clip.duck_under_video,
            fade_to_next_sec: fade,
            fade_in_sec: if visual {
                clip.fade_in_sec.max(0.0)
            } else {
                0.0
            },
            fade_out_sec: if visual {
                clip.fade_out_sec.max(0.0)
            } else {
                0.0
            },
            pdf_page: clip.pdf_page.unwrap_or(1).max(1),
            title_text,
            title_font_pt,
        });
    }

    if segments.iter().any(|s| s.fade_in_sec > 0.0 || s.fade_out_sec > 0.0) {
        notes.push(
            "clip fade in/out: applied per video segment via ffmpeg fade filter (approx)".into(),
        );
    }

    if segments.iter().any(|s| s.duck_under_video) {
        notes.push(
            "duck_under_video: audio tracks mixed at ~35% volume (approximate)".into(),
        );
    }
    if segments.iter().any(|s| s.fade_to_next_sec.is_some()) {
        notes.push("fade: applied between consecutive video segments when ffmpeg xfade succeeds".into());
    }

    if segments.iter().any(|s| s.media == SegmentMedia::PdfPage) {
        notes.push(
            "pdf page clips: rasterize at encode via pdftoppm (preview uses raster proxy)".into(),
        );
    }
    if segments.iter().any(|s| s.media == SegmentMedia::SvgStill) {
        notes.push("svg still clips: rasterize at encode via resvg (MIT)".into());
    }
    if segments.iter().any(|s| s.media == SegmentMedia::TitleCard) {
        notes.push(
            "title cards: rendered at encode; font size from clip title_font_pt (shared font commands)".into(),
        );
    }
    if segments.iter().any(|s| s.media == SegmentMedia::Midi) {
        notes.push(
            "midi clips: render to WAV at encode when fluidsynth + soundfont are available".into(),
        );
    }

    TimelineEncodeIntent {
        segments,
        notes,
        export_width: DEFAULT_EXPORT_WIDTH,
        export_height: DEFAULT_EXPORT_HEIGHT,
    }
}

pub fn video_segments(intent: &TimelineEncodeIntent) -> Vec<&EncodeSegment> {
    intent
        .segments
        .iter()
        .filter(|s| s.media.is_visual_bus())
        .collect()
}

pub fn audio_segments(intent: &TimelineEncodeIntent) -> Vec<&EncodeSegment> {
    intent
        .segments
        .iter()
        .filter(|s| s.media.is_audio_mix())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProjectManifest, ProjectNode, ProjectPackage, TimelineClip};

    fn pkg_with_clips(nodes: Vec<ProjectNode>, clips: Vec<TimelineClip>) -> ProjectPackage {
        ProjectPackage {
            root: std::path::PathBuf::from("/tmp/unibrow-test-intent"),
            manifest: ProjectManifest {
                format_version: crate::FORMAT_VERSION,
                title: "Intent test".into(),
                description: None,
                nodes,
                timeline: crate::ProjectTimeline {
                    clips,
                    transitions: Vec::new(),
                    markers: Vec::new(),
                },
            },
        }
    }

    #[test]
    fn intent_classifies_pdf_page_clip() {
        let pkg = pkg_with_clips(
            vec![ProjectNode {
                id: "n1".into(),
                node_type: "pdf".into(),
                label: "Doc".into(),
                asset: Some("slides.pdf".into()),
                x: 0.0,
                y: 0.0,
                color: None,
            }],
            vec![TimelineClip {
                id: "c1".into(),
                node_id: "n1".into(),
                start_sec: 0.0,
                duration_sec: 3.0,
                label: "Page".into(),
                color: None,
                clip_markers: Vec::new(),
                duck_under_video: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
                pdf_page: Some(2),
                title_text: None,
                title_font_pt: None,
            }],
        );
        let intent = build_timeline_encode_intent(&pkg);
        assert_eq!(intent.segments.len(), 1);
        assert_eq!(intent.segments[0].media, SegmentMedia::PdfPage);
        assert_eq!(intent.segments[0].pdf_page, 2);
    }

    #[test]
    fn intent_classifies_svg_midi_and_title() {
        let pkg = pkg_with_clips(
            vec![
                ProjectNode {
                    id: "svg".into(),
                    node_type: "image".into(),
                    label: "Logo".into(),
                    asset: Some("logo.svg".into()),
                    x: 0.0,
                    y: 0.0,
                    color: None,
                },
                ProjectNode {
                    id: "mid".into(),
                    node_type: "midi".into(),
                    label: "Theme".into(),
                    asset: Some("theme.mid".into()),
                    x: 0.0,
                    y: 0.0,
                    color: None,
                },
                ProjectNode {
                    id: "ttl".into(),
                    node_type: "title".into(),
                    label: "Hello".into(),
                    asset: None,
                    x: 0.0,
                    y: 0.0,
                    color: None,
                },
            ],
            vec![
                TimelineClip {
                    id: "c-svg".into(),
                    node_id: "svg".into(),
                    start_sec: 0.0,
                    duration_sec: 2.0,
                    label: "svg".into(),
                    color: None,
                    clip_markers: Vec::new(),
                    duck_under_video: false,
                    fade_in_sec: 0.0,
                    fade_out_sec: 0.0,
                    pdf_page: None,
                    title_text: None,
                    title_font_pt: None,
                },
                TimelineClip {
                    id: "c-mid".into(),
                    node_id: "mid".into(),
                    start_sec: 0.0,
                    duration_sec: 5.0,
                    label: "midi".into(),
                    color: None,
                    clip_markers: Vec::new(),
                    duck_under_video: false,
                    fade_in_sec: 0.0,
                    fade_out_sec: 0.0,
                    pdf_page: None,
                    title_text: None,
                    title_font_pt: None,
                },
                TimelineClip {
                    id: "c-ttl".into(),
                    node_id: "ttl".into(),
                    start_sec: 0.0,
                    duration_sec: 4.0,
                    label: "card".into(),
                    color: None,
                    clip_markers: Vec::new(),
                    duck_under_video: false,
                    fade_in_sec: 0.0,
                    fade_out_sec: 0.0,
                    pdf_page: None,
                    title_text: Some("Episode 1".into()),
                    title_font_pt: Some(56.0),
                },
            ],
        );
        let intent = build_timeline_encode_intent(&pkg);
        let kinds: Vec<SegmentMedia> = intent.segments.iter().map(|s| s.media).collect();
        assert!(kinds.contains(&SegmentMedia::SvgStill));
        assert!(kinds.contains(&SegmentMedia::Midi));
        assert!(kinds.contains(&SegmentMedia::TitleCard));
        let title = intent
            .segments
            .iter()
            .find(|s| s.media == SegmentMedia::TitleCard)
            .expect("title");
        assert_eq!(title.title_text, "Episode 1");
        assert_eq!(title.title_font_pt, 56.0);
    }

    #[test]
    fn adjust_title_font_pt_updates_timeline_clips() {
        let mut pkg = pkg_with_clips(
            vec![ProjectNode {
                id: "ttl".into(),
                node_type: "title".into(),
                label: "Hi".into(),
                asset: None,
                x: 0.0,
                y: 0.0,
                color: None,
            }],
            vec![TimelineClip {
                id: "c1".into(),
                node_id: "ttl".into(),
                start_sec: 0.0,
                duration_sec: 2.0,
                label: "Hi".into(),
                color: None,
                clip_markers: Vec::new(),
                duck_under_video: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
                pdf_page: None,
                title_text: Some("Hi".into()),
                title_font_pt: Some(40.0),
            }],
        );
        pkg.root = std::env::temp_dir().join("unibrow-font-adj-test");
        let _ = std::fs::create_dir_all(&pkg.root);
        crate::write_manifest(&pkg.root, &pkg.manifest).expect("manifest");
        let pt = crate::adjust_timeline_title_font_pt(&mut pkg, 4.0).expect("adjust");
        assert_eq!(pt, 44.0);
        assert_eq!(pkg.manifest.timeline.clips[0].title_font_pt, Some(44.0));
        let _ = std::fs::remove_dir_all(&pkg.root);
    }
}
