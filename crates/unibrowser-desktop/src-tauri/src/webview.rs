use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use url::Url;

use crate::downloads::DownloadEventPayload;

const CONTENT_LABEL: &str = "content";

pub fn ensure_content_webview(app: &AppHandle) -> Result<(), String> {
    if app.get_webview(CONTENT_LABEL).is_some() {
        return Ok(());
    }
    let window = app.get_window("main").ok_or("main window missing")?;
    let blank: Url = "about:blank".parse().map_err(|e: url::ParseError| e.to_string())?;
    let builder = tauri::webview::WebviewBuilder::new(CONTENT_LABEL, WebviewUrl::External(blank))
        .on_download(|webview, event| {
            use tauri::webview::DownloadEvent;
            match event {
                DownloadEvent::Requested { url, destination } => {
                    let payload = DownloadEventPayload {
                        url: url.to_string(),
                        destination: Some(destination.to_string_lossy().into_owned()),
                    };
                    let _ = webview
                        .app_handle()
                        .emit("unibrowser-download", payload);
                    // Cancel WebKitGTK default save when the shell handles disposition.
                    return false;
                }
                DownloadEvent::Finished {
                    path: Some(path),
                    success: true,
                    ..
                } => {
                    let file_url = format!("file://{}", path.display());
                    let payload = DownloadEventPayload {
                        url: file_url,
                        destination: Some(path.to_string_lossy().into_owned()),
                    };
                    let _ = webview.app_handle().emit("unibrowser-download", payload);
                    return false;
                }
                _ => {}
            }
            true
        });
    window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 140.0),
            LogicalSize::new(800.0, 600.0),
        )
        .map_err(|e| e.to_string())?;
    if let Some(wv) = app.get_webview(CONTENT_LABEL) {
        let _ = wv.hide();
    }
    Ok(())
}

pub fn sync_content_bounds(
    app: &AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    ensure_content_webview(app)?;
    let wv = app.get_webview(CONTENT_LABEL).ok_or("content webview missing")?;
    wv.set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    wv.set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Apply absolute root font size via injected user stylesheet.
pub fn inject_page_font_px(app: &AppHandle, px: u8) -> Result<(), String> {
    ensure_content_webview(app)?;
    let wv = app.get_webview(CONTENT_LABEL).ok_or("content webview missing")?;
    let script = format!(
        "(function(){{try{{if(!document.head){{return;}}var s=document.getElementById('unibrowser-user-style');if(!s){{s=document.createElement('style');s.id='unibrowser-user-style';document.head.appendChild(s);}}s.dataset.ubFont='{px}';s.textContent='html{{font-size:{px}px!important;}}';}}catch(e){{}}}})();",
        px = px
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

/// Adjust font for a page URL, persist per origin, and inject.
pub fn adjust_page_font_for_url(app: &AppHandle, url: &str, delta: i32) -> Result<u8, String> {
    let px = crate::web_font_prefs::adjust_origin_font_px(url, delta)?;
    inject_page_font_px(app, px)?;
    Ok(px)
}

pub fn apply_persisted_web_font(app: &AppHandle, url: &str) -> Result<(), String> {
    let px = crate::web_font_prefs::get_origin_font_px(url);
    inject_page_font_px(app, px)?;
    Ok(())
}

pub fn content_webview_navigate(app: &AppHandle, url: &str) -> Result<(), String> {
    ensure_content_webview(app)?;
    let wv = app.get_webview(CONTENT_LABEL).ok_or("content webview missing")?;
    if url == "about:blank" {
        wv.hide().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let parsed: Url = url.parse().map_err(|e: url::ParseError| e.to_string())?;
    wv.navigate(parsed).map_err(|e| e.to_string())?;
    wv.show().map_err(|e| e.to_string())?;
    Ok(())
}
