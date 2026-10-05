//! Apache-2.0 EPUB spine HTML preview (zip + OPF parse; no AGPL epub.js bundle).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use zip::ZipArchive;

#[derive(Debug, Serialize)]
pub struct EpubChapterDto {
    pub index: usize,
    pub total: usize,
    pub title: String,
    pub html: String,
}

#[derive(Debug, Serialize)]
pub struct EpubChapterListItem {
    pub index: usize,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct EpubSearchHitDto {
    pub chapter_index: usize,
    pub title: String,
    pub snippet: String,
    pub match_count: usize,
}

const SEARCH_SNIPPET_CHARS: usize = 72;
pub fn search_in_book(
    path: &Path,
    query: &str,
    limit: usize,
) -> Result<Vec<EpubSearchHitDto>, String> {
    let needle = query.trim();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let container = read_zip_entry(&mut archive, "META-INF/container.xml")?;
    let opf_path = extract_attr(&container, "full-path").ok_or("container: no full-path")?;
    let opf = read_zip_entry(&mut archive, &opf_path)?;
    let opf_dir = opf_parent(&opf_path);
    let spine_ids = parse_spine_itemrefs(&opf);
    let manifest = parse_manifest_hrefs(&opf);
    if spine_ids.is_empty() {
        return Err("EPUB spine empty".into());
    }
    let needle_lc = needle.to_lowercase();
    let cap = limit.max(1).min(200);
    let mut hits = Vec::new();
    for (chapter_index, id) in spine_ids.iter().enumerate() {
        if hits.len() >= cap {
            break;
        }
        let href = manifest.get(id).ok_or_else(|| format!("manifest missing spine id {}", id))?;
        let chapter_path = resolve_href(&opf_dir, href);
        let html = read_zip_entry(&mut archive, &chapter_path)?;
        let plain = html_to_plain(&html);
        let plain_lc = plain.to_lowercase();
        let match_count = count_substr(&plain_lc, &needle_lc);
        if match_count == 0 {
            continue;
        }
        let title = parse_title(&html).unwrap_or_else(|| format!("Chapter {}", chapter_index + 1));
        let snippet = snippet_around_match(&plain, &needle_lc).unwrap_or_else(|| {
            plain.chars().take(SEARCH_SNIPPET_CHARS).collect()
        });
        hits.push(EpubSearchHitDto {
            chapter_index,
            title,
            snippet,
            match_count,
        });
    }
    Ok(hits)
}

pub fn list_chapters(path: &Path) -> Result<Vec<EpubChapterListItem>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let container = read_zip_entry(&mut archive, "META-INF/container.xml")?;
    let opf_path = extract_attr(&container, "full-path").ok_or("container: no full-path")?;
    let opf = read_zip_entry(&mut archive, &opf_path)?;
    let opf_dir = opf_parent(&opf_path);
    let spine_ids = parse_spine_itemrefs(&opf);
    let manifest = parse_manifest_hrefs(&opf);
    if spine_ids.is_empty() {
        return Err("EPUB spine empty".into());
    }
    let mut items = Vec::with_capacity(spine_ids.len());
    for (chapter_index, id) in spine_ids.iter().enumerate() {
        let href = manifest
            .get(id)
            .ok_or_else(|| format!("manifest missing spine id {}", id))?;
        let chapter_path = resolve_href(&opf_dir, href);
        let html = read_zip_entry(&mut archive, &chapter_path)?;
        let title = parse_title(&html).unwrap_or_else(|| format!("Chapter {}", chapter_index + 1));
        items.push(EpubChapterListItem {
            index: chapter_index,
            title,
        });
    }
    Ok(items)
}

pub fn preview_chapter(path: &Path, index: usize) -> Result<EpubChapterDto, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let container = read_zip_entry(&mut archive, "META-INF/container.xml")?;
    let opf_path = extract_attr(&container, "full-path").ok_or("container: no full-path")?;
    let opf = read_zip_entry(&mut archive, &opf_path)?;
    let opf_dir = opf_parent(&opf_path);
    let spine_ids = parse_spine_itemrefs(&opf);
    let manifest = parse_manifest_hrefs(&opf);
    if spine_ids.is_empty() {
        return Err("EPUB spine empty".into());
    }
    let total = spine_ids.len();
    let idx = index.min(total - 1);
    let id = &spine_ids[idx];
    let href = manifest
        .get(id)
        .ok_or_else(|| format!("manifest missing spine id {}", id))?;
    let chapter_path = resolve_href(&opf_dir, href);
    let html = read_zip_entry(&mut archive, &chapter_path)?;
    let title = parse_title(&html).unwrap_or_else(|| format!("Chapter {}", idx + 1));
    Ok(EpubChapterDto {
        index: idx,
        total,
        title,
        html: wrap_chapter_html(html),
    })
}

