//! Minimal `.unibrow` live preview + local record (ADR 0024). No network.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use image::ImageEncoder;
use live_proving::{
    sine_temperature_source, test_pattern_source, SINE_TEMP_ID, TEST_PATTERN_ID,
};
use serde::Serialize;
use tauri::State;
use unibrowser_live::{
    run_live_session_to_file, scene_from_unibrow_manifest, FileVideoSink, FrameClock,
    LiveCompositor, LiveScene,
};
use unibrowser_project::is_unibrow_path;

use crate::project_backend::BACKEND_ID;
use crate::state::AppState;

const OVERLAY_SVG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../mods/live-proving/assets/overlay.svg"
);

pub struct LiveHostState {
    pub live_mode: bool,
    pub recording: bool,
    pub scene: LiveScene,
    clock: FrameClock,
    live_started: Instant,
    pattern: live_proving::TestPatternSource,
    sine: live_proving::SineTemperatureSource,
    sink: Option<FileVideoSink>,
    record_path: Option<PathBuf>,
}

impl LiveHostState {
    fn from_project_path(project_path: &str) -> Result<Self, String> {
        let root = std::path::Path::new(project_path);
        let pkg = unibrowser_project::open_package(root).map_err(|e| e.to_string())?;
        let scene = scene_from_unibrow_manifest(&pkg.manifest, &pkg.root);
        Ok(Self::from_scene(scene))
    }

    fn from_scene(scene: LiveScene) -> Self {
        let w = scene.width;
        let h = scene.height;
        Self {
            live_mode: true,
            recording: false,
            scene,
            clock: FrameClock::new(10.0),
            live_started: Instant::now(),
            pattern: test_pattern_source(w, h),
            sine: sine_temperature_source(),
            sink: None,
            record_path: None,
        }
    }

    fn new_proving() -> Self {
        Self::from_scene(LiveScene::proving_full_stack(OVERLAY_SVG.to_string()))
    }

    fn elapsed_sec(&self) -> f64 {
        self.live_started.elapsed().as_secs_f64()
    }
}

fn live_by_window() -> &'static Mutex<HashMap<String, LiveHostState>> {
    static MAP: OnceLock<Mutex<HashMap<String, LiveHostState>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn active_unibrow_project_path(state: &AppState, window_label: &str) -> Result<String, String> {
    let sess = state.session(window_label)?;
    let tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    let doc = tab.document.as_ref().ok_or("open a .unibrow project")?;
    if doc.backend.backend_id() != BACKEND_ID {
        return Err("Live mode is only for .unibrow projects".into());
    }
    if !is_unibrow_path(std::path::Path::new(&doc.path)) {
        return Err("not a .unibrow path".into());
    }
    Ok(doc.path.clone())
}

#[derive(Serialize)]
pub struct UnibrowLiveStatusDto {
    pub live_mode: bool,
    pub recording: bool,
    pub png_base64: Option<String>,
    pub record_path: Option<String>,
}

fn rgba_to_png_base64(width: u32, height: u32, rgba: &[u8]) -> Result<String, String> {
    let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .ok_or("invalid rgba buffer")?;
    let mut bytes = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut bytes);
    enc.write_image(
        img.as_raw(),
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| e.to_string())?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

fn render_preview(host: &mut LiveHostState) -> Result<String, String> {
    let tick = host.clock.tick();
    let t_sec = host.elapsed_sec();
    let compositor = LiveCompositor::new(host.scene.width, host.scene.height);
    let mut media: HashMap<String, &mut dyn unibrowser_live::MediaSource> = HashMap::new();
    media.insert(TEST_PATTERN_ID.into(), &mut host.pattern);
    let mut data: HashMap<String, &mut dyn unibrowser_live::DataSource> = HashMap::new();
    data.insert(SINE_TEMP_ID.into(), &mut host.sine);
    let frame = compositor
        .render_frame(&host.scene, t_sec, tick.deadline, &mut media, &mut data)
        .map_err(|e| e.to_string())?;
    if host.recording {
        if let Some(sink) = host.sink.as_mut() {
            let _ = sink.write_frame(&frame);
        }
    }
    rgba_to_png_base64(frame.width, frame.height, &frame.rgba)
}

