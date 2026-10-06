//! Shared SVG → RGBA rasterization (timeline export + live compositor).

use std::path::Path;

use crate::MediaError;

pub fn rasterize_svg_to_rgba(svg: &str, width: u32, height: u32) -> Result<Vec<u8>, MediaError> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| MediaError::Sidecar("pixmap alloc failed".into()))?;
    let scale_x = width as f32 / tree.size().width();
    let scale_y = height as f32 / tree.size().height();
    let scale = scale_x.min(scale_y);
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Ok(pixmap.data().to_vec())
}

pub fn rasterize_svg_file_to_rgba(path: &Path, width: u32, height: u32) -> Result<Vec<u8>, MediaError> {
    let data = std::fs::read_to_string(path).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    rasterize_svg_to_rgba(&data, width, height)
}

pub fn render_svg_to_png(svg: &str, width: u32, height: u32, dest: &Path) -> Result<(), MediaError> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| MediaError::Sidecar(e.to_string()))?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| MediaError::Sidecar("pixmap alloc failed".into()))?;
    let scale_x = width as f32 / tree.size().width();
    let scale_y = height as f32 / tree.size().height();
    let scale = scale_x.min(scale_y);
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap
        .save_png(dest)
        .map_err(|e| MediaError::Sidecar(e.to_string()))?;
    Ok(())
}
