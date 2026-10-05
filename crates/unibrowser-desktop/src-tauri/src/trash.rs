//! Best-effort move to Freedesktop trash; permanent delete fallback.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn trash_or_remove(path: &Path) -> Result<(), String> {
    if try_freedsktop_trash(path).is_ok() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    } else if path.is_file() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    } else {
        return Err("path does not exist".into());
    }
    Ok(())
}

fn trash_home() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg).join("Trash");
        if p.parent().map(|p| p.exists()).unwrap_or(false) {
            return Some(p);
        }
    }
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".local/share/Trash"))
}

fn try_freedsktop_trash(path: &Path) -> Result<(), String> {
    let trash = trash_home().ok_or("no trash dir")?;
    let files_dir = trash.join("files");
    let info_dir = trash.join("info");
    fs::create_dir_all(&files_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&info_dir).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or("no file name")?;
    let mut dest_name = name.clone();
    let mut dest = files_dir.join(&dest_name);
    let mut n = 0u32;
    while dest.exists() {
        n += 1;
        dest_name = format!("{}.{}", name, n);
        dest = files_dir.join(&dest_name);
    }
    if path.is_dir() {
        copy_dir_recursive(path, &dest)?;
    } else {
        fs::copy(path, &dest).map_err(|e| e.to_string())?;
    }
    let deleted = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let info_path = info_dir.join(format!("{}.trashinfo", dest_name));
    let mut info = fs::File::create(&info_path).map_err(|e| e.to_string())?;
    let body = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        path.display(),
        format_deletion_date(deleted)
    );
    info.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn format_deletion_date(secs: u64) -> String {
    let (y, m, d, h, min, s) = unix_secs_to_utc(secs);
    format!("{:04}{:02}{:02}{:02}{:02}{:02}", y, m, d, h, min, s)
}

/// Civil UTC date/time from Unix seconds (for Freedesktop trash metadata).
fn unix_secs_to_utc(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let h = (rem / 3600) as u32;
    let min = ((rem % 3600) / 60) as u32;
    let s = (rem % 60) as u32;
    let (y, m, d) = civil_from_days(days as i64);
    (y, m, d, h, min, s)
}

fn civil_from_days(z: i64) -> (u32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mp < 10 { y } else { y + 1 };
    (y as u32, m as u32, d)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = dest.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
