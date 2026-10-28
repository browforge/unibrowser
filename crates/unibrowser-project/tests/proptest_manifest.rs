use proptest::prelude::*;
use unibrowser_project::{
    validate_manifest, ProjectManifest, ProjectNode, ProjectTimeline, TimelineClip,
};

proptest! {
    #[test]
    fn manifest_json_roundtrip(
        title in "\\PC{3,40}",
        node_id in "[a-z][a-z0-9]{2,8}",
        clip_id in "[a-z][a-z0-9]{2,8}",
    ) {
        let manifest = ProjectManifest {
            format_version: 2,
            title,
            description: None,
            nodes: vec![ProjectNode {
                id: node_id.clone(),
                node_type: "note".into(),
                label: "n".into(),
                asset: None,
                asset_hash: None,
                x: 0.0,
                y: 0.0,
                color: None,
            }],
            timeline: ProjectTimeline {
                clips: vec![TimelineClip {
                    id: clip_id,
                    node_id,
                    start_sec: 0.0,
                    duration_sec: 1.0,
                    label: "clip".into(),
                    color: None,
                    clip_markers: vec![],
                    duck_under_video: false,
                    fade_in_sec: 0.0,
                    fade_out_sec: 0.0,
                    pdf_page: None,
                    title_text: None,
                    title_font_pt: None,
                }],
                transitions: vec![],
                markers: vec![],
            },
        };
        validate_manifest(&manifest).expect("valid");
        let json = serde_json::to_string(&manifest).expect("serialize");
        let back: ProjectManifest = serde_json::from_str(&json).expect("parse");
        prop_assert_eq!(manifest, back);
    }
}
