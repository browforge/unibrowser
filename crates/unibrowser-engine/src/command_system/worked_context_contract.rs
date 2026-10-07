//! Contract tests: worked examples in docs/design/command-system.md vs resolved model.

use unibrowser_mod_api::{
    commands, control_ids, CapabilityState, CommandContext, HostContext, Mode, WorkContext,
};

use super::context::{infer_work_context, ResolutionContext};
use super::families::core_tab_layout;
use super::resolver::CapabilityResolver;
use super::save_guard::assert_save_independent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedCap {
    Supported,
    Emulated,
    Unsupported,
    Locked,
}

impl ExpectedCap {
    fn matches(&self, cap: &CapabilityState) -> bool {
        match (self, cap) {
            (ExpectedCap::Supported, CapabilityState::Supported) => true,
            (ExpectedCap::Emulated, CapabilityState::Emulated { .. }) => true,
            (ExpectedCap::Unsupported, CapabilityState::Unsupported { .. }) => true,
            (ExpectedCap::Locked, CapabilityState::Locked { .. }) => true,
            _ => false,
        }
    }
}

fn resolution_ctx(work: WorkContext, path: &str) -> ResolutionContext {
    let command_context = match work {
        WorkContext::Pdf => CommandContext::Office,
        WorkContext::Image => CommandContext::Image,
        WorkContext::CodeEditor => CommandContext::Code,
        WorkContext::RichDocument => CommandContext::Office,
        WorkContext::Start => CommandContext::Start,
        WorkContext::Browse => CommandContext::Browse,
        WorkContext::Settings => CommandContext::Browse,
        WorkContext::Mods => CommandContext::Browse,
        WorkContext::Project => CommandContext::Browse,
        WorkContext::Web => CommandContext::Web,
        _ => CommandContext::Text,
    };
    let mode = match work {
        WorkContext::Pdf => Mode::Document,
        WorkContext::Image => Mode::Vector,
        WorkContext::CodeEditor => Mode::Text,
        _ => Mode::Text,
    };
    let host = HostContext {
        mode,
        doc_type: String::new(),
        backend_id: String::new(),
        selection: Default::default(),
        capabilities: Default::default(),
    };
    let mut rc = ResolutionContext::new(command_context, host, Some(path.into()), 1);
    rc.work = work;
    rc.markdown_inline_html = true;
    rc
}

fn control_capability(work: WorkContext, path: &str, control_id: &str) -> CapabilityState {
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(work, path);
    let command = match control_id {
        control_ids::TEXT_FONT => commands::TEXT_FONT_FAMILY,
        control_ids::TEXT_SIZE => commands::FONT_SIZE_INCREASE,
        control_ids::TEXT_BOLD => commands::TEXT_BOLD,
        control_ids::TEXT_ITALIC => commands::TEXT_ITALIC,
        control_ids::TEXT_UNDERLINE => commands::TEXT_UNDERLINE,
        control_ids::TEXT_STRIKETHROUGH => commands::TEXT_STRIKETHROUGH,
        control_ids::TEXT_SUBSCRIPT => commands::TEXT_SUBSCRIPT,
        control_ids::TEXT_SUPERSCRIPT => commands::TEXT_SUPERSCRIPT,
        control_ids::TEXT_COLOUR => commands::TEXT_COLOUR,
        control_ids::EDITOR_FONT => commands::EDITOR_FONT_FAMILY,
        control_ids::EDITOR_SIZE => commands::EDITOR_FONT_SIZE,
        _ => commands::TEXT_BOLD,
    };
    resolver.resolve_control(control_id, command, &ctx)
}

fn find_control_label(work: WorkContext, path: &str, control_id: &str) -> Option<String> {
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(work, path);
    for family in core_tab_layout(&ctx, &resolver) {
        for group in &family.groups {
            for control in &group.controls {
                if control.control_id == control_id {
                    return Some(control.label.clone());
                }
            }
        }
    }
    None
}

