//! Directory listing filter and sort (global prefs).

use base_files::DirEntry;
use crate::preferences::{FilesSortMode, FilesViewPrefs};

pub fn apply_files_view(entries: Vec<DirEntry>, prefs: &FilesViewPrefs) -> Vec<DirEntry> {
    let mut entries = entries;
    if !prefs.show_hidden {
        entries.retain(|e| !e.name.starts_with('.'));
    }
    entries.sort_by(|a, b| {
        let dir_order = b.is_dir.cmp(&a.is_dir);
        if dir_order != std::cmp::Ordering::Equal {
            return dir_order;
        }
        let cmp = match prefs.sort {
            FilesSortMode::Name => a
                .name
                .to_lowercase()
                .cmp(&b.name.to_lowercase()),
            FilesSortMode::Date => a.modified_unix.cmp(&b.modified_unix),
            FilesSortMode::Size => a.size_bytes.cmp(&b.size_bytes),
        };
        if prefs.sort_ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_files::DirEntry;
    use crate::preferences::{FilesSortMode, FilesViewPrefs};
    use std::path::PathBuf;

    fn entry(name: &str, dir: bool, size: u64, modified: i64) -> DirEntry {
        DirEntry {
            name: name.into(),
            path: PathBuf::from(format!("/tmp/{}", name)),
            is_dir: dir,
            size_bytes: size,
            modified_unix: modified,
        }
    }

    #[test]
    fn hides_dotfiles_by_default() {
        let entries = vec![
            entry(".hidden", false, 1, 0),
            entry("visible.txt", false, 1, 0),
        ];
        let out = apply_files_view(entries, &FilesViewPrefs::default());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "visible.txt");
    }

    #[test]
    fn sorts_by_size_descending() {
        let prefs = FilesViewPrefs {
            sort: FilesSortMode::Size,
            sort_ascending: false,
            show_hidden: true,
        };
        let entries = vec![
            entry("small", false, 10, 0),
            entry("big", false, 100, 0),
        ];
        let out = apply_files_view(entries, &prefs);
        assert_eq!(out[0].name, "big");
    }
}
