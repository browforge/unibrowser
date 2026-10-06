//! Local durable writes: temp file + rename, rotating slot directories.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
}

/// Write `data` atomically to `dest` (same directory temp + rename).
pub fn atomic_write(dest: &Path, data: &[u8]) -> Result<(), FsError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("tmp-write");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, dest)?;
    Ok(())
}

/// Write a numbered slot under `dir` as `slot-{n}` (1-based), rotate to keep at most `keep` files.
pub fn write_rotating_slot(dir: &Path, keep: usize, data: &[u8]) -> Result<PathBuf, FsError> {
    if keep == 0 {
        return Err(FsError::Msg("keep must be >= 1".into()));
    }
    fs::create_dir_all(dir)?;
    let mut slots: Vec<u32> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if let Some(rest) = s.strip_prefix("slot-") {
            if let Ok(n) = rest.parse::<u32>() {
                slots.push(n);
            }
        }
    }
    slots.sort_unstable();
    let next = slots.last().map(|n| n + 1).unwrap_or(1);
    let dest = dir.join(format!("slot-{}", next));
    atomic_write(&dest, data)?;
    while slots.len() + 1 > keep {
        let oldest = slots.first().copied().unwrap_or(next);
        let path = dir.join(format!("slot-{}", oldest));
        let _ = fs::remove_file(&path);
        slots.remove(0);
    }
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atomic_write_survives_interrupted_temp() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join("doc.txt");
        let tmp = dest.with_extension("tmp-write");
        fs::write(&tmp, b"partial").expect("partial");
        atomic_write(&dest, b"complete").expect("atomic");
        assert_eq!(fs::read(&dest).expect("read"), b"complete");
    }

    #[test]
    fn rotating_slots_keep_n() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("slots");
        for i in 0..5 {
            write_rotating_slot(&root, 3, format!("v{}", i).as_bytes()).expect("slot");
        }
        let mut names: Vec<String> = fs::read_dir(&root)
            .expect("read")
            .filter_map(|e| e.ok().map(|x| x.file_name().to_string_lossy().into_owned()))
            .collect();
        names.sort();
        assert_eq!(names.len(), 3);
        assert!(!names.contains(&"slot-1".into()));
        assert!(!names.contains(&"slot-2".into()));
    }
}
