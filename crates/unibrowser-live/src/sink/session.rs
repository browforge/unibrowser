use std::collections::HashMap;
use std::path::Path;

use crate::clock::FrameClock;
use crate::compositor::LiveCompositor;
use crate::scene::LiveScene;
use crate::sink::file::{ffprobe_duration_sec, FileVideoSink, SinkError};
use crate::source::{DataSource, MediaSource};

#[derive(Debug, Clone)]
pub struct LiveSessionReport {
    pub frames_written: u64,
    pub frames_dropped: u64,
    pub output_path: String,
    pub duration_sec: Option<f64>,
}

/// Run live-mode compositor for a fixed frame count into a file sink (proving slice).
pub fn run_live_session_to_file(
    scene: &LiveScene,
    fps: f32,
    frame_count: u64,
    dest: &Path,
    media: &mut HashMap<String, &mut dyn MediaSource>,
    data: &mut HashMap<String, &mut dyn DataSource>,
) -> Result<LiveSessionReport, SinkError> {
    let compositor = LiveCompositor::new(scene.width, scene.height);
    let mut clock = FrameClock::new(fps);
    let mut sink = FileVideoSink::open(dest, scene.width, scene.height, fps)?;
    let mut written = 0u64;
    let mut dropped = 0u64;

    for _ in 0..frame_count {
        let tick = clock.tick();
        match compositor.render_frame(scene, tick.t_sec, tick.deadline, media, data) {
            Ok(frame) => {
                sink.write_frame(&frame)?;
                written += 1;
            }
            Err(_) => {
                dropped += 1;
            }
        }
    }

    let path = sink.finish()?;
    let duration_sec = ffprobe_duration_sec(&path);
    Ok(LiveSessionReport {
        frames_written: written,
        frames_dropped: dropped,
        output_path: path.to_string_lossy().into_owned(),
        duration_sec,
    })
}
