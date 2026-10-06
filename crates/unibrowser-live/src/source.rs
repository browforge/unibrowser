//! Source abstraction: file (seekable) vs live (clock-driven).

use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingMode {
    /// Seekable media on disk (timeline clip / asset).
    File,
    /// Clock-driven; unseekable; may drop frames under load.
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SourceTiming {
    pub mode: TimingMode,
    /// When `mode == File`, duration if known.
    pub duration_sec: Option<f64>,
}

impl SourceTiming {
    pub fn file(duration_sec: Option<f64>) -> Self {
        Self {
            mode: TimingMode::File,
            duration_sec,
        }
    }

    pub fn live() -> Self {
        Self {
            mode: TimingMode::Live,
            duration_sec: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaFrame {
    pub width: u32,
    pub height: u32,
    /// RGBA8 row-major.
    pub rgba: Vec<u8>,
    pub pts_sec: f64,
}

#[derive(Debug, Clone)]
pub enum SourceFrameResult {
    Frame(MediaFrame),
    /// Live path: skip this tick rather than block the UI thread.
    Dropped { reason: String },
    End,
}

/// Video or raster layer input (file or live).
pub trait MediaSource: Send {
    fn id(&self) -> &SourceId;
    fn timing(&self) -> SourceTiming;
    /// `deadline` is a hint for live sources; file sources may ignore it.
    fn next_frame(&mut self, at_sec: f64, deadline: Instant) -> SourceFrameResult;
}

/// Marker for live-only media (camera, test pattern, …).
pub trait LiveMediaSource: MediaSource {
    fn live_kind(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValue {
    Number(f64),
    Text(String),
    Event(String),
}

#[derive(Debug, Clone)]
pub struct DataSample {
    pub channel: String,
    pub value: DataValue,
    pub pts_sec: f64,
}

/// Typed data stream (IoT, chat, MIDI CC, key events, …).
pub trait DataSource: Send {
    fn id(&self) -> &SourceId;
    fn timing(&self) -> SourceTiming;
    fn poll(&mut self, at_sec: f64) -> Vec<DataSample>;
}