fn wrap_chapter_html(body: String) -> String {
    if body.to_ascii_lowercase().contains("<html") {
        return body;
    }
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>{}</body></html>",
        body
    )
}

fn read_zip_entry(archive: &mut ZipArchive<File>, path: &str) -> Result<String, String> {
    let norm = path.replace('\\', "/");
    let mut file = archive.by_name(&norm).map_err(|e| e.to_string())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn opf_parent(opf_path: &str) -> String {
    let p = opf_path.replace('\\', "/");
    match p.rfind('/') {
        Some(i) => p[..i + 1].to_string(),
        None => String::new(),
    }
}

fn resolve_href(base: &str, href: &str) -> String {
    let href = href.replace('\\', "/");
    if href.starts_with('/') {
        return href.trim_start_matches('/').to_string();
    }
    format!("{}{}", base, href)
}

fn extract_attr(xml: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = xml.find(&needle)?;
    let rest = &xml[start + needle.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn parse_spine_itemrefs(opf: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for part in opf.split("itemref") {
        if let Some(id) = extract_attr(part, "idref") {
            ids.push(id);
        }
    }
    ids
}

fn parse_manifest_hrefs(opf: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for item in opf.split("<item ") {
        let id = extract_attr(item, "id");
        let href = extract_attr(item, "href");
        if let (Some(id), Some(href)) = (id, href) {
            map.insert(id, href);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn write_minimal_epub(path: &Path) {
        let file = File::create(path).expect("create");
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        zip.start_file("META-INF/container.xml", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0"?><container version="1.0"><rootfiles><rootfile full-path="content.opf"/></rootfiles></container>"#
        )
        .unwrap();
        zip.start_file("content.opf", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0"?><package><manifest>
<item id="c1" href="chap.xhtml" media-type="application/xhtml+xml"/>
</manifest><spine><itemref idref="c1"/></spine></package>"#
        )
        .unwrap();
        zip.start_file("chap.xhtml", opts).unwrap();
        write!(
            zip,
            r#"<!DOCTYPE html><html><head><title>Hello EPUB</title></head><body><p>One</p></body></html>"#
        )
        .unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn preview_first_chapter() {
        let path = std::env::temp_dir().join("unibrowser-epub-test.epub");
        write_minimal_epub(&path);
        let ch = preview_chapter(&path, 0).expect("preview");
        assert_eq!(ch.total, 1);
        assert_eq!(ch.title, "Hello EPUB");
        assert!(ch.html.contains("One"));
        let hits = search_in_book(&path, "one", 10).expect("search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chapter_index, 0);
        assert!(hits[0].snippet.to_ascii_lowercase().contains("one"));
        let toc = list_chapters(&path).expect("list");
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].title, "Hello EPUB");
        let _ = std::fs::remove_file(path);
    }
}

fn html_to_plain(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
            continue;
        }
        if ch == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            out.push(ch);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn count_substr(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }
    count
}

fn snippet_around_match(plain: &str, needle_lc: &str) -> Option<String> {
    let plain_lc = plain.to_lowercase();
    let byte_off = plain_lc.find(needle_lc)?;
    let start_char = plain[..byte_off].chars().count().saturating_sub(SEARCH_SNIPPET_CHARS / 3);
    let excerpt: String = plain
        .chars()
        .skip(start_char)
        .take(SEARCH_SNIPPET_CHARS)
        .collect();
    let trimmed = excerpt.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut snippet = String::new();
    if start_char > 0 {
        snippet.push('…');
    }
    snippet.push_str(trimmed);
    if start_char + SEARCH_SNIPPET_CHARS < plain.chars().count() {
        snippet.push('…');
    }
    Some(snippet)
}

fn parse_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    if let Some(start) = lower.find("<title>") {
        let rest = &html[start + 7..];
        if let Some(end) = rest.find("</title>") {
            return Some(rest[..end].trim().to_string());
        }
    }
    None
}
