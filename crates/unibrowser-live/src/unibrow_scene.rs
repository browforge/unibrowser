//! Build a `LiveScene` from `.unibrow` `project.json` timeline clips (ADR 0024).

use std::path::{Path, PathBuf};

use unibrowser_project::{ProjectManifest, ProjectNode};

use crate::binding::DataBinding;
use crate::scene::{LiveLayer, LiveLayerKind, LiveScene, SceneId};
use unibrowser_project::LayerTransform;

const TEST_PATTERN_ID: &str = "live-proving/test-pattern";

pub fn scene_from_unibrow_manifest(
    manifest: &ProjectManifest,
    project_root: &Path,
) -> LiveScene {
    let width = 1280u32;
    let height = 720u32;
    let mut layers = Vec::new();

    layers.push(LiveLayer {
        id: "timeline-base".into(),
        kind: LiveLayerKind::Media {
            source_id: TEST_PATTERN_ID.into(),
        },
        transform: LayerTransform::fill(),
        bindings: DataBinding::default(),
    });

    let mut y = 16.0f32;
    for clip in &manifest.timeline.clips {
        if let Some(text) = clip.title_text.as_ref().filter(|t| !t.is_empty()) {
            layers.push(LiveLayer {
                id: format!("clip-{}", clip.id),
                kind: LiveLayerKind::Text {
                    template: text.clone(),
                },
                transform: LayerTransform::with_offset(16.0, y),
                bindings: DataBinding::default(),
            });
            y += 28.0;
            continue;
        }
        if let Some(page) = clip.pdf_page {
            layers.push(LiveLayer {
                id: format!("pdf-{}", clip.id),
                kind: LiveLayerKind::Text {
                    template: format!("PDF page {}", page),
                },
                transform: LayerTransform::with_offset(16.0, y),
                bindings: DataBinding::default(),
            });
            y += 28.0;
        }
        let node = manifest.nodes.iter().find(|n| n.id == clip.node_id);
        if let Some(path) = node.and_then(|n| asset_path(project_root, n)) {
            if path.extension().and_then(|e| e.to_str()) == Some("svg") {
                layers.push(LiveLayer {
                    id: format!("svg-{}", clip.id),
                    kind: LiveLayerKind::SvgOverlay {
                        path: path.to_string_lossy().into_owned(),
                    },
                    transform: LayerTransform {
                        offset_x: 32.0,
                        offset_y: y,
                        scale_x: 0.35,
                        scale_y: 0.35,
                        crop: None,
                    },
                    bindings: DataBinding::default(),
                });
                y += 64.0;
            }
        }
    }

    LiveScene {
        id: SceneId(format!("unibrow-{}", manifest.title)),
        width,
        height,
        layers,
    }
}

fn asset_path(root: &Path, node: &ProjectNode) -> Option<PathBuf> {
    let rel = node.asset.as_ref()?;
    let p = root.join("assets").join(rel);
    if p.is_file() {
        Some(p)
    } else {
        root.join(rel).canonicalize().ok().filter(|p| p.is_file())
    }
}
