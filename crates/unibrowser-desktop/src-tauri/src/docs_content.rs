//! Load markdown from the repo `docs/` tree for in-app help.

use std::path::PathBuf;

const ALLOWED: &[&str] = &[
    "getting-started",
    "chrome",
    "ui-commands",
    "keybindings",
    "commands-cross-format",
    "browser-shell",
    "downloads",
    "workspace-folders",
    "onboarding",
    "ux-a11y",
    "qa-desktop",
    "pdf-annotate",
    "proving-mod-dxf",
    "unibrow-format-migration",
    "ffmpeg-sidecar",
    "blitz-status",
];

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

pub fn read_markdown(slug: &str) -> Result<(String, String), String> {
    let slug = slug.trim();
    if !ALLOWED.contains(&slug) {
        return Err(format!("Unknown help topic: {}", slug));
    }
    let root = resolve_docs_root().ok_or("Documentation folder not found (dev checkout only)")?;
    let path = root.join(format!("{}.md", slug));
    if !path.is_file() {
        return Err(format!("Missing doc: {}", path.display()));
    }
    let body = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let title = slug.replace('-', " ");
    Ok((title, body))
}
