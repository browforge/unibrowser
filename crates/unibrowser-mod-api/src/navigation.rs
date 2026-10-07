//! Tab locations and download behaviour for the unified file + web browser shell (ADR 0009).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Where a tab points: local file, remote URL, or internal chrome page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserLocation {
    /// `file:///path/to/project/readme.md`
    File { path: PathBuf },
    /// `https://example.com/doc.pdf`
    Http { url: String },
    /// `unibrowser://settings`, mod manager, about, etc.
    Internal { page: String },
}

impl BrowserLocation {
    pub fn display_label(&self) -> String {
        match self {
            BrowserLocation::File { path } => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            BrowserLocation::Http { url } => url.clone(),
            BrowserLocation::Internal { page } => {
                if page == "home" {
                    "New tab".into()
                } else if page.starts_with("mods") {
                    "Mod portal".into()
                } else {
                    page.clone()
                }
            }
        }
    }

    pub fn is_web(&self) -> bool {
        matches!(self, BrowserLocation::Http { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, BrowserLocation::File { .. })
    }
}

/// Stable tab identity (session layer assigns these).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub u64);

/// What to do when the user or web content finishes a download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadDisposition {
    /// Default: open in a new tab (view/edit with mods).
    OpenInNewTab { suggested_location: BrowserLocation },
    /// User explicitly chose save; path chosen by save dialog.
    SaveToDisk { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_location_label_uses_basename() {
        let loc = BrowserLocation::File {
            path: PathBuf::from("/home/bear/notes.txt"),
        };
        assert_eq!(loc.display_label(), "notes.txt");
    }

    #[test]
    fn mod_portal_internal_label() {
        let loc = BrowserLocation::Internal {
            page: "mods?ext=ubunknown".into(),
        };
        assert_eq!(loc.display_label(), "Mod portal");
    }
}
