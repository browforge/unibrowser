//! Minimal `.unibrow` directory bundle ([ADR 0015](../../docs/adr/0015-native-project-format.md)).

mod opc_bundle;
mod zip_bundle;
mod timeline_encode;
mod layer_transform;
mod template;

pub use opc_bundle::{
    content_hash_hex, ensure_package_history_file, ensure_unpacked_package_layout,
    is_content_hash_asset_name, is_legacy_unibrow_zip, is_opc_unibrow_zip, opc_part_bytes_equal,
    opc_part_map, PackageContainerManifest, write_legacy_plain_zip,
};

pub use template::{
    apply_project_json_to_package, apply_template_project, create_memory_package,
    export_ubtemplate_pack, is_memory_project_root, list_builtin_template_dirs,
    load_template_pack_dir, load_user_template_index, memory_root_for_session,
    memory_session_id, save_user_template_index, TemplateManifest, TemplatePack,
    TemplateRelationships, TemplateSourceKind, UserTemplateIndex, UserTemplateIndexEntry,
    MEMORY_ROOT_PREFIX,
};

pub use layer_transform::{blit_rgba_with_transform, LayerTransform, NormalizedCrop};

pub use timeline_encode::{
    build_timeline_encode_intent, audio_segments, video_segments, EncodeSegment,
    SegmentMedia, TimelineEncodeIntent,
};

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROJECT_FILE: &str = "project.json";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const HISTORY_FILE: &str = ".unibrow-history";
pub const CONTAINER_FORMAT_VERSION: u32 = 1;
pub const DIR_ASSETS: &str = "assets";
pub const DIR_REFS: &str = "refs";
pub const DIR_DERIVED: &str = "derived";

pub const FORMAT_VERSION: u32 = 2;
pub const MIN_FORMAT_VERSION: u32 = 1;

pub const LABEL_COLORS: &[&str] = &["blue", "green", "amber", "rose", "violet", "slate"];

