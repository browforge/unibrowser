//! Batch image format conversion (skeleton) for Files multi-select.

use std::path::{Path, PathBuf};

use image::ImageFormat;

const IMAGE_EXT: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];

pub fn is_convertible_image(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    IMAGE_EXT.contains(&ext.as_str())
}

fn target_format(name: &str) -> Result<ImageFormat, String> {
    match name.to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        _ => Err("format must be png or jpeg".into()),
    }
}

fn output_path_for(source: &Path, format: ImageFormat) -> PathBuf {
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".into());
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let ext = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => "img",
    };
    let candidate = parent.join(format!("{}.{}", stem, ext));
    if candidate == source {
        return parent.join(format!("{}-converted.{}", stem, ext));
    }
    if candidate.exists() {
        return parent.join(format!("{}-converted.{}", stem, ext));
    }
    candidate
}

pub fn batch_convert_images(paths: &[String], format: &str) -> Result<Vec<String>, String> {
    let fmt = target_format(format)?;
    let mut written = Vec::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if !path.is_file() {
            return Err(format!("not a file: {}", path.display()));
        }
        if !is_convertible_image(&path) {
            return Err(format!("not a supported image: {}", path.display()));
        }
        let img = image::open(&path).map_err(|e| e.to_string())?;
        let dest = output_path_for(&path, fmt);
        img.save_with_format(&dest, fmt).map_err(|e| e.to_string())?;
        written.push(dest.to_string_lossy().into_owned());
    }
    Ok(written)
}
