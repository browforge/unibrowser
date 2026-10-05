use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use url::Url;

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
            if let DownloadEvent::Requested { url, destination: _ } = event {
                let url_str = url.to_string();
                let _ = webview.app_handle().emit("unibrowser-download", url_str);
                // Return false to cancel default "save to downloads folder" when platform allows.
                return false;
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