pub fn normalize_label_color(raw: Option<&str>) -> Result<Option<String>, ProjectError> {
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() || s.eq_ignore_ascii_case("none") => Ok(None),
        Some(s) => {
            let k = s.trim().to_ascii_lowercase();
            if LABEL_COLORS.iter().any(|c| *c == k) {
                Ok(Some(k))
            } else {
                Err(ProjectError::Msg(format!(
                    "color must be one of: {}",
                    LABEL_COLORS.join(", ")
                )))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a .unibrow package (missing {0})")]
    NotPackage(String),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Board X in document space (infinite-canvas precursor).
    #[serde(default)]
    pub x: f32,
    /// Board Y in document space.
    #[serde(default)]
    pub y: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Named cue within a single timeline clip (e.g. video chapter).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipMarker {
    pub id: String,
    pub offset_sec: f32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineClip {
    pub id: String,
    /// References `ProjectNode::id` and its asset when present.
    pub node_id: String,
    pub start_sec: f32,
    pub duration_sec: f32,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clip_markers: Vec<ClipMarker>,
    /// Encode-time intent: lower this audio when a video clip is active (not applied in blitz encode).
    #[serde(default, skip_serializing_if = "is_false")]
    pub duck_under_video: bool,
    /// Video encode: fade in from black at clip start (seconds).
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub fade_in_sec: f32,
    /// Video encode: fade out to black before clip end (seconds).
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub fade_out_sec: f32,
    /// PDF timeline clip: 1-based page rasterized at encode (ADR 0011).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_page: Option<u32>,
    /// Title card copy (overrides label when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_text: Option<String>,
    /// Title card font size in pt (`unibrowser.font.size.*` on active project).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_font_pt: Option<f32>,
}

fn is_zero_f32(v: &f32) -> bool {
    *v <= 0.0 || !v.is_finite()
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Chapter / cue point on the timeline ruler.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineMarker {
    pub id: String,
    pub time_sec: f32,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Export/preview metadata only — not rendered yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineTransition {
    pub id: String,
    /// Fade (or other stub) after this clip into the next on the timeline.
    pub after_clip_id: String,
    pub kind: String,
    pub duration_sec: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectTimeline {
    #[serde(default)]
    pub clips: Vec<TimelineClip>,
    #[serde(default)]
    pub transitions: Vec<TimelineTransition>,
    #[serde(default)]
    pub markers: Vec<TimelineMarker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub nodes: Vec<ProjectNode>,
    #[serde(default)]
    pub timeline: ProjectTimeline,
}

impl ProjectManifest {
    pub fn empty(title: impl Into<String>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            title: title.into(),
            description: None,
            nodes: Vec::new(),
            timeline: ProjectTimeline::default(),
        }
    }
}

pub fn friendly_open_error(err: &ProjectError) -> String {
    match err {
        ProjectError::NotPackage(missing) => format!(
            "This is not a valid .unibrow project (missing {}).",
            missing
        ),
        ProjectError::Json(e) => format!(
            "Could not read project.json: invalid JSON ({})",
            e
        ),
        ProjectError::Io(e) => format!("Could not read the project: {}", e),
        ProjectError::Zip(e) => format!("Could not read the .unibrow archive: {}", e),
        ProjectError::Msg(s) if s.contains("format_version") => format!(
            "{}. See docs/unibrow-format-migration.md for format versions.",
            s
        ),
        ProjectError::Msg(s) => s.clone(),
    }
}

/// Full validation when opening or importing a package.
pub fn validate_manifest(manifest: &ProjectManifest) -> Result<(), ProjectError> {
    validate_manifest_for_save(manifest)?;
    if manifest.title.trim().is_empty() {
        return Err(ProjectError::Msg("project title must not be empty".into()));
    }
    for clip in &manifest.timeline.clips {
        if !manifest.nodes.iter().any(|n| n.id == clip.node_id) {
            return Err(ProjectError::Msg(format!(
                "timeline clip {} references unknown node {}",
                clip.id,
                clip.node_id
            )));
        }
    }
    Ok(())
}

/// Minimal checks for Save (ADR 0023): never block saving work-in-progress.
pub fn validate_manifest_for_save(manifest: &ProjectManifest) -> Result<(), ProjectError> {
    if manifest.format_version < MIN_FORMAT_VERSION || manifest.format_version > FORMAT_VERSION {
        return Err(ProjectError::Msg(format!(
            "unsupported format_version {} (supported {}–{})",
            manifest.format_version,
            MIN_FORMAT_VERSION,
            FORMAT_VERSION
        )));
    }
    let mut seen = std::collections::HashSet::new();
    for node in &manifest.nodes {
        if node.id.trim().is_empty() {
            return Err(ProjectError::Msg("every board node needs a non-empty id".into()));
        }
        if !seen.insert(node.id.clone()) {
            return Err(ProjectError::Msg(format!("duplicate node id {}", node.id)));
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ProjectPackage {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
}

pub fn is_unibrow_path(path: &Path) -> bool {
    if template::is_memory_project_root(path) {
        return true;
    }
    if path.is_dir() && path.join(PROJECT_FILE).is_file() {
        return true;
    }
    if path.is_dir()
        && path.join(MANIFEST_FILE).is_file()
        && path.join(PROJECT_FILE).is_file()
    {
        return true;
    }
    zip_bundle::is_unibrow_zip_file(path)
}

pub fn export_package_to_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    zip_bundle::export_directory_to_zip(root, dest_zip)
}

pub fn export_share_package_to_zip(root: &Path, dest_zip: &Path) -> Result<(), ProjectError> {
    zip_bundle::export_share_package_to_zip(root, dest_zip)
}

pub fn import_share_package_zip(
    zip_path: &Path,
    parent_dir: &Path,
    folder_name: &str,
) -> Result<PathBuf, ProjectError> {
    zip_bundle::import_share_zip_into_parent(zip_path, parent_dir, folder_name)
}

pub fn snapshot_project_autosave(root: &Path) -> Result<PathBuf, ProjectError> {
    let src = root.join(PROJECT_FILE);
    if !src.is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    let dest_dir = root.join(DIR_DERIVED);
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("project-autosave.json");
    fs::copy(&src, &dest)?;
    Ok(dest)
}

pub fn apply_project_template(
    package: &mut ProjectPackage,
    template: &str,
) -> Result<(), ProjectError> {
    match template.trim().to_ascii_lowercase().as_str() {
        "blank" | "" => Ok(()),
        "notes" | "notes_board" | "board" => seed_notes_board(package),
        "video" | "video_mix" | "mix" => seed_video_mix(package),
        other => Err(ProjectError::Msg(format!("unknown project template: {}", other))),
    }
}

/// Notes + references board with starter timeline (default teaching layout).
pub fn seed_notes_board(package: &mut ProjectPackage) -> Result<(), ProjectError> {
    seed_board_sketch(package)
}

pub fn materialize_zip_package(zip_path: &Path, cache_dir: &Path) -> Result<PathBuf, ProjectError> {
    zip_bundle::materialize_zip_package(zip_path, cache_dir)
}

pub fn list_zip_archive_entries(
    path: &Path,
    max_entries: usize,
) -> Result<Vec<zip_bundle::ZipEntrySummary>, ProjectError> {
    zip_bundle::list_zip_entries(path, max_entries)
}

pub fn extract_zip_archive_entry(
    archive_path: &Path,
    entry_path: &str,
    dest_root: &Path,
) -> Result<PathBuf, ProjectError> {
    zip_bundle::extract_zip_entry(archive_path, entry_path, dest_root)
}

/// Directory containing `project.json` — materialises zip packages when needed.
pub fn resolve_package_root(path: &Path, zip_cache_dir: &Path) -> Result<PathBuf, ProjectError> {
    if path.is_dir() && path.join(PROJECT_FILE).is_file() {
        return Ok(path.to_path_buf());
    }
    if zip_bundle::is_unibrow_zip_file(path) {
        return zip_bundle::materialize_zip_package(path, zip_cache_dir);
    }
    Err(ProjectError::NotPackage(PROJECT_FILE.into()))
}

pub fn create_package(root: &Path, title: &str) -> Result<ProjectPackage, ProjectError> {
    if root.exists() {
        return Err(ProjectError::Msg(format!("already exists: {}", root.display())));
    }
    fs::create_dir_all(root.join(DIR_ASSETS))?;
    fs::create_dir_all(root.join(DIR_REFS))?;
    fs::create_dir_all(root.join(DIR_DERIVED))?;
    let manifest = ProjectManifest::empty(title);
    opc_bundle::write_container_manifest(root, &manifest)?;
    opc_bundle::ensure_package_history_file(root)?;
    write_manifest(root, &manifest)?;
    Ok(ProjectPackage {
        root: root.to_path_buf(),
        manifest,
    })
}

/// Two starter text assets on a simple 2D board (sketch / teaching layout).
pub fn seed_board_sketch(package: &mut ProjectPackage) -> Result<(), ProjectError> {
    let notes = package.root.join(DIR_ASSETS).join("notes.txt");
    fs::write(&notes, b"Project notes\n")?;
    let refs = package.root.join(DIR_ASSETS).join("refs.txt");
    fs::write(&refs, b"Reference links\n")?;
    package.manifest.nodes = vec![
        ProjectNode {
            id: "node-1".into(),
            node_type: "file".into(),
            label: "Notes".into(),
            asset: Some("notes.txt".into()),
            x: 48.0,
            y: 56.0,
            color: None,
        },
        ProjectNode {
            id: "node-2".into(),
            node_type: "file".into(),
            label: "References".into(),
            asset: Some("refs.txt".into()),
            x: 280.0,
            y: 160.0,
            color: None,
        },
    ];
    package.manifest.timeline = ProjectTimeline {
        clips: vec![
            TimelineClip {
                id: "clip-1".into(),
                node_id: "node-1".into(),
                start_sec: 0.0,
                duration_sec: 5.0,
                label: "Notes intro".into(),
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
                id: "clip-2".into(),
                node_id: "node-2".into(),
                start_sec: 5.0,
                duration_sec: 8.0,
                label: "References".into(),
                color: None,
                clip_markers: Vec::new(),
                duck_under_video: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
                pdf_page: None,
                title_text: None,
                title_font_pt: None,
            },
        ],
        transitions: vec![TimelineTransition {
            id: "trans-1".into(),
            after_clip_id: "clip-1".into(),
            kind: "fade".into(),
            duration_sec: 0.5,
        }],
        markers: vec![],
    };
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

/// Three-slot video mix storyboard (placeholders until you drop real clips).
pub fn seed_video_mix(package: &mut ProjectPackage) -> Result<(), ProjectError> {
    let intro = package.root.join(DIR_ASSETS).join("intro-slot.txt");
    let broll = package.root.join(DIR_ASSETS).join("broll-slot.txt");
    let outro = package.root.join(DIR_ASSETS).join("outro-slot.txt");
    fs::write(&intro, b"Drop an MP4/WebM intro here\n")?;
    fs::write(&broll, b"B-roll slot\n")?;
    fs::write(&outro, b"Outro slot\n")?;
    package.manifest.nodes = vec![
        ProjectNode {
            id: "node-1".into(),
            node_type: "video".into(),
            label: "Intro".into(),
            asset: Some("intro-slot.txt".into()),
            x: 40.0,
            y: 48.0,
            color: Some("blue".into()),
        },
        ProjectNode {
            id: "node-2".into(),
            node_type: "video".into(),
            label: "B-roll".into(),
            asset: Some("broll-slot.txt".into()),
            x: 260.0,
            y: 120.0,
            color: Some("amber".into()),
        },
        ProjectNode {
            id: "node-3".into(),
            node_type: "video".into(),
            label: "Outro".into(),
            asset: Some("outro-slot.txt".into()),
            x: 480.0,
            y: 200.0,
            color: Some("violet".into()),
        },
    ];
    package.manifest.timeline = ProjectTimeline {
        clips: vec![
            TimelineClip {
                id: "clip-1".into(),
                node_id: "node-1".into(),
                start_sec: 0.0,
                duration_sec: 6.0,
                label: "Intro".into(),
                color: Some("blue".into()),
                clip_markers: Vec::new(),
                duck_under_video: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
                pdf_page: None,
                title_text: None,
                title_font_pt: None,
            },
            TimelineClip {
                id: "clip-2".into(),
                node_id: "node-2".into(),
                start_sec: 6.0,
                duration_sec: 12.0,
                label: "B-roll".into(),
                color: Some("amber".into()),
                clip_markers: Vec::new(),
                duck_under_video: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
                pdf_page: None,
                title_text: None,
                title_font_pt: None,
            },
            TimelineClip {
                id: "clip-3".into(),
                node_id: "node-3".into(),
                start_sec: 18.0,
                duration_sec: 5.0,
                label: "Outro".into(),
                color: Some("violet".into()),
                clip_markers: Vec::new(),
                duck_under_video: false,
                fade_in_sec: 0.0,
                fade_out_sec: 0.0,
                pdf_page: None,
                title_text: None,
                title_font_pt: None,
            },
        ],
        transitions: vec![TimelineTransition {
            id: "trans-1".into(),
            after_clip_id: "clip-1".into(),
            kind: "fade".into(),
            duration_sec: 0.75,
        }],
        markers: vec![
            TimelineMarker {
                id: "marker-1".into(),
                time_sec: 0.0,
                title: "Open".into(),
                color: Some("green".into()),
            },
            TimelineMarker {
                id: "marker-2".into(),
                time_sec: 18.0,
                title: "Outro".into(),
                color: Some("rose".into()),
            },
        ],
    };
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn default_board_position(node_index: usize) -> (f32, f32) {
    let col = (node_index % 4) as f32;
    let row = (node_index / 4) as f32;
    (32.0 + col * 168.0, 40.0 + row * 128.0)
}

fn timeline_end_sec(clips: &[TimelineClip]) -> f32 {
    clips
        .iter()
        .map(|c| c.start_sec + c.duration_sec)
        .fold(0.0_f32, f32::max)
}

pub fn append_timeline_clip_for_node(
    package: &mut ProjectPackage,
    node_id: &str,
) -> Result<TimelineClip, ProjectError> {
    let node = package
        .manifest
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    let start = timeline_end_sec(&package.manifest.timeline.clips);
    let id = format!("clip-{}", package.manifest.timeline.clips.len() + 1);
    let clip = TimelineClip {
        id,
        node_id: node_id.into(),
        start_sec: start,
        duration_sec: 5.0,
        label: node.label.clone(),
        color: node.color.clone(),
        clip_markers: Vec::new(),
        duck_under_video: false,
        fade_in_sec: 0.0,
        fade_out_sec: 0.0,
        pdf_page: if node.node_type == "pdf"
            || node
                .asset
                .as_ref()
                .map(|a| a.to_ascii_lowercase().ends_with(".pdf"))
                .unwrap_or(false)
        {
            Some(1)
        } else {
            None
        },
        title_text: if node.node_type == "title" {
            Some(node.label.clone())
        } else {
            None
        },
        title_font_pt: if node.node_type == "title" {
            Some(48.0)
        } else {
            None
        },
    };
    package.manifest.timeline.clips.push(clip.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(clip)
}

/// Bump or shrink `title_font_pt` on all title timeline clips (shared font commands).
pub fn adjust_timeline_title_font_pt(
    package: &mut ProjectPackage,
    delta_pt: f32,
) -> Result<f32, ProjectError> {
    let mut changed = false;
    let mut sample = 48.0_f32;
    for clip in &mut package.manifest.timeline.clips {
        let is_title = clip.title_text.is_some()
            || package
                .manifest
                .nodes
                .iter()
                .find(|n| n.id == clip.node_id)
                .map(|n| n.node_type == "title")
                == Some(true);
        if !is_title {
            continue;
        }
        let current = clip.title_font_pt.unwrap_or(48.0);
        let next = (current + delta_pt).clamp(8.0, 200.0);
        clip.title_font_pt = Some(next);
        sample = next;
        changed = true;
    }
    if !changed {
        return Err(ProjectError::Msg(
            "no title clips on timeline to resize".into(),
        ));
    }
    write_manifest(&package.root, &package.manifest)?;
    Ok(sample)
}

pub fn set_timeline_clip_duck_under_video(
    package: &mut ProjectPackage,
    clip_id: &str,
    duck_under_video: bool,
) -> Result<(), ProjectError> {
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.duck_under_video = duck_under_video;
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn set_timeline_clip_fade(
    package: &mut ProjectPackage,
    clip_id: &str,
    fade_in_sec: f32,
    fade_out_sec: f32,
) -> Result<(), ProjectError> {
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.fade_in_sec = if fade_in_sec.is_finite() && fade_in_sec > 0.0 {
        fade_in_sec
    } else {
        0.0
    };
    clip.fade_out_sec = if fade_out_sec.is_finite() && fade_out_sec > 0.0 {
        fade_out_sec
    } else {
        0.0
    };
    if clip.fade_in_sec + clip.fade_out_sec >= clip.duration_sec.max(0.1) {
        return Err(ProjectError::Msg(
            "fade in + fade out must be shorter than clip duration".into(),
        ));
    }
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn upsert_fade_transition(
    package: &mut ProjectPackage,
    after_clip_id: &str,
    duration_sec: f32,
) -> Result<TimelineTransition, ProjectError> {
    if duration_sec <= 0.0 || !duration_sec.is_finite() {
        return Err(ProjectError::Msg("transition duration must be positive".into()));
    }
    let exists = package
        .manifest
        .timeline
        .clips
        .iter()
        .any(|c| c.id == after_clip_id);
    if !exists {
        return Err(ProjectError::Msg(format!("unknown clip {}", after_clip_id)));
    }
    let transitions = &mut package.manifest.timeline.transitions;
    if let Some(t) = transitions.iter_mut().find(|t| t.after_clip_id == after_clip_id) {
        t.kind = "fade".into();
        t.duration_sec = duration_sec;
        let out = t.clone();
        write_manifest(&package.root, &package.manifest)?;
        return Ok(out);
    }
    let id = format!("trans-{}", transitions.len() + 1);
    let transition = TimelineTransition {
        id,
        after_clip_id: after_clip_id.into(),
        kind: "fade".into(),
        duration_sec,
    };
    transitions.push(transition.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(transition)
}

pub fn remove_timeline_transition(
    package: &mut ProjectPackage,
    after_clip_id: &str,
) -> Result<(), ProjectError> {
    let transitions = &mut package.manifest.timeline.transitions;
    let idx = transitions
        .iter()
        .position(|t| t.after_clip_id == after_clip_id);
    if let Some(i) = idx {
        transitions.remove(i);
        write_manifest(&package.root, &package.manifest)?;
    }
    Ok(())
}

pub fn rename_timeline_clip_label(
    package: &mut ProjectPackage,
    clip_id: &str,
    label: &str,
) -> Result<(), ProjectError> {
    let label = label.trim();
    if label.is_empty() {
        return Err(ProjectError::Msg("label cannot be empty".into()));
    }
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.label = label.to_string();
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn remove_timeline_clip(package: &mut ProjectPackage, clip_id: &str) -> Result<(), ProjectError> {
    let idx = package
        .manifest
        .timeline
        .clips
        .iter()
        .position(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    package.manifest.timeline.clips.remove(idx);
    package
        .manifest
        .timeline
        .transitions
        .retain(|t| t.after_clip_id != clip_id);
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn set_node_label_color(
    package: &mut ProjectPackage,
    node_id: &str,
    color: Option<&str>,
) -> Result<(), ProjectError> {
    let normalized = normalize_label_color(color)?;
    let node = package
        .manifest
        .nodes
        .iter_mut()
        .find(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    node.color = normalized;
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn set_timeline_clip_color(
    package: &mut ProjectPackage,
    clip_id: &str,
    color: Option<&str>,
) -> Result<(), ProjectError> {
    let normalized = normalize_label_color(color)?;
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.color = normalized;
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn add_timeline_marker(
    package: &mut ProjectPackage,
    time_sec: f32,
    title: impl Into<String>,
    color: Option<&str>,
) -> Result<TimelineMarker, ProjectError> {
    if time_sec < 0.0 || !time_sec.is_finite() {
        return Err(ProjectError::Msg("time_sec must be non-negative".into()));
    }
    let normalized = normalize_label_color(color)?;
    let id = format!("marker-{}", package.manifest.timeline.markers.len() + 1);
    let marker = TimelineMarker {
        id,
        time_sec,
        title: title.into(),
        color: normalized,
    };
    package.manifest.timeline.markers.push(marker.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(marker)
}

pub fn add_clip_marker(
    package: &mut ProjectPackage,
    clip_id: &str,
    offset_sec: f32,
    label: impl Into<String>,
) -> Result<ClipMarker, ProjectError> {
    if offset_sec < 0.0 || !offset_sec.is_finite() {
        return Err(ProjectError::Msg("offset_sec must be non-negative".into()));
    }
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    if offset_sec > clip.duration_sec {
        return Err(ProjectError::Msg("marker offset past clip end".into()));
    }
    let id = format!("{}-m{}", clip_id, clip.clip_markers.len() + 1);
    let marker = ClipMarker {
        id,
        offset_sec,
        label: label.into(),
    };
    clip.clip_markers.push(marker.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(marker)
}

pub fn remove_clip_marker(
    package: &mut ProjectPackage,
    clip_id: &str,
    marker_id: &str,
) -> Result<(), ProjectError> {
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    let idx = clip
        .clip_markers
        .iter()
        .position(|m| m.id == marker_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip marker {}", marker_id)))?;
    clip.clip_markers.remove(idx);
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn remove_timeline_marker(package: &mut ProjectPackage, marker_id: &str) -> Result<(), ProjectError> {
    let markers = &mut package.manifest.timeline.markers;
    let idx = markers
        .iter()
        .position(|m| m.id == marker_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown marker {}", marker_id)))?;
    markers.remove(idx);
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn reorder_timeline_clip(
    package: &mut ProjectPackage,
    from_index: usize,
    to_index: usize,
) -> Result<(), ProjectError> {
    let clips = &mut package.manifest.timeline.clips;
    if from_index >= clips.len() || to_index >= clips.len() || from_index == to_index {
        return Ok(());
    }
    let item = clips.remove(from_index);
    clips.insert(to_index, item);
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn update_timeline_clip_timing(
    package: &mut ProjectPackage,
    clip_id: &str,
    start_sec: f32,
    duration_sec: f32,
) -> Result<(), ProjectError> {
    if start_sec < 0.0 || !start_sec.is_finite() {
        return Err(ProjectError::Msg("start_sec must be a non-negative number".into()));
    }
    if duration_sec <= 0.0 || !duration_sec.is_finite() {
        return Err(ProjectError::Msg("duration_sec must be positive".into()));
    }
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.start_sec = start_sec;
    clip.duration_sec = duration_sec;
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

/// Timeline span for UI ruler (max of clip end times).
pub fn timeline_duration_sec(clips: &[TimelineClip]) -> f32 {
    timeline_end_sec(clips)
}

/// Resolved on-disk asset for each clip (video export / preview).
pub fn clip_asset_paths(package: &ProjectPackage) -> Vec<(TimelineClip, PathBuf, String)> {
    let assets = package.root.join(DIR_ASSETS);
    package
        .manifest
        .timeline
        .clips
        .iter()
        .filter_map(|clip| {
            let node = package
                .manifest
                .nodes
                .iter()
                .find(|n| n.id == clip.node_id)?;
            let file = node.asset.as_ref()?;
            Some((
                clip.clone(),
                assets.join(file),
                node.node_type.clone(),
            ))
        })
        .collect()
}

const EXPORT_STUB_NAME: &str = "export-mp4-preview.txt";
const EXPORT_MP4_NAME: &str = "timeline-preview.mp4";
const EXPORT_MP4_H264_NAME: &str = "timeline-export-h264.mp4";

/// Write a human-readable stub or return path when encode is handled elsewhere.
pub fn write_mp4_export_stub(
    package: &ProjectPackage,
    reason: impl Into<String>,
) -> Result<PathBuf, ProjectError> {
    fs::create_dir_all(package.root.join(DIR_DERIVED))?;
    let path = package.root.join(DIR_DERIVED).join(EXPORT_STUB_NAME);
    let mut body = String::from("Export MP4 (preview)\n\n");
    body.push_str("Full timeline encode is not available yet.\n");
    body.push_str(&format!("Reason: {}\n\n", reason.into()));
    body.push_str("Timeline clips:\n");
    for clip in &package.manifest.timeline.clips {
        body.push_str(&format!(
            "  - {} @ {:.2}s for {:.2}s (node {})\n",
            clip.label, clip.start_sec, clip.duration_sec, clip.node_id
        ));
    }
    body.push_str("\nWhen all clips are MP4 video assets and ffmpeg is on PATH, Unibrowser can concat to derived/timeline-preview.mp4.\n");
    fs::write(&path, body)?;
    Ok(path)
}

pub fn derived_mp4_preview_path(package: &ProjectPackage) -> PathBuf {
    package.root.join(DIR_DERIVED).join(EXPORT_MP4_NAME)
}

pub fn derived_mp4_h264_path(package: &ProjectPackage) -> PathBuf {
    package.root.join(DIR_DERIVED).join(EXPORT_MP4_H264_NAME)
}

pub fn export_stub_document_path(package: &ProjectPackage) -> PathBuf {
    package.root.join(DIR_DERIVED).join(EXPORT_STUB_NAME)
}

pub fn set_node_board_position(
    package: &mut ProjectPackage,
    node_id: &str,
    x: f32,
    y: f32,
) -> Result<(), ProjectError> {
    let node = package
        .manifest
        .nodes
        .iter_mut()
        .find(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    node.x = x;
    node.y = y;
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn open_package(root: &Path) -> Result<ProjectPackage, ProjectError> {
    let manifest_path = root.join(PROJECT_FILE);
    if !manifest_path.is_file() {
        return Err(ProjectError::NotPackage(PROJECT_FILE.into()));
    }
    let data = fs::read_to_string(&manifest_path)?;
    let manifest: ProjectManifest = serde_json::from_str(&data)?;
    validate_manifest(&manifest)?;
    if root.join(MANIFEST_FILE).is_file() {
        let container_data = fs::read_to_string(root.join(MANIFEST_FILE))?;
        let _container: PackageContainerManifest = serde_json::from_str(&container_data)?;
    }
    Ok(ProjectPackage {
        root: root.to_path_buf(),
        manifest,
    })
}

/// Unpacked native package (ADR 0032 §2b): `manifest.json` + `project.json` at the root.
pub fn is_unpacked_native_package(root: &Path) -> bool {
    root.is_dir()
        && root.join(MANIFEST_FILE).is_file()
        && root.join(PROJECT_FILE).is_file()
}

pub fn write_manifest(root: &Path, manifest: &ProjectManifest) -> Result<(), ProjectError> {
    if template::is_memory_project_root(root) {
        return Ok(());
    }
    let mut to_write = manifest.clone();
    to_write.format_version = FORMAT_VERSION;
    if to_write.title.trim().is_empty() {
        to_write.title = "Untitled project".into();
    }
    validate_manifest_for_save(&to_write)?;
    opc_bundle::migrate_assets_to_content_hashes(root, &mut to_write)?;
    opc_bundle::write_container_manifest(root, &to_write)?;
    opc_bundle::ensure_package_history_file(root)?;
    let data = serde_json::to_string_pretty(&to_write)?;
    let dest = root.join(PROJECT_FILE);
    unibrowser_fs::atomic_write(&dest, data.as_bytes()).map_err(|e| ProjectError::Msg(e.to_string()))?;
    Ok(())
}

pub fn guess_asset_node_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "image",
        "pdf" => "pdf",
        "docx" => "document",
        "dxf" => "drawing",
        "mp4" | "webm" => "video",
        "wav" => "audio",
        "mid" | "midi" => "midi",
        "js" | "rhai" => "script",
        _ => "file",
    }
}

pub fn sanitize_label_for_filename(label: &str) -> String {
    let mut s: String = label
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "asset".into()
    } else {
        s
    }
}

pub fn unique_asset_file_name(package: &ProjectPackage, original: &str) -> String {
    unique_asset_file_name_excluding(package, original, None)
}

/// Pick a non-colliding name under `assets/`; `exclude` is the current file (may be reused).
pub fn unique_asset_file_name_excluding(
    package: &ProjectPackage,
    original: &str,
    exclude: Option<&str>,
) -> String {
    let assets_dir = package.root.join(DIR_ASSETS);
    let path = Path::new(original);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "asset".into());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string());
    let mut candidate = original.to_string();
    let mut n = 2u32;
    while {
        let dest = assets_dir.join(&candidate);
        dest.exists() && exclude.map(|e| e != candidate).unwrap_or(true)
    } {
        candidate = match &ext {
            Some(ext) if !ext.is_empty() => format!("{}-{}.{}", stem, n, ext),
            _ => format!("{}-{}", stem, n),
        };
        n += 1;
    }
    candidate
}

pub fn insert_project_node(
    package: &mut ProjectPackage,
    node_type: &str,
    label: &str,
    asset: Option<String>,
) -> Result<ProjectNode, ProjectError> {
    let id = format!("node-{}", package.manifest.nodes.len() + 1);
    let (x, y) = default_board_position(package.manifest.nodes.len());
    let node = ProjectNode {
        id,
        node_type: node_type.into(),
        label: label.into(),
        asset,
        x,
        y,
        color: None,
    };
    package.manifest.nodes.push(node.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(node)
}

pub fn set_timeline_clip_pdf_page(
    package: &mut ProjectPackage,
    clip_id: &str,
    page: u32,
) -> Result<(), ProjectError> {
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.pdf_page = Some(page.max(1));
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn set_timeline_clip_title_text(
    package: &mut ProjectPackage,
    clip_id: &str,
    title_text: &str,
) -> Result<(), ProjectError> {
    let text = title_text.trim();
    if text.is_empty() {
        return Err(ProjectError::Msg("title text must not be empty".into()));
    }
    let clip = package
        .manifest
        .timeline
        .clips
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown clip {}", clip_id)))?;
    clip.title_text = Some(text.to_string());
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn add_pdf_page_timeline_clip(
    package: &mut ProjectPackage,
    source: &Path,
    page: u32,
    label: &str,
) -> Result<TimelineClip, ProjectError> {
    let node = add_asset_copy(package, source, "pdf", label)?;
    let clip = append_timeline_clip_for_node(package, &node.id)?;
    set_timeline_clip_pdf_page(package, &clip.id, page)?;
    package
        .manifest
        .timeline
        .clips
        .iter()
        .find(|c| c.id == clip.id)
        .cloned()
        .ok_or_else(|| ProjectError::Msg("clip missing after add".into()))
}

pub fn add_asset_timeline_clip(
    package: &mut ProjectPackage,
    source: &Path,
    label: &str,
) -> Result<TimelineClip, ProjectError> {
    let node_type = guess_asset_node_type(source);
    let node = add_asset_copy(package, source, node_type, label)?;
    append_timeline_clip_for_node(package, &node.id)
}

pub fn add_title_card_timeline_clip(
    package: &mut ProjectPackage,
    title_text: &str,
) -> Result<TimelineClip, ProjectError> {
    let text = title_text.trim();
    if text.is_empty() {
        return Err(ProjectError::Msg("title text must not be empty".into()));
    }
    let node = insert_project_node(package, "title", text, None)?;
    append_timeline_clip_for_node(package, &node.id)
}

pub fn add_asset_copy(
    package: &mut ProjectPackage,
    source: &Path,
    node_type: &str,
    label: &str,
) -> Result<ProjectNode, ProjectError> {
    if !source.is_file() {
        return Err(ProjectError::Msg("source is not a file".into()));
    }
    let id = format!("node-{}", package.manifest.nodes.len() + 1);
    let bytes = fs::read(source)?;
    let file_name = content_hash_hex(&bytes);
    let dest = package.root.join(DIR_ASSETS).join(&file_name);
    if !dest.is_file() {
        fs::write(&dest, &bytes)?;
    }
    let (x, y) = default_board_position(package.manifest.nodes.len());
    let node = ProjectNode {
        id,
        node_type: node_type.into(),
        label: label.into(),
        asset: Some(file_name),
        x,
        y,
        color: None,
    };
    package.manifest.nodes.push(node.clone());
    write_manifest(&package.root, &package.manifest)?;
    Ok(node)
}

/// Asset file names referenced by board nodes (`ProjectNode::asset`).
pub fn referenced_asset_names(manifest: &ProjectManifest) -> HashSet<String> {
    manifest
        .nodes
        .iter()
        .filter_map(|n| n.asset.as_ref())
        .cloned()
        .collect()
}

/// Files under `assets/` that are not linked from any node (timeline clips use `node_id`).
pub fn list_orphan_assets(package: &ProjectPackage) -> Result<Vec<String>, ProjectError> {
    let assets_dir = package.root.join(DIR_ASSETS);
    if !assets_dir.is_dir() {
        return Ok(vec![]);
    }
    let referenced = referenced_asset_names(&package.manifest);
    let mut orphans = Vec::new();
    for entry in fs::read_dir(&assets_dir).map_err(ProjectError::Io)? {
        let entry = entry.map_err(ProjectError::Io)?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !referenced.contains(&name) {
            orphans.push(name);
        }
    }
    orphans.sort();
    Ok(orphans)
}

fn validate_asset_file_name(file_name: &str) -> Result<(), ProjectError> {
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
    {
        return Err(ProjectError::Msg("invalid asset name".into()));
    }
    Ok(())
}

/// Delete an on-disk asset only when no node references it.
pub fn remove_orphan_asset(package: &mut ProjectPackage, file_name: &str) -> Result<(), ProjectError> {
    validate_asset_file_name(file_name)?;
    let referenced = referenced_asset_names(&package.manifest);
    if referenced.contains(file_name) {
        return Err(ProjectError::Msg(
            "asset is still referenced by a project node".into(),
        ));
    }
    let path = package.root.join(DIR_ASSETS).join(file_name);
    if !path.is_file() {
        return Err(ProjectError::Msg("asset file not found".into()));
    }
    fs::remove_file(path).map_err(ProjectError::Io)?;
    Ok(())
}

pub fn remove_node(package: &mut ProjectPackage, node_id: &str) -> Result<(), ProjectError> {
    let idx = package
        .manifest
        .nodes
        .iter()
        .position(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    let node = package.manifest.nodes.remove(idx);
    if let Some(asset) = node.asset {
        let still_used = package
            .manifest
            .nodes
            .iter()
            .any(|n| n.asset.as_deref() == Some(asset.as_str()));
        if !still_used {
            let path = package.root.join(DIR_ASSETS).join(asset);
            if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

pub fn rename_node_label(
    package: &mut ProjectPackage,
    node_id: &str,
    label: &str,
) -> Result<(), ProjectError> {
    let label = label.trim();
    if label.is_empty() {
        return Err(ProjectError::Msg("label cannot be empty".into()));
    }
    let idx = package
        .manifest
        .nodes
        .iter()
        .position(|n| n.id == node_id)
        .ok_or_else(|| ProjectError::Msg(format!("unknown node {}", node_id)))?;
    let old_name = package.manifest.nodes[idx].asset.clone();
    let mut new_asset = old_name.clone();
    if let Some(ref old_name) = old_name {
        if is_content_hash_asset_name(old_name) {
            let node = &mut package.manifest.nodes[idx];
            node.label = label.to_string();
            write_manifest(&package.root, &package.manifest)?;
            return Ok(());
        }
        let assets_dir = package.root.join(DIR_ASSETS);
        let old_path = assets_dir.join(old_name);
        if old_path.is_file() {
            let ext = Path::new(old_name)
                .extension()
                .map(|e| e.to_string_lossy().to_string());
            let base = sanitize_label_for_filename(label);
            let desired = match &ext {
                Some(ext) if !ext.is_empty() => format!("{}.{}", base, ext),
                _ => base.clone(),
            };
            let new_name =
                unique_asset_file_name_excluding(package, &desired, Some(old_name));
            if new_name != *old_name {
                let new_path = assets_dir.join(&new_name);
                fs::rename(&old_path, &new_path)?;
            }
            new_asset = Some(new_name);
        }
    }
    let node = &mut package.manifest.nodes[idx];
    node.asset = new_asset;
    node.label = label.to_string();
    write_manifest(&package.root, &package.manifest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sacred_save_empty_timeline_writes_manifest() {
        let dir = std::env::temp_dir().join("unibrow-sacred-empty-timeline");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let manifest = ProjectManifest::empty("No video yet");
        write_manifest(&dir, &manifest).expect("save must not block on empty timeline");
        assert!(dir.join(PROJECT_FILE).is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sacred_save_orphan_clip_allowed_on_write() {
        let dir = std::env::temp_dir().join("unibrow-sacred-orphan-clip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let manifest = ProjectManifest {
            format_version: FORMAT_VERSION,
            title: "WIP".into(),
            description: None,
            nodes: Vec::new(),
            timeline: {
                let mut tl = ProjectTimeline::default();
                tl.clips = vec![TimelineClip {
                    id: "c1".into(),
                    node_id: "missing-node".into(),
                    start_sec: 0.0,
                    duration_sec: 1.0,
                    label: "wip".into(),
                    color: None,
                    clip_markers: Vec::new(),
                    duck_under_video: false,
                    fade_in_sec: 0.0,
                    fade_out_sec: 0.0,
                    pdf_page: None,
                    title_text: None,
                    title_font_pt: None,
                }];
                tl
            },
        };
        write_manifest(&dir, &manifest).expect("save work-in-progress");
        assert!(validate_manifest(&manifest).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_manifest_rejects_duplicate_nodes() {
        let manifest = ProjectManifest {
            format_version: 2,
            title: "T".into(),
            description: None,
            nodes: vec![
                ProjectNode {
                    id: "a".into(),
                    node_type: "note".into(),
                    label: "A".into(),
                    asset: None,
                    x: 0.0,
                    y: 0.0,
                    color: None,
                },
                ProjectNode {
                    id: "a".into(),
                    node_type: "note".into(),
                    label: "B".into(),
                    asset: None,
                    x: 0.0,
                    y: 0.0,
                    color: None,
                },
            ],
            timeline: ProjectTimeline::default(),
        };
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn unique_asset_name_when_taken() {
        let dir = std::env::temp_dir().join("unibrow-test-dup");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Dup").expect("create");
        let incoming = dir.join("incoming");
        fs::create_dir_all(&incoming).unwrap();
        let first = incoming.join("photo.png");
        fs::write(&first, b"1").unwrap();
        add_asset_copy(&mut pkg, &first, "image", "first").expect("first");
        let second = incoming.join("photo.png");
        fs::write(&second, b"2").unwrap();
        let node = add_asset_copy(&mut pkg, &second, "image", "second").expect("dup");
        let first_hash = pkg.manifest.nodes[0].asset.clone().unwrap();
        let second_hash = node.asset.clone().unwrap();
        assert_ne!(first_hash, second_hash);
        assert!(is_content_hash_asset_name(&first_hash));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn orphan_asset_list_and_remove() {
        let dir = std::env::temp_dir().join("unibrow-test-orphan");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Orphan").expect("create");
        let src = dir.join("orphan.txt");
        fs::write(&src, "x").expect("write");
        add_asset_copy(&mut pkg, &src, "file", "linked").expect("add");
        let linked = pkg.manifest.nodes[0].asset.clone().unwrap();
        let orphan_path = pkg.root.join(DIR_ASSETS).join("stray.bin");
        fs::write(&orphan_path, b"orphan").expect("orphan");
        let orphans = list_orphan_assets(&pkg).expect("list");
        assert_eq!(orphans, vec!["stray.bin".to_string()]);
        assert!(referenced_asset_names(&pkg.manifest).contains(&linked));
        remove_orphan_asset(&mut pkg, "stray.bin").expect("remove");
        assert!(list_orphan_assets(&pkg).expect("list").is_empty());
        assert!(remove_orphan_asset(&mut pkg, &linked).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_node_drops_asset() {
        let dir = std::env::temp_dir().join("unibrow-test-remove");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Rm").expect("create");
        let src = dir.join("x.txt");
        fs::write(&src, b"x").unwrap();
        let node = add_asset_copy(&mut pkg, &src, "file", "x").expect("add");
        let id = node.id.clone();
        remove_node(&mut pkg, &id).expect("remove");
        assert!(pkg.manifest.nodes.is_empty());
        assert!(fs::read_dir(dir.join(DIR_ASSETS)).unwrap().count() == 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_node_renames_asset_file() {
        let dir = std::env::temp_dir().join("unibrow-test-rename");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Ren").expect("create");
        let src = dir.join("incoming.txt");
        fs::write(&src, b"hi").unwrap();
        let node = add_asset_copy(&mut pkg, &src, "file", "Old Label").expect("add");
        let id = node.id.clone();
        rename_node_label(&mut pkg, &id, "My Asset").expect("rename");
        assert_eq!(pkg.manifest.nodes[0].label, "My Asset");
        let hash = pkg.manifest.nodes[0].asset.clone().unwrap();
        assert!(is_content_hash_asset_name(&hash));
        assert!(dir.join(DIR_ASSETS).join(hash).is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_and_open_roundtrip() {
        let dir = std::env::temp_dir().join("unibrow-test-create");
        let _ = fs::remove_dir_all(&dir);
        let _pkg = create_package(&dir, "Demo").expect("create");
        assert!(dir.join(DIR_ASSETS).is_dir());
        let again = open_package(&dir).expect("open");
        assert_eq!(again.manifest.title, "Demo");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_timeline_clip_label_persists() {
        let dir = std::env::temp_dir().join("unibrow-test-clip-rename");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Clip").expect("create");
        seed_board_sketch(&mut pkg).expect("seed");
        let id = pkg.manifest.nodes[0].id.clone();
        let clip = append_timeline_clip_for_node(&mut pkg, &id).expect("clip");
        rename_timeline_clip_label(&mut pkg, &clip.id, "Intro").expect("rename");
        let updated = pkg
            .manifest
            .timeline
            .clips
            .iter()
            .find(|c| c.id == clip.id)
            .expect("clip");
        assert_eq!(updated.label, "Intro");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeline_clip_roundtrip() {
        let dir = std::env::temp_dir().join("unibrow-test-timeline");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Tl").expect("create");
        seed_board_sketch(&mut pkg).expect("seed");
        let id = pkg.manifest.nodes[0].id.clone();
        append_timeline_clip_for_node(&mut pkg, &id).expect("append");
        assert_eq!(pkg.manifest.timeline.clips.len(), 3);
        let clip_id = pkg.manifest.timeline.clips[2].id.clone();
        remove_timeline_clip(&mut pkg, &clip_id).expect("remove");
        assert_eq!(pkg.manifest.timeline.clips.len(), 2);
        reorder_timeline_clip(&mut pkg, 1, 0).expect("reorder");
        assert_eq!(pkg.manifest.timeline.clips[0].node_id, "node-2");
        let clip_id = pkg.manifest.timeline.clips[0].id.clone();
        update_timeline_clip_timing(&mut pkg, &clip_id, 1.5, 4.0).expect("update timing");
        assert_eq!(pkg.manifest.timeline.clips[0].start_sec, 1.5);
        assert_eq!(pkg.manifest.timeline.clips[0].duration_sec, 4.0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clip_marker_on_timeline_clip() {
        let dir = std::env::temp_dir().join("unibrow-test-clip-marker");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Cue").expect("create");
        pkg.manifest.nodes.push(ProjectNode {
            id: "vid-node".into(),
            node_type: "video".into(),
            label: "Take".into(),
            asset: Some("take.mp4".into()),
            x: 0.0,
            y: 0.0,
            color: None,
        });
        let clip = append_timeline_clip_for_node(&mut pkg, "vid-node").expect("clip");
        let m = add_clip_marker(&mut pkg, &clip.id, 2.0, "Beat").expect("marker");
        assert_eq!(m.label, "Beat");
        let again = open_package(&dir).expect("open");
        assert_eq!(again.manifest.timeline.clips[0].clip_markers.len(), 1);
        remove_clip_marker(&mut pkg, &clip.id, &m.id).expect("remove");
        assert!(open_package(&dir)
            .expect("open2")
            .manifest
            .timeline
            .clips[0]
            .clip_markers
            .is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn board_sketch_has_two_positioned_nodes() {
        let dir = std::env::temp_dir().join("unibrow-test-board");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Board").expect("create");
        seed_board_sketch(&mut pkg).expect("seed");
        assert_eq!(pkg.manifest.nodes.len(), 2);
        assert_eq!(pkg.manifest.nodes[0].x, 48.0);
        assert_eq!(pkg.manifest.nodes[1].y, 160.0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn zip_export_and_materialize() {
        let dir = std::env::temp_dir().join("unibrow-test-zip");
        let _ = fs::remove_dir_all(&dir);
        let _pkg = create_package(&dir, "ZipMe").expect("create");
        let zip_path = dir.with_extension("unibrow");
        export_package_to_zip(&dir, &zip_path).expect("export");
        assert!(zip_path.is_file());
        assert!(is_unibrow_path(&zip_path));
        let cache = dir.join("cache");
        let materialized = materialize_zip_package(&zip_path, &cache).expect("materialize");
        let opened = open_package(&materialized).expect("open");
        assert_eq!(opened.manifest.title, "ZipMe");
        let listed = list_zip_archive_entries(&zip_path, 50).expect("list");
        assert!(listed.iter().any(|e| e.path.contains("project.json")));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&zip_path);
    }

    #[test]
    fn project_templates_roundtrip() {
        let dir = std::env::temp_dir().join("unibrow-test-templates");
        let _ = fs::remove_dir_all(&dir);
        let blank_dir = dir.join("blank");
        let mut blank = create_package(&blank_dir, "Blank").expect("create");
        apply_project_template(&mut blank, "blank").expect("blank");
        assert!(blank.manifest.nodes.is_empty());
        let notes_dir = dir.join("notes");
        let mut notes = create_package(&notes_dir, "Notes").expect("create");
        apply_project_template(&mut notes, "notes").expect("notes");
        assert_eq!(notes.manifest.nodes.len(), 2);
        let video_dir = dir.join("video");
        let mut video = create_package(&video_dir, "Video").expect("create");
        apply_project_template(&mut video, "video_mix").expect("video");
        assert_eq!(video.manifest.timeline.clips.len(), 3);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeline_crud_save_load_fuzz() {
        let dir = std::env::temp_dir().join("unibrow-test-timeline-fuzz");
        let _ = fs::remove_dir_all(&dir);
        let mut pkg = create_package(&dir, "Fuzz").expect("create");
        seed_board_sketch(&mut pkg).expect("seed");
        let node_ids: Vec<String> = pkg.manifest.nodes.iter().map(|n| n.id.clone()).collect();
        let mut rng = 0xC0FFEE_u64;
        let mut step = 0u32;
        while step < 96 {
            rng = rng.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let op = (rng >> 17) as u8 % 9;
            match op {
                0 if !node_ids.is_empty() => {
                    let i = (rng as usize) % node_ids.len();
                    let _ = append_timeline_clip_for_node(&mut pkg, &node_ids[i]);
                }
                1 => {
                    if let Some(c) = pkg.manifest.timeline.clips.last() {
                        let id = c.id.clone();
                        let _ = remove_timeline_clip(&mut pkg, &id);
                    }
                }
                2 if pkg.manifest.timeline.clips.len() > 1 => {
                    let len = pkg.manifest.timeline.clips.len();
                    let from = (rng as usize) % len;
                    let to = (rng as usize >> 8) % len;
                    let _ = reorder_timeline_clip(&mut pkg, from, to);
                }
                3 if !pkg.manifest.timeline.clips.is_empty() => {
                    let i = (rng as usize) % pkg.manifest.timeline.clips.len();
                    let id = pkg.manifest.timeline.clips[i].id.clone();
                    let start = ((rng >> 5) % 400) as f32 / 10.0;
                    let dur = 0.5 + ((rng >> 11) % 80) as f32 / 10.0;
                    let _ = update_timeline_clip_timing(&mut pkg, &id, start, dur);
                }
                4 if !pkg.manifest.timeline.clips.is_empty() => {
                    let i = (rng as usize) % pkg.manifest.timeline.clips.len();
                    let id = pkg.manifest.timeline.clips[i].id.clone();
                    let _ = upsert_fade_transition(&mut pkg, &id, 0.25 + (rng % 5) as f32 / 10.0);
                }
                5 => {
                    let t = ((rng >> 3) % 200) as f32 / 10.0;
                    let _ = add_timeline_marker(&mut pkg, t, format!("M{}", step), Some("blue"));
                }
                6 if !pkg.manifest.timeline.markers.is_empty() => {
                    let i = (rng as usize) % pkg.manifest.timeline.markers.len();
                    let id = pkg.manifest.timeline.markers[i].id.clone();
                    let _ = remove_timeline_marker(&mut pkg, &id);
                }
                7 if !node_ids.is_empty() => {
                    let i = (rng as usize) % node_ids.len();
                    let colors = ["blue", "green", "amber", "none"];
                    let c = colors[(rng as usize >> 4) % colors.len()];
                    let _ = set_node_label_color(&mut pkg, &node_ids[i], Some(c));
                }
                _ => {}
            }
            pkg = open_package(&dir).expect("reload");
            let on_disk = fs::read_to_string(dir.join(PROJECT_FILE)).expect("read json");
            let parsed: ProjectManifest = serde_json::from_str(&on_disk).expect("parse json");
            assert_eq!(parsed.timeline.clips.len(), pkg.manifest.timeline.clips.len());
            assert_eq!(
                parsed.timeline.markers.len(),
                pkg.manifest.timeline.markers.len()
            );
            step += 1;
        }
        let exported = dir.with_extension("fuzz.unibrow");
        export_package_to_zip(&dir, &exported).expect("zip");
        let cache = dir.join("zip-cache");
        let materialized = materialize_zip_package(&exported, &cache).expect("materialize");
        let again = open_package(&materialized).expect("open zip");
        assert_eq!(again.manifest.title, "Fuzz");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&exported);
        let _ = fs::remove_dir_all(&cache);
    }
}
