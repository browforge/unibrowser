use std::time::Instant;

use unibrowser_live::{
    LiveMediaSource, MediaFrame, MediaSource, SourceFrameResult, SourceId, SourceTiming,
};

pub struct TestPatternSource {
    id: SourceId,
    width: u32,
    height: u32,
}

impl TestPatternSource {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            id: SourceId("live-proving/test-pattern".into()),
            width: width.max(16),
            height: height.max(16),
        }
    }
}

impl MediaSource for TestPatternSource {
    fn id(&self) -> &SourceId {
        &self.id
    }

    fn timing(&self) -> SourceTiming {
        SourceTiming::live()
    }

    fn next_frame(&mut self, at_sec: f64, _deadline: Instant) -> SourceFrameResult {
        let w = self.width;
        let h = self.height;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        let phase = (at_sec * 2.0).sin();
        for y in 0..h {
            for x in 0..w {
                let bar = ((x as f64 / w as f64) * 6.0 + at_sec * 3.0) as u32 % 6;
                let (r, g, b) = match bar {
                    0 => (235, 235, 235),
                    1 => (235, 235, 16),
                    2 => (16, 235, 235),
                    3 => (16, 235, 16),
                    4 => (235, 16, 235),
                    _ => (235, 16, 16),
                };
                let i = ((y * w + x) * 4) as usize;
                rgba[i] = ((r as f32) * (0.85 + 0.15 * phase as f32)) as u8;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = 255;
            }
        }
        SourceFrameResult::Frame(MediaFrame {
            width: w,
            height: h,
            rgba,
            pts_sec: at_sec,
        })
    }
}

impl LiveMediaSource for TestPatternSource {
    fn live_kind(&self) -> &'static str {
        "test_pattern"
    }
}