#[tauri::command]
pub fn set_unibrow_live_mode(
    window: tauri::Window,
    enabled: bool,
    state: State<AppState>,
) -> Result<UnibrowLiveStatusDto, String> {
    let label = window.label().to_string();
    let _path = active_unibrow_project_path(state.inner(), &label)?;
    let mut map = live_by_window().lock().map_err(|e| e.to_string())?;
    if enabled {
        let host = LiveHostState::from_project_path(&_path).unwrap_or_else(|_| LiveHostState::new_proving());
        map.insert(label, host);
    } else {
        if let Some(mut host) = map.remove(&label) {
            if host.recording {
                if let Some(sink) = host.sink.take() {
                    let _ = sink.finish();
                }
            }
        }
    }
    Ok(unibrow_live_status(window, state)?)
}

#[tauri::command]
pub fn unibrow_live_status(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<UnibrowLiveStatusDto, String> {
    let label = window.label().to_string();
    let mut map = live_by_window().lock().map_err(|e| e.to_string())?;
    let host = map.get_mut(&label);
    let live_mode = host.as_ref().map(|h| h.live_mode).unwrap_or(false);
    let recording = host.as_ref().map(|h| h.recording).unwrap_or(false);
    let record_path = host
        .as_ref()
        .and_then(|h| h.record_path.as_ref())
        .map(|p| p.to_string_lossy().into_owned());
    let png_base64 = if let Some(h) = host {
        Some(render_preview(h)?)
    } else {
        None
    };
    Ok(UnibrowLiveStatusDto {
        live_mode,
        recording,
        png_base64,
        record_path,
    })
}

#[tauri::command]
pub fn start_unibrow_live_recording(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<UnibrowLiveStatusDto, String> {
    let label = window.label().to_string();
    let project_path = active_unibrow_project_path(state.inner(), &label)?;
    let mut map = live_by_window().lock().map_err(|e| e.to_string())?;
    let host = map
        .get_mut(&label)
        .ok_or("enable Live mode first")?;
    if host.recording {
        return Ok(unibrow_live_status(window, state)?);
    }
    let project = PathBuf::from(&project_path);
    let derived = project.parent().unwrap_or(&project).join("derived");
    std::fs::create_dir_all(&derived).map_err(|e| e.to_string())?;
    let dest = derived.join("live-record.mp4");
    let sink = FileVideoSink::open(
        &dest,
        host.scene.width,
        host.scene.height,
        host.clock.target_fps,
    )
    .map_err(|e| e.to_string())?;
    host.sink = Some(sink);
    host.record_path = Some(dest);
    host.recording = true;
    Ok(unibrow_live_status(window, state)?)
}

#[tauri::command]
pub fn stop_unibrow_live_recording(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<UnibrowLiveStatusDto, String> {
    let label = window.label().to_string();
    let mut map = live_by_window().lock().map_err(|e| e.to_string())?;
    let host = map
        .get_mut(&label)
        .ok_or("Live mode is off")?;
    if let Some(sink) = host.sink.take() {
        let path = sink.finish().map_err(|e| e.to_string())?;
        host.record_path = Some(path);
    }
    host.recording = false;
    Ok(unibrow_live_status(window, state)?)
}

/// One-shot record without UI polling (integration helper).
#[tauri::command]
pub fn record_unibrow_live_clip(
    window: tauri::Window,
    frames: u64,
    state: State<AppState>,
) -> Result<String, String> {
    let label = window.label().to_string();
    let project_path = active_unibrow_project_path(state.inner(), &label)?;
    let root = std::path::Path::new(&project_path);
    let scene = unibrowser_project::open_package(root)
        .map(|pkg| scene_from_unibrow_manifest(&pkg.manifest, &pkg.root))
        .unwrap_or_else(|_| LiveScene::proving_full_stack(OVERLAY_SVG.to_string()));
    let mut pattern = test_pattern_source(scene.width, scene.height);
    let mut sine = sine_temperature_source();
    let mut media: HashMap<String, &mut dyn unibrowser_live::MediaSource> = HashMap::new();
    media.insert(TEST_PATTERN_ID.into(), &mut pattern);
    let mut data: HashMap<String, &mut dyn unibrowser_live::DataSource> = HashMap::new();
    data.insert(SINE_TEMP_ID.into(), &mut sine);
    let derived = PathBuf::from(&project_path)
        .parent()
        .unwrap()
        .join("derived");
    std::fs::create_dir_all(&derived).map_err(|e| e.to_string())?;
    let dest = derived.join("live-record.mp4");
    let report = run_live_session_to_file(
        &scene,
        10.0,
        frames,
        &dest,
        &mut media,
        &mut data,
    )
    .map_err(|e| e.to_string())?;
    Ok(report.output_path)
}
