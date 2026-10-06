use serde::{Deserialize, Serialize};
use unibrowser_project::LayerTransform;

use crate::binding::DataBinding;
use crate::html_snapshot::HtmlSnapshotKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveLayerKind {
    /// Full-frame or transformed live/file media source.
    Media { source_id: String },
    /// SVG overlay (resvg), path relative to project or absolute.
    SvgOverlay { path: String },
    /// HTML/JS snapshot (webview in shell; stub in engine tests).
    HtmlSnapshot { kind: HtmlSnapshotKind },
    /// Text template with data bindings.
    Text { template: String },
    /// Scalar chart bar from data binding.
    Chart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveLayer {
    pub id: String,
    pub kind: LiveLayerKind,
    pub transform: LayerTransform,
    pub bindings: DataBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveScene {
    pub id: SceneId,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LiveLayer>,
}

impl LiveScene {
    pub fn proving_full_stack(svg_overlay_path: String) -> Self {
        Self {
            id: SceneId("proving-full".into()),
            width: 320,
            height: 180,
            layers: vec![
                LiveLayer {
                    id: "pattern".into(),
                    kind: LiveLayerKind::Media {
                        source_id: "live-proving/test-pattern".into(),
                    },
                    transform: LayerTransform::fill(),
                    bindings: DataBinding::default(),
                },
                LiveLayer {
                    id: "svg-badge".into(),
                    kind: LiveLayerKind::SvgOverlay {
                        path: svg_overlay_path,
                    },
                    transform: LayerTransform {
                        offset_x: 8.0,
                        offset_y: 8.0,
                        scale_x: 0.5,
                        scale_y: 0.5,
                        crop: None,
                    },
                    bindings: DataBinding::default(),
                },
                LiveLayer {
                    id: "clock".into(),
                    kind: LiveLayerKind::HtmlSnapshot {
                        kind: HtmlSnapshotKind::SpeedrunClock,
                    },
                    transform: LayerTransform::fill(),
                    bindings: DataBinding::default(),
                },
                LiveLayer {
                    id: "temp-text".into(),
                    kind: LiveLayerKind::Text {
                        template: "{value} C".into(),
                    },
                    transform: LayerTransform::with_offset(8.0, 140.0),
                    bindings: DataBinding {
                        bindings: vec![crate::binding::PropertyBinding {
                            data_channel: "temperature".into(),
                            target: crate::binding::LayerProperty::TextValue,
                        }],
                    },
                },
                LiveLayer {
                    id: "temp-chart".into(),
                    kind: LiveLayerKind::Chart,
                    transform: LayerTransform {
                        offset_x: 280.0,
                        offset_y: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        crop: None,
                    },
                    bindings: DataBinding {
                        bindings: vec![crate::binding::PropertyBinding {
                            data_channel: "temperature".into(),
                            target: crate::binding::LayerProperty::ChartScalar { column: 0 },
                        }],
                    },
                },
            ],
        }
    }

    /// Smaller scene for quick tests (pattern + data overlays only).
    pub fn proving_default() -> Self {
        Self {
            id: SceneId("proving".into()),
            width: 320,
            height: 180,
            layers: vec![
                LiveLayer {
                    id: "pattern".into(),
                    kind: LiveLayerKind::Media {
                        source_id: "live-proving/test-pattern".into(),
                    },
                    transform: LayerTransform::fill(),
                    bindings: DataBinding::default(),
                },
                LiveLayer {
                    id: "temp-text".into(),
                    kind: LiveLayerKind::Text {
                        template: "{value} C".into(),
                    },
                    transform: LayerTransform::with_offset(8.0, 8.0),
                    bindings: DataBinding {
                        bindings: vec![crate::binding::PropertyBinding {
                            data_channel: "temperature".into(),
                            target: crate::binding::LayerProperty::TextValue,
                        }],
                    },
                },
                LiveLayer {
                    id: "temp-chart".into(),
                    kind: LiveLayerKind::Chart,
                    transform: LayerTransform::fill(),
                    bindings: DataBinding {
                        bindings: vec![crate::binding::PropertyBinding {
                            data_channel: "temperature".into(),
                            target: crate::binding::LayerProperty::ChartScalar { column: 0 },
                        }],
                    },
                },
            ],
        }
    }
}
