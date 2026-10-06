//! In-page find for the https content webview (`window.find` injection).

use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WebFindStatus {
    pub available: bool,
    pub detail: String,
}

pub fn web_find_status() -> WebFindStatus {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        return WebFindStatus {
            available: true,
            detail: "Uses content webview window.find() for https tabs.".into(),
        };
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        WebFindStatus {
            available: false,
            detail: "In-page find is not wired for this platform yet.".into(),
        }
    }
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Run find in the active content webview. Returns whether a match was found.
pub fn find_in_content_webview(
    app: &AppHandle,
    window_label: &str,
    query: &str,
    forward: bool,
) -> Result<(), String> {
    let status = web_find_status();
    if !status.available {
        return Err(status.detail);
    }
    let q = query.trim();
    if q.is_empty() {
        return Ok(());
    }
    crate::webview::ensure_content_webview(app, window_label)?;
    let label = crate::webview::content_label_for_capture(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    let escaped = escape_js_string(q);
    let back = if forward { "false" } else { "true" };
    let script = format!(
        "(function(){{try{{return Boolean(window.find && window.find('{q}', false, {back}, false, false, false, false));}}catch(e){{return false;}}}})();",
        q = escaped,
        back = back
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}

/// Focus the first element matching `selector` in the content webview (https tabs / forms).
pub fn focus_dom_in_content_webview(
    app: &AppHandle,
    window_label: &str,
    selector: &str,
) -> Result<(), String> {
    let status = web_find_status();
    if !status.available {
        return Err(status.detail);
    }
    let sel = selector.trim();
    if sel.is_empty() {
        return Ok(());
    }
    crate::webview::ensure_content_webview(app, window_label)?;
    let label = crate::webview::content_label_for_capture(window_label);
    let wv = app.get_webview(&label).ok_or("content webview missing")?;
    let escaped = escape_js_string(sel);
    let script = format!(
        "(function(){{try{{var el=document.querySelector('{sel}');if(!el){{return false;}}el.scrollIntoView({{block:'center',inline:'nearest'}});if(typeof el.focus==='function'){{el.focus({{preventScroll:true}});}}return true;}}catch(e){{return false;}}}})();",
        sel = escaped
    );
    wv.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}
