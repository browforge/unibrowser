use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use url::Url;

use crate::downloads::DownloadEventPayload;

fn content_label(window_label: &str) -> String {
    format!("content-{}", window_label)
}

pub fn ensure_content_webview(app: &AppHandle, window_label: &str) -> Result<(), String> {
    let label = content_label(window_label);
    if app.get_webview(&label).is_some() {
        return Ok(());
    }
    let window = app
        .get_window(window_label)
        .ok_or_else(|| format!("window {} missing", window_label))?;
    let blank: Url = "about:blank".parse().map_err(|e: url::ParseError| e.to_string())?;
    let builder = tauri::webview::WebviewBuilder::new(&label, WebviewUrl::External(blank))
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
    if let Some(wv) = app.get_webview(&label) {
        let _ = wv.hide();
    }
    Ok(())
}

pub fn sync_content_bounds(
    app: &AppHandle,
    window_label: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    wv.set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    wv.set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Apply absolute root font size via injected user stylesheet.
pub fn inject_page_font_px(app: &AppHandle, window_label: &str, px: u8) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    let script = format!(
        "(function(){{try{{if(!document.head){{return;}}var s=document.getElementById('unibrowser-user-style');if(!s){{s=document.createElement('style');s.id='unibrowser-user-style';document.head.appendChild(s);}}s.dataset.ubFont='{px}';s.textContent='html{{font-size:{px}px!important;}}';}}catch(e){{}}}})();",
        px = px
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_persisted_web_font(app: &AppHandle, window_label: &str, url: &str) -> Result<(), String> {
    let px = crate::web_font_prefs::get_origin_font_px(url);
    inject_page_font_px(app, window_label, px)?;
    Ok(())
}

pub fn inject_page_zoom_percent(
    app: &AppHandle,
    window_label: &str,
    percent: u8,
) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    let script = format!(
        "(function(){{try{{if(!document.documentElement){{return;}}document.documentElement.style.zoom='{pct}%';}}catch(e){{}}}})();",
        pct = percent
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_persisted_web_zoom(app: &AppHandle, window_label: &str, url: &str) -> Result<(), String> {
    let pct = crate::web_zoom_prefs::get_origin_zoom_percent(url);
    inject_page_zoom_percent(app, window_label, pct)?;
    Ok(())
}

/// Adjust font for a page URL, persist per origin, and inject into every open window.
pub fn adjust_page_font_for_url(app: &AppHandle, url: &str, delta: i32) -> Result<u8, String> {
    let px = crate::web_font_prefs::adjust_origin_font_px(url, delta)?;
    for (label, _) in app.webview_windows() {
        inject_page_font_px(app, &label, px)?;
    }
    Ok(px)
}

pub fn adjust_page_zoom_for_url(app: &AppHandle, url: &str, delta: i32) -> Result<u8, String> {
    let pct = crate::web_zoom_prefs::adjust_origin_zoom_percent(url, delta)?;
    for (label, _) in app.webview_windows() {
        inject_page_zoom_percent(app, &label, pct)?;
    }
    Ok(pct)
}

pub fn content_webview_navigate(
    app: &AppHandle,
    window_label: &str,
    url: &str,
) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    if url == "about:blank" {
        wv.hide().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let parsed: Url = url.parse().map_err(|e: url::ParseError| e.to_string())?;
    wv.navigate(parsed).map_err(|e| e.to_string())?;
    wv.show().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn sync_content_webview_for_window(
    app: &AppHandle,
    window_label: &str,
    loc: &unibrowser_mod_api::BrowserLocation,
) -> Result<(), String> {
    match loc {
        unibrowser_mod_api::BrowserLocation::Http { url } => {
            content_webview_navigate(app, window_label, url)?;
            apply_persisted_web_font(app, window_label, url)?;
            apply_persisted_web_zoom(app, window_label, url)?;
        }
        _ => {
            content_webview_navigate(app, window_label, "about:blank")?;
        }
    }
    if crate::preferences::load_preferences().advanced_webview_devtools {
        let _ = set_content_webview_devtools(app, window_label, true);
    }
    Ok(())
}

pub fn set_content_webview_devtools(
    app: &AppHandle,
    window_label: &str,
    open: bool,
) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    if open {
        wv.open_devtools();
    } else {
        wv.close_devtools();
    }
    Ok(())
}
