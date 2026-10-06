//! Rasterize lazy timeline clips at export resolution (ADR 0011).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use unibrowser_project::{EncodeSegment, SegmentMedia};

use crate::{find_binary, MediaError};

pub fn materialize_visual_segment(
    seg: &EncodeSegment,
    width: u32,
    height: u32,
    work_dir: &Path,
    index: usize,
) -> Result<PathBuf, MediaError> {
    match seg.media {
        SegmentMedia::Video => {
            if seg.path.extension().and_then(|e| e.to_str()) != Some("mp4") {
                return Err(MediaError::Sidecar(
                    "H.264 export needs MP4 video assets".into(),
                ));
            }
            Ok(seg.path.clone())
        }
        SegmentMedia::PdfPage => rasterize_pdf_page(
            &seg.path,
            seg.pdf_page,
            seg.duration_sec,
            width,
            height,
            work_dir,
            index,
        ),
        SegmentMedia::SvgStill => {
            rasterize_svg(&seg.path, seg.duration_sec, width, height, work_dir, index)
        }
        SegmentMedia::TitleCard => rasterize_title(
            &seg.title_text,
            seg.title_font_pt,
            seg.duration_sec,
            width,
            height,
            work_dir,
            index,
        ),
        _ => Err(MediaError::Sidecar("not a visual segment".into())),
    }
}

fn rasterize_pdf_page(
    pdf: &Path,
    page: u32,
    duration_sec: f32,
    width: u32,
    height: u32,
    work_dir: &Path,
    index: usize,
) -> Result<PathBuf, MediaError> {
    fs::create_dir_all(work_dir).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if let Some(pdftoppm) = find_binary("pdftoppm") {
        let prefix = work_dir.join(format!("ub-pdf-{}-{}", index, page));
        let status = Command::new(&pdftoppm)
            .args([
                "-png",
                "-f",
                &page.to_string(),
                "-l",
                &page.to_string(),
                "-singlefile",
                "-scale-to-x",
                &width.to_string(),
                "-scale-to-y",
                &height.to_string(),
                pdf.to_string_lossy().as_ref(),
                prefix.to_string_lossy().as_ref(),
            ])
            .status()
            .map_err(|e| MediaError::Sidecar(e.to_string()))?;
        let out = prefix.with_extension("png");
        if status.success() && out.is_file() {
            return png_to_mp4(&out, duration_sec, work_dir, index, "pdf");
        }
    }
    Err(MediaError::Sidecar(
        "PDF page export needs pdftoppm on PATH (preview uses pdf.js raster proxy)".into(),
    ))
}

fn rasterize_svg(
    svg_path: &Path,
    duration_sec: f32,
    width: u32,
    height: u32,
    work_dir: &Path,
    index: usize,
) -> Result<PathBuf, MediaError> {
    fs::create_dir_all(work_dir).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let data = fs::read_to_string(svg_path).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let png = work_dir.join(format!("ub-svg-{}.png", index));
    render_svg_to_png(&data, width, height, &png)?;
    png_to_mp4(&png, duration_sec, work_dir, index, "svg")
}

fn rasterize_title(
    text: &str,
    font_pt: f32,
    duration_sec: f32,
    width: u32,
    height: u32,
    work_dir: &Path,
    index: usize,
) -> Result<PathBuf, MediaError> {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
  <rect width="100%" height="100%" fill="#14141c"/>
  <text x="50%" y="50%" text-anchor="middle" dominant-baseline="middle"
        font-family="sans-serif" font-size="{pt}" fill="#f2f2f8">{escaped}</text>
</svg>"##,
        w = width,
        h = height,
        pt = font_pt,
        escaped = escaped
    );
    fs::create_dir_all(work_dir).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let png = work_dir.join(format!("ub-title-{}.png", index));
    render_svg_to_png(&svg, width, height, &png)?;
    png_to_mp4(&png, duration_sec, work_dir, index, "title")
}

fn render_svg_to_png(svg: &str, width: u32, height: u32, dest: &Path) -> Result<(), MediaError> {
    crate::svg_raster::render_svg_to_png(svg, width, height, dest)
}

fn png_to_mp4(
    png: &Path,
    duration_sec: f32,
    work_dir: &Path,
    index: usize,
    tag: &str,
) -> Result<PathBuf, MediaError> {
    let ffmpeg = find_binary("ffmpeg").ok_or(MediaError::NotFound)?;
    let out = work_dir.join(format!("ub-{}-{}.mp4", tag, index));
    let dur = format!("{:.3}", duration_sec.max(0.1));
    let status = Command::new(&ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-loop",
            "1",
            "-i",
            png.to_string_lossy().as_ref(),
            "-t",
            &dur,
            "-vf",
            "format=yuv420p",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-an",
            "-y",
            out.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !out.is_file() {
        return Err(MediaError::Sidecar("ffmpeg still-frame video failed".into()));
    }
    Ok(out)
}

pub fn resolve_audio_path(
    seg: &EncodeSegment,
    soundfont: Option<&Path>,
    work_dir: &Path,
    index: usize,
) -> Result<PathBuf, MediaError> {
    if seg.media == SegmentMedia::Audio {
        return Ok(seg.path.clone());
    }
    if seg.media != SegmentMedia::Midi {
        return Err(MediaError::Sidecar("not an audio mix segment".into()));
    }
    let sf = soundfont.filter(|p| p.is_file()).ok_or_else(|| {
        MediaError::Sidecar("MIDI mix skipped: set a soundfont in Settings or install default.sf2".into())
    })?;
    let fluidsynth = find_binary("fluidsynth").ok_or_else(|| {
        MediaError::Sidecar("MIDI mix skipped: fluidsynth not on PATH".into())
    })?;
    fs::create_dir_all(work_dir).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let wav = work_dir.join(format!("ub-midi-{}.wav", index));
    let status = Command::new(&fluidsynth)
        .args([
            "-ni",
            sf.to_string_lossy().as_ref(),
            seg.path.to_string_lossy().as_ref(),
            "-F",
            wav.to_string_lossy().as_ref(),
            "-r",
            "44100",
        ])
        .status()
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    if !status.success() || !wav.is_file() {
        return Err(MediaError::Sidecar("fluidsynth render failed".into()));
    }
    Ok(wav)
}
