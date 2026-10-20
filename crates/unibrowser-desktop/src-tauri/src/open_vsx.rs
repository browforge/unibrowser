//! Thin Open VSX search client ([ADR 0014](../../docs/adr/0014-vsix-compatibility-layer.md)).
//! Opt-in network; in-memory cache; failures are soft (caller shows a message, UI stays up).

use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const SEARCH_URL: &str = "https://open-vsx.org/api/-/search";
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Serialize)]
pub struct OpenVsxHitDto {
    pub extension_id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub license: Option<String>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    extensions: Option<Vec<ExtensionEntry>>,
}

#[derive(Debug, Deserialize)]
struct ExtensionEntry {
    namespace: Option<String>,
    name: Option<String>,
    version: Option<String>,
    publisher: Option<Publisher>,
    files: Option<Files>,
    license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Publisher {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Files {
    download: Option<String>,
}

struct CacheEntry {
    query: String,
    hits: Vec<OpenVsxHitDto>,
    at: Instant,
}

static CACHE: Mutex<Option<CacheEntry>> = Mutex::new(None);

/// Open VSX catalog + VSIX download (not https tabs or user-opened URLs).
pub fn ensure_catalog_network(network_opt_in: bool) -> Result<(), String> {
    let prefs = crate::preferences::load_preferences();
    if prefs.offline_mode {
        return Err(
            "Offline mode is on (Settings → Network). Turn it off to reach Open VSX.".into(),
        );
    }
    if !network_opt_in {
        return Err("Open VSX search needs network opt-in (Mods panel)".into());
    }
    Ok(())
}

pub fn search_open_vsx(query: &str, network_opt_in: bool) -> Result<Vec<OpenVsxHitDto>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    ensure_catalog_network(network_opt_in)?;
    if let Ok(guard) = CACHE.lock() {
        if let Some(entry) = guard.as_ref() {
            if entry.query == query && entry.at.elapsed() < CACHE_TTL {
                return Ok(entry.hits.clone());
            }
        }
    }
    let url = format!(
        "{}?query={}&size=12",
        SEARCH_URL,
        urlencoding::encode(query)
    );
    let body = match ureq::get(&url)
        .timeout(std::time::Duration::from_secs(8))
        .call()
    {
        Ok(resp) => resp.into_string().map_err(|e| e.to_string())?,
        Err(ureq::Error::Status(code, resp)) => {
            return Err(format!("Open VSX HTTP {}: {}", code, resp.status_text()));
        }
        Err(e) => return Err(format!("Open VSX unreachable: {}", e)),
    };
    let parsed: SearchResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Open VSX response parse error: {}", e))?;
    let hits = parsed
        .extensions
        .unwrap_or_default()
        .into_iter()
        .filter_map(map_hit)
        .collect::<Vec<_>>();
    if let Ok(mut guard) = CACHE.lock() {
        *guard = Some(CacheEntry {
            query: query.to_string(),
            hits: hits.clone(),
            at: Instant::now(),
        });
    }
    Ok(hits)
}

pub fn download_vsix_bytes(url: &str, network_opt_in: bool) -> Result<Vec<u8>, String> {
    ensure_catalog_network(network_opt_in)?;
    let url = url.trim();
    if url.is_empty() {
        return Err("empty download URL".into());
    }
    if !url.starts_with("https://") {
        return Err("only https VSIX downloads are supported".into());
    }
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(60))
        .call()
        .map_err(|e| format!("download failed: {}", e))?;
    let mut reader = resp.into_reader();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read body: {}", e))?;
    if bytes.len() < 4 || bytes[0] != 0x50 || bytes[1] != 0x4b {
        return Err("response is not a zip/vsix payload".into());
    }
    Ok(bytes)
}

fn map_hit(ext: ExtensionEntry) -> Option<OpenVsxHitDto> {
    let namespace = ext.namespace.unwrap_or_default();
    let name = ext.name.unwrap_or_default();
    if namespace.is_empty() || name.is_empty() {
        return None;
    }
    let extension_id = format!("{}.{}", namespace, name);
    let version = ext.version.unwrap_or_else(|| "?".into());
    let publisher = ext
        .publisher
        .and_then(|p| p.display_name)
        .unwrap_or_else(|| namespace.clone());
    let url = ext
        .files
        .and_then(|f| f.download)
        .unwrap_or_else(|| format!("https://open-vsx.org/extension/{}/{}", namespace, name));
    Some(OpenVsxHitDto {
        extension_id,
        name,
        version,
        publisher,
        license: ext.license,
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use crate::preferences::{Preferences, save_preferences};

    static PERM_CONFIG_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn perm03_rejects_without_opt_in() {
        let err = search_open_vsx("markdown", false).unwrap_err();
        assert!(err.to_lowercase().contains("opt-in"));
    }

    #[test]
    fn perm03_offline_mode_blocks_catalog() {
        let _guard = PERM_CONFIG_LOCK.lock().expect("perm config lock");
        let dir = tempfile::tempdir().expect("tempdir");
        let prev = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        let mut prefs = Preferences::default();
        prefs.offline_mode = true;
        save_preferences(&prefs).expect("save prefs");
        let err = search_open_vsx("markdown", true).unwrap_err();
        if let Some(p) = prev {
            std::env::set_var("XDG_CONFIG_HOME", p);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        assert!(
            err.to_lowercase().contains("offline"),
            "unexpected: {}",
            err
        );
    }
}
