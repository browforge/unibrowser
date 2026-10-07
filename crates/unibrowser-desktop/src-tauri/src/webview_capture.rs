//! Native https content-webview PNG capture (Linux WebKitGTK).
//! Doc screenshots use composited X11 root capture (child GDK webviews are not in the shell snapshot).

use tauri::{AppHandle, Manager, Window};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WebCaptureStatus {
    pub native_available: bool,
    pub detail: String,
}

pub fn web_capture_status() -> WebCaptureStatus {
    #[cfg(target_os = "linux")]
    {
        return WebCaptureStatus {
            native_available: true,
            detail: "Linux WebKitGTK visible-region snapshot (WRY content webview)".into(),
        };
    }
    #[cfg(not(target_os = "linux"))]
    {
        WebCaptureStatus {
            native_available: false,
            detail:
                "HTTPS tab capture needs Linux WebKitGTK. On other platforms use image export or shell canvas fallback on viewers."
                    .into(),
        }
    }
}

/// PNG snapshot of a shell webview by window label (`main` or `toolbar-*`).
pub fn capture_shell_webview_png_for_label(
    app: &AppHandle,
    window_label: &str,
) -> Result<Vec<u8>, String> {
    let wv = app
        .get_webview(window_label)
        .ok_or_else(|| format!("shell webview missing: {}", window_label))?;
    #[cfg(target_os = "linux")]
    {
        return capture_linux_webview(&wv);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = wv;
        Err("shell webview capture only implemented on Linux WebKitGTK".into())
    }
}

/// PNG snapshot of the main shell webview (`main` window).
pub fn capture_shell_webview_png(app: &AppHandle) -> Result<Vec<u8>, String> {
    capture_shell_webview_png_for_label(app, "main")
}