fn assert_matrix_row(work: WorkContext, path: &str, control_id: &str, expected: ExpectedCap) {
    let cap = control_capability(work, path, control_id);
    assert!(
        expected.matches(&cap),
        "{} in {:?} ({}) expected {:?}, got {:?}",
        control_id,
        work,
        path,
        expected,
        cap
    );
}

#[test]
fn text_family_matrix_matches_command_system_doc() {
    let txt = "notes.txt";
    let md = "readme.md";
    let docx = "brief.docx";
    let code = "main.ts";

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_FONT,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_FONT,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_FONT,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::CodeEditor,
        code,
        control_ids::TEXT_FONT,
        ExpectedCap::Unsupported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_SIZE,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_SIZE,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_SIZE,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::CodeEditor,
        code,
        control_ids::EDITOR_SIZE,
        ExpectedCap::Supported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_BOLD,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_BOLD,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_BOLD,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::CodeEditor,
        code,
        control_ids::TEXT_BOLD,
        ExpectedCap::Unsupported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_ITALIC,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_ITALIC,
        ExpectedCap::Supported,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_ITALIC,
        ExpectedCap::Supported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_UNDERLINE,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_UNDERLINE,
        ExpectedCap::Emulated,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_UNDERLINE,
        ExpectedCap::Supported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_STRIKETHROUGH,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_STRIKETHROUGH,
        ExpectedCap::Supported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_SUBSCRIPT,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_SUBSCRIPT,
        ExpectedCap::Emulated,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_SUBSCRIPT,
        ExpectedCap::Supported,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_SUPERSCRIPT,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_SUPERSCRIPT,
        ExpectedCap::Emulated,
    );

    assert_matrix_row(
        WorkContext::PlainText,
        txt,
        control_ids::TEXT_COLOUR,
        ExpectedCap::Unsupported,
    );
    assert_matrix_row(
        WorkContext::Markdown,
        md,
        control_ids::TEXT_COLOUR,
        ExpectedCap::Emulated,
    );
    assert_matrix_row(
        WorkContext::RichDocument,
        docx,
        control_ids::TEXT_COLOUR,
        ExpectedCap::Supported,
    );
}

#[test]
fn plain_text_size_label_is_editor_display() {
    let label = find_control_label(WorkContext::PlainText, "notes.txt", control_ids::TEXT_SIZE)
        .expect("size control");
    assert!(
        label.contains("Editor text size"),
        "plain text size label must not imply file formatting: {}",
        label
    );
}

#[test]
fn plain_text_strikethrough_reason_matches_doc() {
    let cap = control_capability(
        WorkContext::PlainText,
        "notes.txt",
        control_ids::TEXT_STRIKETHROUGH,
    );
    let reason = cap.unsupported_reason().unwrap_or("");
    assert!(
        reason.to_lowercase().contains("plain text"),
        "reason: {}",
        reason
    );
}

#[test]
fn pdf_context_exposes_annotation_family() {
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(WorkContext::Pdf, "sample.pdf");
    let families = core_tab_layout(&ctx, &resolver);
    let pdf = families
        .iter()
        .find(|f| f.family_id == "unibrowser.family.pdf-annotation")
        .expect("pdf family");
    let ids: Vec<_> = pdf
        .groups
        .iter()
        .flat_map(|g| g.controls.iter())
        .map(|c| c.control_id.as_str())
        .collect();
    assert!(ids.iter().any(|id| *id == control_ids::PDF_PAGE));
    assert!(ids.iter().any(|id| *id == control_ids::PDF_EXPORT));
}

#[test]
fn image_context_exposes_adjust_family() {
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(WorkContext::Image, "photo.svg");
    let families = core_tab_layout(&ctx, &resolver);
    assert!(
        families
            .iter()
            .any(|f| f.family_id == "unibrowser.family.image-adjust"),
        "image adjust family present"
    );
}

#[test]
fn svg_path_infers_image_work_context() {
    assert_eq!(
        infer_work_context(CommandContext::Text, Mode::Text, Some("photo.svg")),
        WorkContext::Image
    );
}

