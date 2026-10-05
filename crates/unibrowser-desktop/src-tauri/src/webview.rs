use serde::Serialize;
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use url::Url;

use crate::downloads::DownloadEventPayload;
use crate::webview_download::emit_unibrowser_download;

pub fn content_label_for_capture(window_label: &str) -> String {
    format!("content-{}", window_label)
}

fn content_label(window_label: &str) -> String {
    content_label_for_capture(window_label)
}

fn window_label_from_content_webview(content_webview_label: &str) -> String {
    content_webview_label
        .strip_prefix("content-")
        .unwrap_or(content_webview_label)
        .to_string()
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpPageLoadEvent {
    window_label: String,
    url: String,
    phase: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpPageTitleEvent {
    window_label: String,
    title: String,
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
    let mut builder = tauri::webview::WebviewBuilder::new(&label, WebviewUrl::External(blank));
    if let Some(proxy) =
        crate::web_proxy::proxy_url_for_prefs(&crate::preferences::load_preferences().web_proxy)
    {
        builder = builder.proxy_url(proxy);
    }
    builder = builder
        .on_page_load(|webview, payload| {
            let phase = match payload.event() {
                PageLoadEvent::Started => "started".to_string(),
                PageLoadEvent::Finished => "finished".to_string(),
            };
            let window_label = window_label_from_content_webview(webview.label());
            let _ = webview.app_handle().emit(
                "unibrowser-http-page-load",
                HttpPageLoadEvent {
                    window_label,
                    url: payload.url().to_string(),
                    phase,
                },
            );
        })
        .on_document_title_changed(|webview, title| {
            let window_label = window_label_from_content_webview(webview.label());
            let _ = webview.app_handle().emit(
                "unibrowser-http-page-title",
                HttpPageTitleEvent {
                    window_label,
                    title,
                },
            );
        });
    #[cfg(not(target_os = "windows"))]
    {
        builder = builder.on_download(|webview, event| {
            use tauri::webview::DownloadEvent;
            match event {
                DownloadEvent::Requested { url, destination } => {
                    let payload = DownloadEventPayload {
                        url: url.to_string(),
                        destination: Some(destination.to_string_lossy().into_owned()),
                    };
                    emit_unibrowser_download(webview.app_handle(), payload);
                    return false;
                }
                DownloadEvent::Finished { .. } => {
                    return false;
                }
                _ => {}
            }
            true
        });
    }
    window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 140.0),
            LogicalSize::new(800.0, 600.0),
        )
        .map_err(|e| e.to_string())?;
    if let Some(wv) = app.get_webview(&label) {
        #[cfg(target_os = "windows")]
        {
            crate::webview_download_win::try_attach_download_starting(
                app.clone(),
                &wv,
                &label,
            );
        }
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

pub fn reset_page_zoom_for_url(app: &AppHandle, url: &str) -> Result<u8, String> {
    let pct = crate::web_zoom_prefs::reset_origin_zoom_percent(url)?;
    for (label, _) in app.webview_windows() {
        inject_page_zoom_percent(app, &label, pct)?;
    }
    Ok(pct)
}

pub fn reset_page_font_for_url(app: &AppHandle, url: &str) -> Result<u8, String> {
    let px = crate::web_font_prefs::reset_origin_font_px(url)?;
    for (label, _) in app.webview_windows() {
        inject_page_font_px(app, &label, px)?;
    }
    Ok(px)
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

pub fn apply_reader_mode(
    app: &AppHandle,
    window_label: &str,
    enabled: bool,
) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    let script = if enabled {
        r#"(function(){try{var id='unibrowser-reader-style';var s=document.getElementById(id);if(!s){s=document.createElement('style');s.id=id;document.head.appendChild(s);}s.textContent='header,nav,footer,aside,[role=banner],[role=navigation],.ad,.ads,.advertisement,.cookie,.newsletter{display:none!important;}body{margin:0 auto!important;max-width:42rem!important;padding:1rem 1.25rem!important;line-height:1.65!important;font-size:1.05rem!important;background:#fff!important;color:#111!important;}article,main,[role=main]{display:block!important;}img,video{max-width:100%!important;height:auto!important;}';document.documentElement.classList.add('ub-reader-mode');var pick=document.querySelector('article')||document.querySelector('main')||document.querySelector('[role=main]');if(pick){pick.scrollIntoView({block:'start'});}}catch(e){}})();"#
    } else {
        r#"(function(){try{var s=document.getElementById('unibrowser-reader-style');if(s){s.remove();}document.documentElement.classList.remove('ub-reader-mode');}catch(e){}})();"#
    };
    wv.eval(script).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_tab_mute(app: &AppHandle, window_label: &str, muted: bool) -> Result<(), String> {
    ensure_content_webview(app, window_label)?;
    let label = content_label(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    let script = format!(
        r#"(function(){{try{{var muted={muted};function apply(){{document.querySelectorAll('video,audio').forEach(function(el){{el.muted=muted;}});}}apply();if(!window.__ubMuteObs){{window.__ubMuteObs=new MutationObserver(apply);window.__ubMuteObs.observe(document.documentElement,{{childList:true,subtree:true}});}}window.__ubTabMuted=muted;}}catch(e){{}}}})();"#,
        muted = if muted { "true" } else { "false" }
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn sync_content_webview_for_window(
    app: &AppHandle,
    window_label: &str,
    loc: &unibrowser_mod_api::BrowserLocation,
    reader_mode: bool,
    tab_muted: bool,
) -> Result<(), String> {
    match loc {
        unibrowser_mod_api::BrowserLocation::Http { url } => {
            content_webview_navigate(app, window_label, url)?;
            apply_persisted_web_font(app, window_label, url)?;
            apply_persisted_web_zoom(app, window_label, url)?;
            apply_reader_mode(app, window_label, reader_mode)?;
            apply_tab_mute(app, window_label, tab_muted)?;
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
    set_devtools_open_flag(window_label, open);
    Ok(())
}

fn devtools_flags() -> &'static std::sync::Mutex<std::collections::HashMap<String, bool>> {
    use std::sync::{Mutex, OnceLock};
    static FLAGS: OnceLock<Mutex<std::collections::HashMap<String, bool>>> = OnceLock::new();
    FLAGS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn set_devtools_open_flag(window_label: &str, open: bool) {
    if let Ok(mut map) = devtools_flags().lock() {
        map.insert(window_label.to_string(), open);
    }
}

pub fn toggle_content_webview_devtools(
    app: &AppHandle,
    window_label: &str,
) -> Result<bool, String> {
    let open = devtools_flags()
        .lock()
        .map_err(|e| e.to_string())?
        .get(window_label)
        .copied()
        .unwrap_or(false);
    set_content_webview_devtools(app, window_label, !open)?;
    Ok(!open)
}

