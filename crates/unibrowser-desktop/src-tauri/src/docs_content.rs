//! Load markdown from the repo `docs/` tree for in-app help.

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct DocTocEntry {
    pub section: String,
    pub slug: String,
    pub label: String,
}

pub fn resolve_docs_root() -> Option<PathBuf> {
    let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    if from_manifest.is_dir() {
        return from_manifest.canonicalize().ok();
    }
    let cwd = PathBuf::from("docs");
    if cwd.is_dir() {
        return cwd.canonicalize().ok();
    }
    None
}

fn safe_slug(slug: &str) -> bool {
    let slug = slug.trim();
    !slug.is_empty()
        && slug.len() <= 80
        && !slug.contains("..")
        && slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '/')
}

fn slug_from_link(path: &str) -> Option<String> {
    let path = path.trim();
    if !path.ends_with(".md") {
        return None;
    }
    let without = path.strip_suffix(".md")?;
    if without.contains("adr/") || without.contains("CONTRIBUTING") {
        return None;
    }
    let slug = without
        .rsplit('/')
        .next()
        .unwrap_or(without)
        .to_string();
    if safe_slug(&slug) {
        Some(slug)
    } else {
        None
    }
}

pub fn slug_markdown_path(slug: &str) -> Result<PathBuf, String> {
    if slug == "index" {
        let root = resolve_docs_root().ok_or("Documentation folder not found")?;
        return Ok(root.join("index.md"));
    }
    if !safe_slug(slug) {
        return Err("Invalid doc slug".into());
    }
    let root = resolve_docs_root().ok_or("Documentation folder not found")?;
    let path = root.join(format!("{}.md", slug));
    if !path.is_file() {
        return Err(format!("Missing doc: {}", path.display()));
    }
    Ok(path)
}

pub fn read_markdown(slug: &str) -> Result<(String, String), String> {
    let slug = slug.trim();
    let path = slug_markdown_path(slug)?;
    let body = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let title = if slug == "index" {
        "Documentation index".into()
    } else {
        slug.replace('-', " ")
    };
    Ok((title, body))
}

pub fn toc_from_index() -> Result<Vec<DocTocEntry>, String> {
    let (_, index_md) = read_markdown("index")?;
    let mut section = "Documentation".to_string();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in index_md.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            section = rest.trim().to_string();
            continue;
        }
        let mut search = line;
        while let Some(start) = search.find("](") {
            let after = &search[start + 2..];
            if let Some(end) = after.find(')') {
                let link = after[..end].trim();
                if let Some(slug) = slug_from_link(link) {
                    if slug_markdown_path(&slug).is_ok() && seen.insert(slug.clone()) {
                        let label_start = line.rfind('[').unwrap_or(0);
                        let label = line[label_start + 1..]
                            .split(']')
                            .next()
                            .unwrap_or(&slug)
                            .trim()
                            .to_string();
                        out.push(DocTocEntry {
                            section: section.clone(),
                            slug,
                            label,
                        });
                    }
                }
                search = &after[end + 1..];
            } else {
                break;
            }
        }
    }
    if !seen.contains("index") {
        out.insert(
            0,
            DocTocEntry {
                section: "Documentation".into(),
                slug: "index".into(),
                label: "Index".into(),
            },
        );
    }
    Ok(out)
}
