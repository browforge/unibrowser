//! Native https content-webview PNG capture (Linux WebKitGTK).

use tauri::{AppHandle, Manager};

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
