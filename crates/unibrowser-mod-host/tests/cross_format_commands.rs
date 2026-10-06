//! Same command ids must resolve to a handler per Bear headline format.

use std::collections::HashMap;
use std::path::Path;

use unibrowser_engine::{CommandRegistry, document::DocumentSession};
use unibrowser_mod_api::handlers::{
    EXPORT_PRINT_PDF_ID, FIND_IN_DOCUMENT_ID, FONT_INCREASE_ID, PAGE_ZOOM_FIT_ID,
};
use unibrowser_mod_host::{ModHost, ModRegisterFn, default_mods_dir};

fn bundled_registry() -> CommandRegistry {
    let mut registry = CommandRegistry::with_builtins();
    let bundled: HashMap<&str, ModRegisterFn> = [
        ("base-files", base_files::register as ModRegisterFn),
        ("base-web", base_web::register as ModRegisterFn),
        ("base-text", base_text::register as ModRegisterFn),
        ("base-png", base_png::register as ModRegisterFn),
        ("base-svg", base_svg::register as ModRegisterFn),
        ("base-docx", base_docx::register as ModRegisterFn),
        ("base-pdf", base_pdf::register as ModRegisterFn),
        ("base-dxf", base_dxf::register as ModRegisterFn),
        ("base-js", base_js::register as ModRegisterFn),
        ("base-video", base_video::register as ModRegisterFn),
        ("base-audio", base_audio::register as ModRegisterFn),
        ("toy-markdown", toy_markdown::register as ModRegisterFn),
        ("base-hex", base_hex::register as ModRegisterFn),
        ("base-epub", base_epub::register as ModRegisterFn),
        ("base-csv", base_csv::register as ModRegisterFn),
        ("spell-en-nz", spell_en_nz::register as ModRegisterFn),
        ("grammar-harper", grammar_harper::register as ModRegisterFn),
        ("dxf-checker", dxf_checker::register as ModRegisterFn),
        ("unibrow-checker", unibrow_checker::register as ModRegisterFn),
        ("live-proving", live_proving::register as ModRegisterFn),
    ]
    .into_iter()
    .collect();
    ModHost::discover_and_load(&[default_mods_dir()], &mut registry, &bundled)
        .expect("load mods");
    registry
}

fn assert_handler(reg: &CommandRegistry, command_id: &str, backend_id: &str) {
    assert!(
        reg.resolve_handler(command_id, backend_id).is_some(),
        "missing handler {command_id} for {backend_id}"
    );
}

#[test]
fn font_increase_hits_each_headline_format_handler() {
    let reg = bundled_registry();
    for backend_id in [
        base_text::BACKEND_ID,
        base_web::BACKEND_ID,
        base_svg::BACKEND_ID,
        base_docx::BACKEND_ID,
        base_epub::BACKEND_ID,
        base_dxf::BACKEND_ID,
        base_png::BACKEND_ID,
    ] {
        assert_handler(&reg, FONT_INCREASE_ID, backend_id);
    }
}

#[test]
fn font_increase_executes_on_text_docx_epub_svg_dxf_png_annotation() {
    let reg = bundled_registry();
    let dir = std::env::temp_dir().join("unibrowser_cross_font");
    std::fs::create_dir_all(&dir).expect("tmpdir");

    let txt = dir.join("a.txt");
    std::fs::write(&txt, "hi").expect("write");
    let mut text_doc = DocumentSession::new(
        txt.to_string_lossy().as_ref(),
        base_text::open_file(&txt).expect("open"),
    );
    reg.execute(FONT_INCREASE_ID, &mut text_doc).expect("text");

    let docx = dir.join("a.docx");
    std::fs::write(&docx, b"%DOCX%").expect("write");
    let mut docx_doc = DocumentSession::new(
        docx.to_string_lossy().as_ref(),
        base_docx::open_file(&docx).expect("open"),
    );
    reg.execute(FONT_INCREASE_ID, &mut docx_doc).expect("docx");

    let mut epub_doc = DocumentSession::new(
        "/x/book.epub",
        base_epub::open_file(Path::new("/x/book.epub")),
    );
    reg.execute(FONT_INCREASE_ID, &mut epub_doc).expect("epub");

    let svg = dir.join("a.svg");
    std::fs::write(
        &svg,
        r#"<svg width="10" height="10"><text font-size="10">x</text></svg>"#,
    )
    .expect("write");
    let mut svg_doc =
        DocumentSession::new(svg.to_string_lossy().as_ref(), base_svg::open_file(&svg));
    reg.execute(FONT_INCREASE_ID, &mut svg_doc).expect("svg");

    let dxf = dir.join("a.dxf");
    std::fs::write(&dxf, "0\nTEXT\n").expect("write");
    let mut dxf_doc = DocumentSession::new(
        dxf.to_string_lossy().as_ref(),
        base_dxf::open_file(&dxf).expect("open"),
    );
    reg.execute(FONT_INCREASE_ID, &mut dxf_doc).expect("dxf");

    let png = dir.join("a.png");
    write_min_png_with_text(&png, "font_pt=12;label=Note");
    let mut png_doc =
        DocumentSession::new(png.to_string_lossy().as_ref(), base_png::open_file(&png));
    reg.execute(FONT_INCREASE_ID, &mut png_doc).expect("png");

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn find_in_document_handler_per_format() {
    let reg = bundled_registry();
    for backend_id in [
        base_text::BACKEND_ID,
        base_web::BACKEND_ID,
        base_docx::BACKEND_ID,
        base_epub::BACKEND_ID,
        base_svg::BACKEND_ID,
        base_dxf::BACKEND_ID,
        base_png::BACKEND_ID,
    ] {
        assert_handler(&reg, FIND_IN_DOCUMENT_ID, backend_id);
    }
}

#[test]
fn zoom_to_fit_handler_per_viewer() {
    let reg = bundled_registry();
    for backend_id in [
        base_web::BACKEND_ID,
        base_png::BACKEND_ID,
        base_pdf::BACKEND_ID,
        base_svg::BACKEND_ID,
        base_dxf::BACKEND_ID,
        base_epub::BACKEND_ID,
    ] {
        assert_handler(&reg, PAGE_ZOOM_FIT_ID, backend_id);
    }
}

#[test]
fn export_print_pdf_handler_registered_for_doc_formats() {
    let reg = bundled_registry();
    for backend_id in [base_text::BACKEND_ID, base_docx::BACKEND_ID] {
        assert_handler(&reg, EXPORT_PRINT_PDF_ID, backend_id);
    }
}

fn write_min_png_with_text(path: &Path, annotation: &str) {
    let mut out = Vec::new();
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    append_chunk(&mut out, b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]);
    let text = format!("unibrowser_text\0{}", annotation);
    append_chunk(&mut out, b"tEXt", text.as_bytes());
    append_chunk(&mut out, b"IEND", &[]);
    std::fs::write(path, out).expect("png");
}

fn append_chunk(out: &mut Vec<u8>, typ: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(typ);
    out.extend_from_slice(data);
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(typ);
    hasher.update(data);
    let crc = hasher.finalize();
    out.extend_from_slice(&crc.to_be_bytes());
}
