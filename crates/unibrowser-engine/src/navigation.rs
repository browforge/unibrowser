use std::path::{Path, PathBuf};

use unibrowser_mod_api::BrowserLocation;

/// Parse omnibox / user input into a location.
pub fn parse_user_input(input: &str, cwd: &Path) -> BrowserLocation {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return BrowserLocation::Internal {
            page: "newtab".into(),
        };
    }
    if trimmed.starts_with("unibrowser://") {
        return BrowserLocation::Internal {
            page: trimmed.trim_start_matches("unibrowser://").into(),
        };
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return BrowserLocation::Http {
            url: trimmed.into(),
        };
    }
    if let Some(rest) = trimmed.strip_prefix("file://") {
        return BrowserLocation::File {
            path: PathBuf::from(rest),
        };
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return BrowserLocation::File { path };
    }
    BrowserLocation::File {
        path: cwd.join(path),
    }
}

pub fn is_http(location: &BrowserLocation) -> bool {
    matches!(location, BrowserLocation::Http { .. })
}

pub fn file_path(location: &BrowserLocation) -> Option<&Path> {
    match location {
        BrowserLocation::File { path } => Some(path),
        _ => None,
    }
}
