use std::path::PathBuf;

/// `keybindings.json` under the per-user config dir (VS Code–style location).
pub fn keybindings_file_path() -> Option<PathBuf> {
    let base = config_dir()?;
    Some(base.join("keybindings.json"))
}

pub fn toolbars_file_path() -> Option<PathBuf> {
    let base = config_dir()?;
    Some(base.join("toolbars.json"))
}

fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return std::env::var("APPDATA")
            .ok()
            .map(|a| PathBuf::from(a).join("unibrowser"));
    }
    #[cfg(target_os = "macos")]
    {
        return std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library/Application Support/unibrowser"));
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let base = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })?;
        return Some(base.join("unibrowser"));
    }
}
