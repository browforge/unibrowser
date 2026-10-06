use std::collections::HashMap;
use std::time::Instant;

use crate::binding::LayerProperty;
use crate::scene::{LiveLayer, LiveScene};
use crate::source::{DataSource, MediaFrame, MediaSource, SourceFrameResult};

#[derive(Debug, Clone)]
pub struct LiveCompositorConfig {
    pub width: u32,
    pub height: u32,
}

pub struct LiveCompositor {
    pub config: LiveCompositorConfig,
}

impl LiveCompositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            config: LiveCompositorConfig { width, height },
        }
    }

    pub fn render_frame(
        &self,
        scene: &LiveScene,
        at_sec: f64,
        deadline: Instant,
        media: &mut HashMap<String, &mut dyn MediaSource>,
        data: &mut HashMap<String, &mut dyn DataSource>,
    ) -> Result<MediaFrame, String> {
        if deadline <= Instant::now() {
            return Err("composite deadline exceeded (drop frame)".into());
        }
        let w = self.config.width;
        let h = self.config.height;
        let mut rgba = vec![0u8; (w * h * 4) as usize];

        let mut numbers: Vec<(String, f64)> = Vec::new();
        for ds in data.values_mut() {
            for sample in ds.poll(at_sec) {
                if let crate::source::DataValue::Number(n) = sample.value {
                    numbers.push((sample.channel.clone(), n));
                }
            }
        }

        for layer in &scene.layers {
            if let Some(ref sid) = layer.media_source_id {
                if let Some(src) = media.get_mut(sid) {
                    match src.next_frame(at_sec, deadline) {
                        SourceFrameResult::Frame(frame) => {
                            blit_scale(&mut rgba, w, h, &frame);
                        }
                        SourceFrameResult::Dropped { .. } => {}
                        SourceFrameResult::End => {}
                    }
                }
            }
            if layer.chart_bar {
                let scalar = chart_scalar(layer, &numbers);
                draw_chart_bar(&mut rgba, w, h, scalar);
            }
            if let Some(ref tpl) = layer.text_template {
                let text = resolve_template(tpl, layer, &numbers);
                draw_text_overlay(&mut rgba, w, h, &text, 8, 8);
            }
        }

        Ok(MediaFrame {
            width: w,
            height: h,
            rgba,
            pts_sec: at_sec,
        })
    }
}

fn chart_scalar(layer: &LiveLayer, numbers: &[(String, f64)]) -> f32 {
    for b in &layer.bindings.bindings {
        if matches!(b.target, LayerProperty::ChartScalar { .. }) {
            if let Some(v) = numbers
                .iter()
                .find(|(c, _)| c == &b.data_channel)
                .map(|(_, n)| *n)
            {
                // Map 0..40 C to 0..1 for proving sine ~15..25
                return ((v / 40.0).clamp(0.0, 1.0)) as f32;
            }
        }
    }
    0.0
}

fn resolve_template(tpl: &str, layer: &LiveLayer, numbers: &[(String, f64)]) -> String {
    let mut value = String::from("—");
    for b in &layer.bindings.bindings {
        if b.target == LayerProperty::TextValue {
            if let Some(v) = numbers
                .iter()
                .find(|(c, _)| c == &b.data_channel)
                .map(|(_, n)| *n)
            {
                value = format!("{:.1}", v);
            }
        }
    }
    tpl.replace("{value}", &value)
}

fn blit_scale(dst: &mut [u8], dw: u32, dh: u32, src: &MediaFrame) {
    if src.width == 0 || src.height == 0 {
        return;
    }
    for y in 0..dh {
        for x in 0..dw {
            let sx = x * src.width / dw;
            let sy = y * src.height / dh;
            let si = ((sy * src.width + sx) * 4) as usize;
            if si + 3 >= src.rgba.len() {
                continue;
            }
            let di = ((y * dw + x) * 4) as usize;
            if di + 3 >= dst.len() {
                continue;
            }
            dst[di..di + 4].copy_from_slice(&src.rgba[si..si + 4]);
        }
    }
}

fn draw_chart_bar(rgba: &mut [u8], w: u32, h: u32, scalar: f32) {
    let bar_w = 24u32;
    let x0 = w - bar_w - 8;
    let bar_h = ((h as f32 - 16.0) * scalar) as u32;
    let y0 = h - 8 - bar_h;
    for y in y0..h - 8 {
        for x in x0..x0 + bar_w {
            let i = ((y * w + x) * 4) as usize;
            if i + 3 < rgba.len() {
                rgba[i] = 40;
                rgba[i + 1] = 180;
                rgba[i + 2] = 90;
                rgba[i + 3] = 255;
            }
        }
    }
}

fn draw_text_overlay(rgba: &mut [u8], w: u32, _h: u32, text: &str, x0: u32, y0: u32) {
    let mut x = x0;
    for ch in text.chars().take(24) {
        draw_char(rgba, w, ch, x, y0);
        x += 9;
    }
}

fn draw_char(rgba: &mut [u8], w: u32, ch: char, x0: u32, y0: u32) {
    let glyph = glyph_5x7(ch);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                for dy in 0..2 {
                    for dx in 0..2 {
                        let x = x0 + col as u32 * 2 + dx;
                        let y = y0 + row as u32 * 2 + dy;
                        let i = ((y * w + x) * 4) as usize;
                        if i + 3 < rgba.len() {
                            rgba[i] = 240;
                            rgba[i + 1] = 240;
                            rgba[i + 2] = 240;
                            rgba[i + 3] = 255;
                        }
                    }
                }
            }
        }
    }
}

fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x06, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        ' ' => [0; 7],
        _ => [0x1F, 0x11, 0x15, 0x15, 0x15, 0x11, 0x1F],
    }
}
