//! HTML/JS layer snapshot path (offscreen webview in desktop; CPU stub for tests).

use crate::compositor::draw_text_overlay_rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HtmlSnapshotKind {
    /// Speedrun-style elapsed timer (MM:SS.cs).
    SpeedrunClock,
}

pub fn render_html_snapshot_rgba(
    kind: HtmlSnapshotKind,
    width: u32,
    height: u32,
    at_sec: f64,
) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    match kind {
        HtmlSnapshotKind::SpeedrunClock => {
            let panel_w = width.min(120);
            let panel_h = 28u32;
            let x0 = width - panel_w - 8;
            let y0 = 8u32;
            for y in y0..y0 + panel_h {
                for x in x0..x0 + panel_w {
                    if x >= width || y >= height {
                        continue;
                    }
                    let i = ((y * width + x) * 4) as usize;
                    rgba[i] = 20;
                    rgba[i + 1] = 20;
                    rgba[i + 2] = 28;
                    rgba[i + 3] = 220;
                }
            }
            let mins = (at_sec / 60.0).floor() as u32;
            let secs = at_sec % 60.0;
            let label = format!("{:02}:{:05.2}", mins, secs);
            draw_text_overlay_rgba(&mut rgba, width, height, &label, x0 + 6, y0 + 6);
        }
    }
    rgba
}
