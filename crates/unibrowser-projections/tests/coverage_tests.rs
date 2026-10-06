use std::path::Path;

use base_dxf::DxfBackend;
use base_text::TextBackend;
use toy_markdown::MarkdownBackend;
use unibrowser_engine::document::DocumentSession;
use unibrowser_projections::{ProjectionExtras, project_text, project_unibrow_manifest};
use unibrowser_project::ProjectManifest;

#[test]
fn text_backend_projection() {
    let backend = TextBackend {
        path: "/tmp/a.txt".into(),
        content: "hello world".into(),
        selection_start: 0,
        selection_end: 0,
        font_size_pt: 12.0,
        dirty: false,
    };
    let mut doc = DocumentSession::new("/tmp/a.txt", Box::new(backend));
    let text = project_text(&mut doc, &ProjectionExtras::default());
    assert!(text.contains("hello"));
}

#[test]
fn markdown_projection() {
    let backend = MarkdownBackend {
        source: "# Title\n\nBody".into(),
        heading_font_pt: 16.0,
        heading_selected: false,
    };
    let mut doc = DocumentSession::new("/tmp/a.md", Box::new(backend));
    let text = project_text(&mut doc, &ProjectionExtras::default());
    assert!(text.contains("Title"));
}

#[test]
fn web_dom_redacts_password_fields() {
    let backend = base_web::WebBackend::new("https://example.com");
    let mut doc = DocumentSession::new("https://example.com", Box::new(backend));
    let extras = ProjectionExtras {
        web_dom_text: Some("<input type=\"password\" value=x>".into()),
        ..Default::default()
    };
    let text = project_text(&mut doc, &extras);
    assert!(text.contains("[redacted]"));
    assert!(!text.contains("type=\"password\""));
}

#[test]
fn dxf_layers_and_text() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("t.dxf");
    std::fs::write(
        &path,
        "0\nSECTION\n2\nENTITIES\n0\nTEXT\n8\nWalls\n1\nHello\n0\nENDSEC\n0\nEOF\n",
    )
    .expect("write");
    let backend = DxfBackend::from_path(&path).expect("dxf");
    let mut doc = DocumentSession::new(path.to_string_lossy().as_ref(), Box::new(backend));
    let text = project_text(&mut doc, &ProjectionExtras::default());
    assert!(text.contains("Walls"));
    assert!(text.contains("Hello"));
}

#[test]
fn unibrow_timeline_summary() {
    let manifest = ProjectManifest::empty("Test");
    let summary = project_unibrow_manifest(&manifest);
    assert!(summary.contains("Project:"));
}
