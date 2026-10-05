//! Apply `preferences::WebProxyPrefs` to content webviews ([docs/network-proxy.md](../../../docs/network-proxy.md)).

use url::Url;

use crate::preferences::{WebProxyMode, WebProxyPrefs};

/// Proxy URL for Tauri `WebviewBuilder::proxy_url`, if manual mode is configured.
pub fn proxy_url_for_prefs(prefs: &WebProxyPrefs) -> Option<Url> {
    match prefs.mode {
        WebProxyMode::System | WebProxyMode::Direct => None,
        WebProxyMode::Manual => {
            let host = prefs.host.trim();
            if host.is_empty() || prefs.port == 0 {
                return None;
            }
            let url = format!("http://{}:{}", host, prefs.port);
            Url::parse(&url).ok()
        }
    }
}

/// Human-readable line for diagnostics.
pub fn proxy_status_line(prefs: &WebProxyPrefs) -> String {
    match prefs.mode {
        WebProxyMode::System => "system (OS / WebView default)".into(),
        WebProxyMode::Direct => "direct (no proxy override)".into(),
        WebProxyMode::Manual => {
            if let Some(u) = proxy_url_for_prefs(prefs) {
                format!("manual → {}", u)
            } else {
                "manual (incomplete host/port — not applied)".into()
            }
        }
    }
}
