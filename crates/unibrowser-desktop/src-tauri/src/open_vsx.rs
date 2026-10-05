//! Thin Open VSX search client ([ADR 0014](../../docs/adr/0014-vsix-compatibility-layer.md)).
//! Opt-in network; in-memory cache; failures are soft (caller shows a message, UI stays up).

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

pub fn search_open_vsx(query: &str, network_opt_in: bool) -> Result<Vec<OpenVsxHitDto>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    if !network_opt_in {
        return Err("Open VSX search needs network opt-in (Mods panel)".into());
    }
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

    #[test]
    fn rejects_without_opt_in() {
        let err = search_open_vsx("markdown", false).unwrap_err();
        assert!(err.contains("opt-in"));
    }
}
