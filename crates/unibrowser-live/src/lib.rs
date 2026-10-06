//! Live compositor engine: sources, bindings, frame clock, sinks ([ADR 0024]).
//!
//! Real device/network capture mods are out of scope for the proving slice;
//! see `mods/live-proving` and [live-mod-authoring.md].

mod binding;
mod clock;
mod compositor;
mod html_snapshot;
mod scene;
mod sink;
mod source;

pub use binding::{DataBinding, LayerProperty, PropertyBinding};
pub use clock::{DropFramePolicy, FrameClock, FrameTick};
pub use compositor::{LiveCompositor, LiveCompositorConfig};
pub use html_snapshot::{HtmlSnapshotKind, render_html_snapshot_rgba};
pub use scene::{LiveLayer, LiveLayerKind, LiveScene, SceneId};
pub use sink::{FileVideoSink, LiveSessionReport, SinkKind};
pub use source::{
    DataSample, DataSource, DataValue, LiveMediaSource, MediaFrame, MediaSource,
    SourceFrameResult, SourceId, SourceTiming, TimingMode,
};

pub use sink::session::run_live_session_to_file;
