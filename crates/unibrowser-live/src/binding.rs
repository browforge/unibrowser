//! Bind data streams to layer properties (same model as timeline keyframes).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerProperty {
    /// Substitute into a text layer template (`{value}`).
    TextValue,
    /// Normalised 0..1 chart column.
    ChartScalar { column: u32 },
    Opacity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyBinding {
    pub data_channel: String,
    pub target: LayerProperty,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataBinding {
    pub bindings: Vec<PropertyBinding>,
}

impl DataBinding {
    pub fn number_for_channel(&self, samples: &[(String, f64)], channel: &str) -> Option<f64> {
        samples
            .iter()
            .find(|(c, _)| c == channel)
            .map(|(_, v)| *v)
    }
}
