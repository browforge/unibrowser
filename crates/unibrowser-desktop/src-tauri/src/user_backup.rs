//! Export/import preferences, keybindings, and command-manager toolbars (grandma backup).

use serde::{Deserialize, Serialize};

use crate::config::{keybindings_file_path, toolbars_file_path};
use crate::preferences::{load_preferences, save_preferences, Preferences};
use unibrowser_engine::{load_command_manager_file, save_command_manager_file, CommandManagerFile};

pub const BACKUP_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandmaBackupV1 {
    pub version: u32,
    pub preferences: Preferences,
    pub keybindings: serde_json::Value,
    pub toolbars: CommandManagerFile,
}

pub fn build_backup() -> Result<GrandmaBackupV1, String> {
    let prefs = load_preferences();
    let kb_path = keybindings_file_path().ok_or("config dir unavailable")?;
    let keybindings = if kb_path.is_file() {
        let raw = std::fs::read_to_string(&kb_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({})
    };
    let toolbars = load_command_manager_file(toolbars_file_path().as_deref());
    Ok(GrandmaBackupV1 {
        version: BACKUP_VERSION,
        preferences: prefs,
        keybindings,
        toolbars,
    })
}

pub fn apply_backup(backup: &GrandmaBackupV1) -> Result<(), String> {
    if backup.version != BACKUP_VERSION {
        return Err(format!("unsupported backup version {}", backup.version));
    }
    save_preferences(&backup.preferences)?;
    if let Some(kb_path) = keybindings_file_path() {
        if let Some(parent) = kb_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(&backup.keybindings).map_err(|e| e.to_string())?;
        std::fs::write(&kb_path, data).map_err(|e| e.to_string())?;
    }
    if let Some(tb_path) = toolbars_file_path() {
        save_command_manager_file(&tb_path, &backup.toolbars)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_roundtrip_serde() {
        let b = GrandmaBackupV1 {
            version: BACKUP_VERSION,
            preferences: Preferences::default(),
            keybindings: serde_json::json!({ "bindings": [] }),
            toolbars: CommandManagerFile::factory_default(),
        };
        let json = serde_json::to_string(&b).expect("json");
        let back: GrandmaBackupV1 = serde_json::from_str(&json).expect("parse");
        assert_eq!(back.version, BACKUP_VERSION);
    }
}
