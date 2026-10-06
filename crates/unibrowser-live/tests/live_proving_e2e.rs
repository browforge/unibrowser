//! Live compositor proving: pattern + SVG + HTML clock + chart → MP4.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use live_proving::{SineTemperatureSource, TestPatternSource, SINE_TEMP_ID, TEST_PATTERN_ID};
use unibrowser_live::{run_live_session_to_file, LiveScene};

fn ffmpeg_missing() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
}

fn overlay_svg_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../mods/live-proving/assets/overlay.svg")
}

#[test]
fn live_mode_writes_expected_duration() {
    if ffmpeg_missing() {
        eprintln!("skip: ffmpeg required for live proving e2e");
        return;
    }
    let svg = overlay_svg_path();
    assert!(svg.is_file(), "missing overlay svg at {}", svg.display());
    let scene = LiveScene::proving_full_stack(svg.to_string_lossy().into_owned());
    let fps = 10.0f32;
    let frames = 20u64;
    let expected_dur = frames as f64 / fps as f64;

    let mut pattern = TestPatternSource::new(scene.width, scene.height);
    let mut sine = SineTemperatureSource::default();

    let mut media: HashMap<String, &mut dyn unibrowser_live::MediaSource> = HashMap::new();
    media.insert(TEST_PATTERN_ID.into(), &mut pattern);
    let mut data: HashMap<String, &mut dyn unibrowser_live::DataSource> = HashMap::new();
    data.insert(SINE_TEMP_ID.into(), &mut sine);

    let dest = std::env::temp_dir().join(format!("ub-live-prove-{}.mp4", std::process::id()));
    let _ = std::fs::remove_file(&dest);

    let report = run_live_session_to_file(&scene, fps, frames, &dest, &mut media, &mut data)
        .expect("live session");

    assert!(report.frames_written >= frames - 1);
    assert!(PathBuf::from(&report.output_path).is_file());
    let dur = report.duration_sec.expect("ffprobe duration");
    assert!(
        (dur - expected_dur).abs() < 0.35,
        "duration {} vs expected {}",
        dur,
        expected_dur
    );
    let _ = std::fs::remove_file(&dest);
}