pub fn capture_content_webview_png(app: &AppHandle, window_label: &str) -> Result<Vec<u8>, String> {
    crate::webview::ensure_content_webview(app, window_label)?;
    let label = crate::webview::content_label_for_capture(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;

    #[cfg(target_os = "linux")]
    {
        return capture_linux_webview(&wv);
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = wv;
        Err(web_capture_status().detail)
    }
}

#[cfg(target_os = "linux")]
fn capture_linux_webview(wv: &tauri::Webview) -> Result<Vec<u8>, String> {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let (tx, rx) = mpsc::sync_channel::<Result<Vec<u8>, String>>(1);
    wv.with_webview(move |platform| {
        use gio::Cancellable;
        use webkit2gtk::WebViewExt;
        use webkit2gtk::{SnapshotOptions, SnapshotRegion};
        let webview = platform.inner();
        let options = SnapshotOptions::empty();
        webview.snapshot(
            SnapshotRegion::Visible,
            options,
            None::<&Cancellable>,
            move |result| match result {
                Ok(surface) => {
                    let mut bytes = Vec::new();
                    match surface.write_to_png(&mut bytes) {
                        Ok(()) if bytes.is_empty() => {
                            tx.send(Err("webview snapshot produced empty PNG".into()))
                                .ok();
                        }
                        Ok(()) => {
                            tx.send(Ok(bytes)).ok();
                        }
                        Err(e) => {
                            tx.send(Err(e.to_string())).ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(Err(e.to_string())).ok();
                }
            },
        );
    })
    .map_err(|e| e.to_string())?;

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(result) = rx.try_recv() {
            return result;
        }
        while gtk::events_pending() {
            gtk::main_iteration();
        }
        std::thread::sleep(Duration::from_millis(12));
    }
    Err("webview snapshot timed out (no display or page not ready)".into())
}

/// Full X11 root pixmap cropped to the main window outer bounds (includes native child webviews).
#[cfg(target_os = "linux")]
pub fn capture_composited_root_crop_png(window: &Window) -> Result<Vec<u8>, String> {
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_millis(400);
    while Instant::now() < deadline {
        while gtk::events_pending() {
            gtk::main_iteration();
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let x = pos.x.max(0);
    let y = pos.y.max(0);
    let w = size.width.max(1);
    let h = size.height.max(1);
    let crop = format!("{}x{}+{}+{}", w, h, x, y);

    let tmp_root: PathBuf =
        std::env::temp_dir().join(format!("ub-xcap-root-{}.png", std::process::id()));
    let root_arg = tmp_root.to_string_lossy();
    let import_status = Command::new("import")
        .args(["-window", "root", root_arg.as_ref()])
        .status()
        .map_err(|e| format!("import not available: {e}"))?;
    if !import_status.success() {
        return Err("import -window root failed (install ImageMagick import)".into());
    }

    let output = Command::new("convert")
        .args([root_arg.as_ref(), "-crop", &crop, "png:-"])
        .output()
        .map_err(|e| format!("convert crop failed: {e}"))?;
    let _ = std::fs::remove_file(&tmp_root);
    if !output.status.success() {
        return Err(format!(
            "convert crop failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout.len() < 256 {
        return Err("composited capture produced empty PNG".into());
    }
    Ok(output.stdout)
}

#[cfg(not(target_os = "linux"))]
pub fn capture_composited_root_crop_png(_window: &Window) -> Result<Vec<u8>, String> {
    Err("composited X11 capture is only implemented on Linux".into())
}

/// Doc screenshots: prefer the shell WebKitGTK snapshot (Monaco lives in the main webview).
/// Composited X11 root crops can miss GPU-layered editor paint on WebKitGTK.
pub fn capture_doc_screenshot_png(app: &AppHandle, window: &Window) -> Result<Vec<u8>, String> {
    // Prefer composited X11 root when it paints the editor brighter than the shell snapshot
    // (WebKitGTK shell snapshots often flatten Monaco to a dark rectangle under xvfb).
    if let Ok(root_bytes) = capture_composited_root_crop_png(window) {
        if let Ok(shell_bytes) = capture_shell_webview_png(app) {
            if editor_region_luma(&root_bytes) > editor_region_luma(&shell_bytes) + 0.08 {
                return Ok(root_bytes);
            }
        } else if editor_region_luma(&root_bytes) > 0.35 {
            return Ok(root_bytes);
        }
    }
    if let Ok(bytes) = capture_shell_webview_png(app) {
        if score_png_informativeness(&bytes) > 100 {
            return Ok(bytes);
        }
    }
    let mut candidates: Vec<Vec<u8>> = Vec::new();
    if let Ok(bytes) = capture_shell_webview_png(app) {
        candidates.push(bytes);
    }
    if let Ok(bytes) = capture_composited_root_crop_png(window) {
        candidates.push(bytes);
    }
    if let Ok(bytes) = capture_content_webview_png(app, window.label()) {
        candidates.push(bytes);
    }
    if candidates.is_empty() {
        return Err("no doc capture candidates succeeded".into());
    }
    Ok(candidates
        .into_iter()
        .max_by_key(|b| score_png_informativeness(b))
        .expect("candidates non-empty"))
}

/// Mean luma (0..1) of the central document band (where Monaco usually lives).
#[cfg(target_os = "linux")]
fn editor_region_luma(png: &[u8]) -> f64 {
    use image::GenericImageView;
    match image::load_from_memory(png) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            if w < 32 || h < 32 {
                return 0.0;
            }
            let x0 = (w as f64 * 0.14) as u32;
            let x1 = (w as f64 * 0.62) as u32;
            let y0 = (h as f64 * 0.18) as u32;
            let y1 = (h as f64 * 0.78) as u32;
            let step_x = ((x1 - x0) / 32).max(1);
            let step_y = ((y1 - y0) / 32).max(1);
            let mut sum = 0.0;
            let mut n = 0u64;
            for y in (y0..y1).step_by(step_y as usize) {
                for x in (x0..x1).step_by(step_x as usize) {
                    let p = img.get_pixel(x, y);
                    sum += (p[0] as f64 + p[1] as f64 + p[2] as f64) / (3.0 * 255.0);
                    n += 1;
                }
            }
            if n == 0 {
                0.0
            } else {
                sum / n as f64
            }
        }
        Err(_) => 0.0,
    }
}

#[cfg(not(target_os = "linux"))]
fn editor_region_luma(_png: &[u8]) -> f64 {
    0.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorTextPixelStats {
    pub sampled_pixels: u64,
    pub background_pixels: u64,
    pub ink_pixels: u64,
    pub rows_with_ink: u32,
}

pub fn validate_editor_text_pixels(
    png: &[u8],
    css_rect: (f64, f64, f64, f64),
    css_viewport: (f64, f64),
    dark: bool,
) -> Result<EditorTextPixelStats, String> {
    use image::GenericImageView;

    let img = image::load_from_memory(png).map_err(|e| e.to_string())?;
    let (png_w, png_h) = img.dimensions();
    let (viewport_w, viewport_h) = css_viewport;
    if viewport_w <= 0.0 || viewport_h <= 0.0 {
        return Err("editor pixel gate received an invalid viewport".into());
    }
    let scale_x = png_w as f64 / viewport_w;
    let scale_y = png_h as f64 / viewport_h;
    let (x, y, width, height) = css_rect;
    let x0 = ((x + 48.0) * scale_x).clamp(0.0, png_w as f64) as u32;
    let x1 = ((x + width - 16.0) * scale_x).clamp(0.0, png_w as f64) as u32;
    let y0 = ((y + 4.0) * scale_y).clamp(0.0, png_h as f64) as u32;
    let y1 = ((y + height.min(320.0)) * scale_y).clamp(0.0, png_h as f64) as u32;
    if x1 <= x0 || y1 <= y0 {
        return Err("editor pixel gate received an empty editor region".into());
    }

    let expected = if dark {
        [0x23i32, 0x25i32, 0x26i32]
    } else {
        [0xffi32, 0xffi32, 0xffi32]
    };
    let mut stats = EditorTextPixelStats {
        sampled_pixels: 0,
        background_pixels: 0,
        ink_pixels: 0,
        rows_with_ink: 0,
    };
    for py in y0..y1 {
        let mut row_ink = 0u32;
        for px in x0..x1 {
            let p = img.get_pixel(px, py);
            let dr = p[0] as i32 - expected[0];
            let dg = p[1] as i32 - expected[1];
            let db = p[2] as i32 - expected[2];
            let distance_sq = dr * dr + dg * dg + db * db;
            stats.sampled_pixels += 1;
            if distance_sq <= 18 * 18 * 3 {
                stats.background_pixels += 1;
            } else if distance_sq >= 34 * 34 * 3 {
                stats.ink_pixels += 1;
                row_ink += 1;
            }
        }
        if row_ink >= 3 {
            stats.rows_with_ink += 1;
        }
    }

    let background_ratio = stats.background_pixels as f64 / stats.sampled_pixels as f64;
    let ink_ratio = stats.ink_pixels as f64 / stats.sampled_pixels as f64;
    if background_ratio < 0.55 {
        return Err(format!(
            "editor pixel gate: expected theme background is not visible (background ratio {:.3})",
            background_ratio
        ));
    }
    if stats.ink_pixels < 100 || stats.rows_with_ink < 12 || ink_ratio < 0.0005 {
        return Err(format!(
            "editor pixel gate: visible text pixels missing (ink={}, rows={}, ratio={:.5})",
            stats.ink_pixels, stats.rows_with_ink, ink_ratio
        ));
    }
    Ok(stats)
}

#[cfg(target_os = "linux")]
fn score_png_informativeness(png: &[u8]) -> u64 {
    use image::GenericImageView;
    match image::load_from_memory(png) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            if w == 0 || h == 0 {
                return 0;
            }
            let step_x = (w / 48).max(1);
            let step_y = (h / 48).max(1);
            let mut bright = 0u64;
            let mut samples = 0u64;
            for y in (0..h).step_by(step_y as usize) {
                for x in (0..w).step_by(step_x as usize) {
                    let p = img.get_pixel(x, y);
                    let luma = (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3;
                    if luma > 28 && luma < 240 {
                        bright += 1;
                    }
                    samples += 1;
                }
            }
            bright * 1_000_000 + samples
        }
        Err(_) => png.len() as u64,
    }
}

#[cfg(not(target_os = "linux"))]
fn score_png_informativeness(png: &[u8]) -> u64 {
    png.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn png_with_text(light: bool, include_text: bool) -> Vec<u8> {
        let bg = if light {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0x23, 0x25, 0x26, 255])
        };
        let ink = if light {
            Rgba([31, 35, 38, 255])
        } else {
            Rgba([240, 239, 235, 255])
        };
        let mut image = RgbaImage::from_pixel(100, 100, bg);
        if include_text {
            for line in 0..10 {
                let y = 8 + line * 7;
                for py in y..y + 2 {
                    for x in 54..72 {
                        image.put_pixel(x, py, ink);
                    }
                }
            }
        }
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn editor_pixel_gate_accepts_light_and_dark_text() {
        for dark in [false, true] {
            let bytes = png_with_text(!dark, true);
            let stats =
                validate_editor_text_pixels(&bytes, (0.0, 0.0, 100.0, 100.0), (100.0, 100.0), dark)
                    .unwrap();
            assert!(stats.ink_pixels >= 100);
            assert!(stats.rows_with_ink >= 12);
        }
    }

    #[test]
    fn editor_pixel_gate_rejects_blank_surface() {
        let bytes = png_with_text(true, false);
        let err =
            validate_editor_text_pixels(&bytes, (0.0, 0.0, 100.0, 100.0), (100.0, 100.0), false)
                .unwrap_err();
        assert!(err.contains("visible text pixels missing"));
    }

    #[test]
    fn editor_pixel_gate_rejects_wrong_surface_colour() {
        let bytes = png_with_text(false, false);
        let err =
            validate_editor_text_pixels(&bytes, (0.0, 0.0, 100.0, 100.0), (100.0, 100.0), false)
                .unwrap_err();
        assert!(err.contains("background is not visible"));
    }
}
