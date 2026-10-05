use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Pseudo-command id for opening the command palette (VS Code: Show All Commands).
pub const COMMAND_PALETTE_TOGGLE: &str = "unibrowser.palette.toggle";

/// Default chord for the command palette (Windows/Linux). macOS uses Meta+Shift+P when supported.
pub const DEFAULT_PALETTE_CHORD: &str = "Ctrl+Shift+P";
pub const DEFAULT_PALETTE_CHORD_MACOS: &str = "Cmd+Shift+P";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub key: String,
}

impl KeyChord {
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.is_empty() {
            return None;
        }
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut meta = false;
        let mut key = String::new();
        for part in parts {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "alt" | "option" => alt = true,
                "shift" => shift = true,
                "cmd" | "command" | "meta" | "super" | "win" => meta = true,
                other => key = other.to_ascii_lowercase(),
            }
        }
        if key.is_empty() {
            return None;
        }
        Some(Self {
            ctrl,
            alt,
            shift,
            meta,
            key,
        })
    }

    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Cmd");
        }
        parts.push(&self.key);
        parts.join("+")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeybindingFile {
    /// Map command id (or `unibrowser.palette.toggle`) to chord string, e.g. `Ctrl+Shift+P`.
    #[serde(default)]
    pub bindings: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct KeybindingMap {
    by_command: HashMap<String, KeyChord>,
    by_chord: HashMap<KeyChord, String>,
}

impl Default for KeybindingMap {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl KeybindingMap {
    pub fn with_defaults() -> Self {
        let mut map = Self {
            by_command: HashMap::new(),
            by_chord: HashMap::new(),
        };
        #[cfg(target_os = "macos")]
        map.set_binding(COMMAND_PALETTE_TOGGLE, DEFAULT_PALETTE_CHORD_MACOS);
        #[cfg(not(target_os = "macos"))]
        map.set_binding(COMMAND_PALETTE_TOGGLE, DEFAULT_PALETTE_CHORD);
        map.set_binding(
            unibrowser_mod_api::commands::FONT_SIZE_INCREASE,
            "Ctrl+=",
        );
        map.set_binding(
            unibrowser_mod_api::commands::FONT_SIZE_DECREASE,
            "Ctrl+-",
        );
        map.set_binding(unibrowser_mod_api::commands::TEXT_SAVE, "Ctrl+S");
        map
    }

    pub fn merge_user_file(&mut self, file: &KeybindingFile) {
        for (command, chord) in &file.bindings {
            self.set_binding(command, chord);
        }
    }

    pub fn set_binding(&mut self, command_id: &str, chord_str: &str) {
        if let Some(chord) = KeyChord::parse(chord_str) {
            if let Some(old) = self.by_command.get(command_id) {
                self.by_chord.remove(old);
            }
            self.by_command.insert(command_id.to_string(), chord.clone());
            self.by_chord.insert(chord, command_id.to_string());
        }
    }

    pub fn chord_for_command(&self, command_id: &str) -> Option<&KeyChord> {
        self.by_command.get(command_id)
    }

    pub fn command_for_chord(&self, chord: &KeyChord) -> Option<&str> {
        self.by_chord.get(chord).map(|s| s.as_str())
    }

    pub fn is_palette_chord(&self, chord: &KeyChord) -> bool {
        self.command_for_chord(chord) == Some(COMMAND_PALETTE_TOGGLE)
    }

    pub fn palette_chord_display(&self) -> String {
        self.chord_for_command(COMMAND_PALETTE_TOGGLE)
            .map(|c| c.display())
            .unwrap_or_else(|| DEFAULT_PALETTE_CHORD.to_string())
    }

    /// Load engine defaults and merge `keybindings.json` when the path exists.
    pub fn load_merged(user_file: Option<&Path>) -> Self {
        let mut map = Self::with_defaults();
        if let Some(path) = user_file {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(file) = serde_json::from_str::<KeybindingFile>(&data) {
                    map.merge_user_file(&file);
                }
            }
        }
        map
    }

    pub fn bindings_for_display(&self) -> Vec<(String, String)> {
        let mut out: Vec<_> = self
            .by_command
            .iter()
            .map(|(cmd, chord)| (cmd.clone(), chord.display()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    pub fn matches_modifiers_and_key(
        &self,
        chord: &KeyChord,
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
        key: &str,
    ) -> bool {
        chord.ctrl == ctrl
            && chord.alt == alt
            && chord.shift == shift
            && chord.meta == meta
            && chord.key == key.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ctrl_shift_p() {
        let c = KeyChord::parse("Ctrl+Shift+P").expect("parse");
        assert!(c.ctrl && c.shift && c.key == "p");
    }

    #[test]
    fn palette_default_binding() {
        let map = KeybindingMap::with_defaults();
        assert!(map.is_palette_chord(
            &KeyChord::parse("Ctrl+Shift+P").expect("parse")
        ));
    }

    #[test]
    fn user_override_replaces_palette() {
        let mut map = KeybindingMap::with_defaults();
        map.set_binding(COMMAND_PALETTE_TOGGLE, "Ctrl+K");
        assert!(map.is_palette_chord(&KeyChord::parse("Ctrl+K").expect("parse")));
    }
}
