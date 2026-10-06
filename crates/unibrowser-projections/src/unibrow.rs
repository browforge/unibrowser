use unibrowser_project::ProjectManifest;

pub fn timeline_summary(manifest: &ProjectManifest) -> String {
    let mut lines = vec![
        format!("Project: {}", manifest.title),
        format!("Nodes: {}", manifest.nodes.len()),
        format!("Timeline clips: {}", manifest.timeline.clips.len()),
    ];
    for clip in &manifest.timeline.clips {
        let title = clip
            .title_text
            .as_deref()
            .filter(|t| !t.is_empty())
            .unwrap_or(clip.label.as_str());
        let extra = match (clip.pdf_page, clip.title_font_pt) {
            (Some(p), _) => format!(" pdf-page {}", p),
            (_, Some(pt)) => format!(" title {}pt", pt),
            _ => String::new(),
        };
        lines.push(format!(
            "  clip {} @ {:.2}s — {}{}",
            clip.id,
            clip.start_sec,
            title,
            extra
        ));
    }
    lines.join("\n")
}
