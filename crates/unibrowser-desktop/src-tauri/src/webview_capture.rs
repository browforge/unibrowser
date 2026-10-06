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

/// PNG snapshot of the main shell webview (`main` window).
pub fn capture_shell_webview_png(app: &AppHandle) -> Result<Vec<u8>, String> {
    let wv = app
        .get_webview("main")
        .ok_or("main shell webview missing")?;
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
                            tx.send(Err("webview snapshot produced empty PNG".into())).ok();
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

    let tmp_root: PathBuf = std::env::temp_dir().join(format!("ub-xcap-root-{}.png", std::process::id()));
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

/// Doc screenshots: composited X11 crop plus WebKitGTK snapshots; pick the least empty frame.
pub fn capture_doc_screenshot_png(app: &AppHandle, window: &Window) -> Result<Vec<u8>, String> {
    let mut candidates: Vec<Vec<u8>> = Vec::new();
    if let Ok(bytes) = capture_composited_root_crop_png(window) {
        candidates.push(bytes);
    }
    if let Ok(bytes) = capture_shell_webview_png(app) {
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
