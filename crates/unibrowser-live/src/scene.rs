use serde::{Deserialize, Serialize};

use crate::binding::DataBinding;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveLayer {
    pub id: String,
    /// Media source id when this layer shows video/raster.
    pub media_source_id: Option<String>,
    /// Text overlay template; `{value}` filled from `DataBinding`.
    pub text_template: Option<String>,
    pub bindings: DataBinding,
    /// Chart bar uses bound scalar 0..1 vertically.
    pub chart_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveScene {
    pub id: SceneId,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LiveLayer>,
}

impl LiveScene {
    pub fn proving_default() -> Self {
        Self {
            id: SceneId("proving".into()),
            width: 320,
            height: 180,
            layers: vec![
                LiveLayer {
                    id: "pattern".into(),
                    media_source_id: Some("live-proving/test-pattern".into()),
                    text_template: None,
                    bindings: DataBinding::default(),
                    chart_bar: false,
                },
                LiveLayer {
                    id: "temp-text".into(),
                    media_source_id: None,
                    text_template: Some("{value} C".into()),
                    bindings: DataBinding {
                        bindings: vec![crate::binding::PropertyBinding {
                            data_channel: "temperature".into(),
                            target: crate::binding::LayerProperty::TextValue,
                        }],
                    },
                    chart_bar: false,
                },
                LiveLayer {
                    id: "temp-chart".into(),
                    media_source_id: None,
                    text_template: None,
                    bindings: DataBinding {
                        bindings: vec![crate::binding::PropertyBinding {
                            data_channel: "temperature".into(),
                            target: crate::binding::LayerProperty::ChartScalar { column: 0 },
                        }],
                    },
                    chart_bar: true,
                },
            ],
        }
    }
}
