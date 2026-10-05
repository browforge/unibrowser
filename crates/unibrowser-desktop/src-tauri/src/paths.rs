//! Well-known user folders (Downloads, etc.).

use std::path::PathBuf;

/// User Downloads directory (Windows Known Folder, XDG on Linux, ~/Downloads fallback).
pub fn user_downloads_dir() -> PathBuf {
    if let Some(dir) = dirs::download_dir() {
        return dir;
    }
    home_dir().join("Downloads")
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downloads_dir_is_absolute() {
        let dir = user_downloads_dir();
        assert!(dir.is_absolute());
    }
}