#[test]
fn unibrow_path_resolves_project_family() {
    assert_eq!(
        infer_work_context(
            CommandContext::Browse,
            Mode::Browse,
            Some("/projects/show.unibrow"),
        ),
        WorkContext::Project
    );
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(WorkContext::Project, "/projects/show.unibrow");
    let families = core_tab_layout(&ctx, &resolver);
    let project = families
        .iter()
        .find(|family| family.family_id == "unibrowser.family.project")
        .expect("project family");
    let labels: Vec<_> = project
        .groups
        .iter()
        .flat_map(|group| group.controls.iter())
        .map(|control| control.label.as_str())
        .collect();
    assert!(labels.contains(&"Add asset"));
    assert!(labels.contains(&"Preview"));
    assert!(labels.contains(&"Export"));
}

#[test]
fn svg_image_context_marks_non_image_commands_unsupported() {
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(WorkContext::Image, "photo.svg");
    let cases = [
        (
            "unibrowser.control.find",
            commands::FIND_IN_DOCUMENT,
            "not available for images",
        ),
        (
            "unibrowser.control.proofing",
            commands::PROOFING_TOGGLE,
            "not images",
        ),
        (
            "unibrowser.control.image.curves",
            commands::IMAGE_CURVES,
            "raster pixels",
        ),
    ];
    for (control_id, command_id, fragment) in cases {
        let cap = resolver.resolve_control(control_id, command_id, &ctx);
        let reason = cap.unsupported_reason().unwrap_or("");
        assert!(reason.contains(fragment), "{control_id}: {reason}");
    }
}

#[test]
fn cad3d_context_exposes_proving_view_family_and_reasons() {
    assert_eq!(
        infer_work_context(CommandContext::Text, Mode::Text, Some("part.dxf")),
        WorkContext::Cad3d
    );
    let resolver = CapabilityResolver::default();
    let ctx = resolution_ctx(WorkContext::Cad3d, "part.dxf");
    let families = core_tab_layout(&ctx, &resolver);
    let cad = families
        .iter()
        .find(|family| family.family_id == "unibrowser.family.cad")
        .expect("CAD family");
    let controls: Vec<_> = cad
        .groups
        .iter()
        .flat_map(|group| group.controls.iter())
        .collect();
    for id in [
        "unibrowser.control.cad.fitView",
        "unibrowser.control.cad.zoomOut",
        "unibrowser.control.cad.zoomIn",
    ] {
        let control = controls
            .iter()
            .find(|control| control.control_id == id)
            .expect("supported CAD view control");
        assert!(matches!(&control.capability, CapabilityState::Supported));
    }
    for (id, reason_fragment) in [
        ("unibrowser.control.cad.find", "indexing provider"),
        ("unibrowser.control.cad.measure", "geometry engine"),
        ("unibrowser.control.cad.exportStep", "geometry engine"),
    ] {
        let control = controls
            .iter()
            .find(|control| control.control_id == id)
            .expect("unsupported CAD proving control");
        let reason = control.capability.unsupported_reason().unwrap_or("");
        assert!(reason.contains(reason_fragment), "reason: {reason}");
    }
}

#[test]
fn save_stays_supported_in_every_worked_context() {
    let resolver = CapabilityResolver::default();
    let paths = [
        (WorkContext::PlainText, "a.txt"),
        (WorkContext::Markdown, "a.md"),
        (WorkContext::RichDocument, "a.docx"),
        (WorkContext::CodeEditor, "a.ts"),
        (WorkContext::Pdf, "a.pdf"),
        (WorkContext::Image, "a.svg"),
        (WorkContext::Cad3d, "a.dxf"),
    ];
    for (work, path) in paths {
        let ctx = resolution_ctx(work, path);
        let cap = resolver.resolve_control("unibrowser.control.save", commands::TEXT_SAVE, &ctx);
        assert_save_independent(&cap);
    }
}
