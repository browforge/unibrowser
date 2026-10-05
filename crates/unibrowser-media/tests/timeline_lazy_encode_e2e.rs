//! End-to-end lazy timeline encode (skips when ffmpeg or pdftoppm missing).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use unibrowser_media::{encode_h264_timeline_intents, find_binary, TimelineEncodeOptions};
use unibrowser_project::{
    add_pdf_page_timeline_clip, add_title_card_timeline_clip, build_timeline_encode_intent,
    create_package,
};

fn tool_missing() -> bool {
    find_binary("ffmpeg").is_none() || find_binary("pdftoppm").is_none()
}

fn write_minimal_svg(path: &PathBuf) {
    fs::write(
        path,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#2244aa"/></svg>"##,
    )
    .expect("svg");
}

fn write_minimal_pdf(path: &PathBuf) {
    let bytes = b"%PDF-1.1\n1 0 obj<< /Type /Catalog /Pages 2 0 R>>endobj\n2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1>>endobj\n3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents 4 0 R>>endobj\n4 0 obj<< /Length 44>>stream\nBT /F1 24 Tf 50 120 Td (Hi) Tj ET\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000206 00000 n \ntrailer<< /Size 5 /Root 1 0 R>>\nstartxref\n300\n%%EOF\n";
    fs::write(path, bytes).expect("pdf");
}

fn ffprobe_duration_sec(path: &PathBuf) -> Option<f64> {
    let ffprobe = find_binary("ffprobe")?;
    let out = Command::new(&ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout.trim().parse().ok()
}

#[test]
fn lazy_visual_clips_encode_to_mp4_with_expected_duration() {
    if tool_missing() {
        eprintln!("skip: ffmpeg and pdftoppm required for lazy encode e2e");
        return;
    }
    let root = std::env::temp_dir().join(format!("ub-lazy-e2e-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("tmpdir");
    let mut pkg = create_package(&root, "Lazy e2e").expect("create");
    let svg = root.join("assets").join("still.svg");
    let pdf = root.join("assets").join("page.pdf");
    write_minimal_svg(&svg);
    write_minimal_pdf(&pdf);
    add_title_card_timeline_clip(&mut pkg, "E2E title").expect("title");
    unibrowser_project::add_asset_timeline_clip(&mut pkg, &svg, "still").expect("svg");
    add_pdf_page_timeline_clip(&mut pkg, &pdf, 1, "page").expect("pdf");

    let expected_dur = pkg
        .manifest
        .timeline
        .clips
        .iter()
        .map(|c| c.duration_sec)
        .sum::<f32>();
    assert!(expected_dur > 0.5);

    let intent = build_timeline_encode_intent(&pkg);
    let dest = root.join("derived").join("out.mp4");
    fs::create_dir_all(dest.parent().unwrap()).expect("derived");
    encode_h264_timeline_intents(
        &intent,
        &dest,
        "ultrafast",
        &TimelineEncodeOptions::default(),
        &mut |_p, _s| {},
    )
    .expect("encode");

    assert!(dest.is_file());
    assert!(dest.metadata().map(|m| m.len()).unwrap_or(0) > 1024);
    let dur = ffprobe_duration_sec(&dest).expect("ffprobe duration");
    assert!(
        (dur - expected_dur as f64).abs() < 1.5,
        "duration {} vs expected {}",
        dur,
        expected_dur
    );
    let _ = fs::remove_dir_all(&root);
}
