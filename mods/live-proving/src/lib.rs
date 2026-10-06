//! Bundled proving mods for live engine development ([ADR 0024]).
//!
//! Real V4L2, RTMP, MQTT, etc. mods are authored separately; see `docs/live-mod-authoring.md`.

mod sine_temperature;
mod test_pattern;

pub use sine_temperature::SineTemperatureSource;
pub use test_pattern::TestPatternSource;

use unibrowser_live::{DataSource, MediaSource, SourceId};

pub const TEST_PATTERN_ID: &str = "live-proving/test-pattern";
pub const SINE_TEMP_ID: &str = "live-proving/sine-temperature";

/// Register hook for future mod-host live catalog (no palette commands yet).
pub fn register(registry: &mut unibrowser_engine::CommandRegistry) {
    let _ = registry;
    // Manifest `contributes.liveSources` in info.json; host loads descriptors when live UI lands.
}

pub fn test_pattern_source(width: u32, height: u32) -> TestPatternSource {
    TestPatternSource::new(width, height)
}

pub fn sine_temperature_source() -> SineTemperatureSource {
    SineTemperatureSource::default()
}

pub fn media_by_id(
    id: &str,
    width: u32,
    height: u32,
) -> Option<Box<dyn MediaSource>> {
    if id == TEST_PATTERN_ID {
        Some(Box::new(TestPatternSource::new(width, height)))
    } else {
        None
    }
}

pub fn data_by_id(id: &str) -> Option<Box<dyn DataSource>> {
    if id == SINE_TEMP_ID {
        Some(Box::new(SineTemperatureSource::default()))
    } else {
        None
    }
}

pub fn source_id(s: &str) -> SourceId {
    SourceId(s.to_string())
}
