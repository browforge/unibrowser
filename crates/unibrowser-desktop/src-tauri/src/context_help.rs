//! Map active tab context to bundled docs slugs ([docs/]).

use unibrowser_mod_api::BrowserLocation;
use unibrowser_engine::tabs::Tab;

use base_audio::BACKEND_ID as AUDIO_ID;
use base_docx::BACKEND_ID as DOCX_ID;
use base_dxf::BACKEND_ID as DXF_ID;
use base_pdf::BACKEND_ID as PDF_ID;
use base_png::BACKEND_ID as PNG_ID;
use base_svg::BACKEND_ID as SVG_ID;
use base_text::BACKEND_ID as TEXT_ID;
use base_video::BACKEND_ID as VIDEO_ID;
use base_web::BACKEND_ID as WEB_ID;

pub fn slug_for_tab(tab: &Tab) -> String {
    if let Some(doc) = &tab.document {
        return slug_for_backend(doc.backend.backend_id()).to_string();
    }
    match &tab.location {
        BrowserLocation::File { path } if path.is_dir() => "workspace-folders".into(),
        BrowserLocation::File { .. } => "getting-started".into(),
        BrowserLocation::Http { .. } => "browser-shell".into(),
        BrowserLocation::Internal { page } => slug_for_internal_page(page),
    }
}

fn slug_for_internal_page(page: &str) -> String {
    if page == "home" {
        return "getting-started".into();
    }
    if page == "settings" {
        return "onboarding".into();
    }
    if page == "downloads" {
        return "downloads".into();
    }
    if page == "diagnostics" {
        return "qa-desktop".into();
    }
    if page == "keys" {
        return "keybindings".into();
    }
    if page == "about" {
        return "getting-started".into();
    }
    if let Some(rest) = page.strip_prefix("docs/") {
        let slug = rest.split('/').next().unwrap_or("getting-started");
        if crate::docs_content::read_markdown(slug).is_ok() {
            return slug.to_string();
        }
    }
    if page.starts_with("files") {
        return "workspace-folders".into();
    }
    "getting-started".into()
}

pub fn slug_for_backend(backend_id: &str) -> &'static str {
    match backend_id {
        x if x == PDF_ID => "pdf-annotate",
        x if x == TEXT_ID => "ui-commands",
        x if x == DOCX_ID => "commands-cross-format",
        x if x == base_epub::BACKEND_ID => "commands-cross-format",
        x if x == WEB_ID => "browser-shell",
        x if x == PNG_ID || x == SVG_ID => "commands-cross-format",
        x if x == VIDEO_ID || x == AUDIO_ID => "ffmpeg-sidecar",
        x if x == DXF_ID => "proving-mod-dxf",
        x if x == crate::project_backend::BACKEND_ID => "unibrow-format-migration",
        x if x == crate::settings_stub::BACKEND_ID => "onboarding",
        x if x == crate::diagnostics_stub::BACKEND_ID => "qa-desktop",
        x if x == crate::keys_stub::BACKEND_ID => "keybindings",
        x if x == crate::downloads_stub::BACKEND_ID => "downloads",
        x if x == crate::about_stub::BACKEND_ID => "getting-started",
        x if x == crate::docs_stub::BACKEND_ID => "getting-started",
        _ => "getting-started",
    }
}
