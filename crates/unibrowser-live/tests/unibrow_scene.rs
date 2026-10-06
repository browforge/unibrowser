use std::path::Path;

use unibrowser_live::{scene_from_unibrow_manifest, LiveLayerKind};
use unibrowser_project::{ProjectManifest, ProjectTimeline, TimelineClip};

#[test]
fn scene_includes_title_clip_layer() {
    let mut manifest = ProjectManifest::empty("Demo");
    manifest.timeline = ProjectTimeline {
        clips: vec![TimelineClip {
            id: "c1".into(),
            node_id: "n1".into(),
            label: "title".into(),
            title_text: Some("Hello timeline".into()),
            start_sec: 0.0,
            duration_sec: 1.0,
            color: None,
            clip_markers: vec![],
            duck_under_video: false,
            fade_in_sec: 0.0,
            fade_out_sec: 0.0,
            pdf_page: None,
            title_font_pt: None,
        }],
        ..Default::default()
    };
    let scene = scene_from_unibrow_manifest(&manifest, Path::new("/tmp/demo.unibrow"));
    assert!(scene.layers.len() >= 2);
    let has_title = scene.layers.iter().any(|l| {
        matches!(
            &l.kind,
            LiveLayerKind::Text { template } if template.contains("Hello timeline")
        )
    });
    assert!(has_title);
}
