//! Frame clock for live mode (target fps, drop-frame policy).

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct DropFramePolicy {
    /// Never block longer than this on a single frame composite.
    pub max_composite_ms: u64,
    /// When behind, skip ticks instead of catching up (keeps UI responsive).
    pub skip_catch_up: bool,
}

impl Default for DropFramePolicy {
    fn default() -> Self {
        Self {
            max_composite_ms: 12,
            skip_catch_up: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameClock {
    pub target_fps: f32,
    pub policy: DropFramePolicy,
    frame_index: u64,
    started: Instant,
}

impl FrameClock {
    pub fn new(target_fps: f32) -> Self {
        Self {
            target_fps: target_fps.max(1.0),
            policy: DropFramePolicy::default(),
            frame_index: 0,
            started: Instant::now(),
        }
    }

    pub fn frame_duration(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.target_fps as f64)
    }

    pub fn tick(&mut self) -> FrameTick {
        let idx = self.frame_index;
        self.frame_index += 1;
        let t_sec = idx as f64 / self.target_fps as f64;
        FrameTick {
            index: idx,
            t_sec,
            deadline: Instant::now() + Duration::from_millis(self.policy.max_composite_ms),
        }
    }

    pub fn elapsed_since_start(&self) -> Duration {
        self.started.elapsed()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FrameTick {
    pub index: u64,
    pub t_sec: f64,
    pub deadline: Instant,
}
