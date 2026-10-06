//! Crop / scale / position for timeline and live compositor layers (ADR 0024).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct NormalizedCrop {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LayerTransform {
    /// Top-left placement in output pixels.
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<NormalizedCrop>,
}

impl Default for LayerTransform {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            crop: None,
        }
    }
}

impl LayerTransform {
    pub fn fill() -> Self {
        Self::default()
    }

    pub fn with_offset(x: f32, y: f32) -> Self {
        Self {
            offset_x: x,
            offset_y: y,
            scale_x: 1.0,
            scale_y: 1.0,
            crop: None,
        }
    }
}

/// Blit RGBA `src` (sw×sh) into `dst` (dw×dh) using transform (shared live + export path).
pub fn blit_rgba_with_transform(
    dst: &mut [u8],
    dw: u32,
    dh: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    transform: &LayerTransform,
) {
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return;
    }
    let (crop_x, crop_y, crop_w, crop_h) = match transform.crop {
        Some(c) => (
            (c.x.clamp(0.0, 1.0) * sw as f32) as u32,
            (c.y.clamp(0.0, 1.0) * sh as f32) as u32,
            (c.w.clamp(0.01, 1.0) * sw as f32) as u32,
            (c.h.clamp(0.01, 1.0) * sh as f32) as u32,
        ),
        None => (0, 0, sw, sh),
    };
    let crop_w = crop_w.min(sw - crop_x);
    let crop_h = crop_h.min(sh - crop_y);
    if crop_w == 0 || crop_h == 0 {
        return;
    }
    let out_w = ((crop_w as f32) * transform.scale_x).max(1.0) as u32;
    let out_h = ((crop_h as f32) * transform.scale_y).max(1.0) as u32;
    let ox = transform.offset_x.round() as i32;
    let oy = transform.offset_y.round() as i32;

    for dy in 0..out_h {
        for dx in 0..out_w {
            let sx = crop_x + dx * crop_w / out_w;
            let sy = crop_y + dy * crop_h / out_h;
            let si = ((sy * sw + sx) * 4) as usize;
            if si + 3 >= src.len() {
                continue;
            }
            let x = ox + dx as i32;
            let y = oy + dy as i32;
            if x < 0 || y < 0 || x >= dw as i32 || y >= dh as i32 {
                continue;
            }
            let di = ((y as u32 * dw + x as u32) * 4) as usize;
            if di + 3 >= dst.len() {
                continue;
            }
            let alpha = src[si + 3] as f32 / 255.0;
            if alpha <= 0.001 {
                continue;
            }
            if alpha >= 0.999 {
                dst[di..di + 4].copy_from_slice(&src[si..si + 4]);
            } else {
                for c in 0..3 {
                    let bg = dst[di + c] as f32;
                    let fg = src[si + c] as f32;
                    dst[di + c] = (bg * (1.0 - alpha) + fg * alpha) as u8;
                }
                dst[di + 3] = 255;
            }
        }
    }
}
