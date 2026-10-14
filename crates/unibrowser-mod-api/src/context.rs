use serde::{Deserialize, Serialize};

use crate::Mode;

/// Command Manager tab / rollout lane (text → office → code → image → audio).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandContext {
    Start,
    Browse,
    Text,
    Office,
    Code,
    Image,
    Audio,
    Web,
}

impl CommandContext {
    pub fn tab_id(self) -> &'static str {
        match self {
            CommandContext::Start => "start",
            CommandContext::Browse => "browse",
            CommandContext::Text => "text",
            CommandContext::Office => "office",
            CommandContext::Code => "code",
            CommandContext::Image => "image",
            CommandContext::Audio => "audio",
            CommandContext::Web => "web",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CommandContext::Start => "Start",
            CommandContext::Browse => "Browse",
            CommandContext::Text => "Text",
            CommandContext::Office => "Office",
            CommandContext::Code => "Code",
            CommandContext::Image => "Image",
            CommandContext::Audio => "Audio",
            CommandContext::Web => "Web",
        }
    }

    /// Product rollout order for default tab strip.
    pub fn rollout_tabs() -> &'static [CommandContext] {
        &[
            CommandContext::Text,
            CommandContext::Office,
            CommandContext::Code,
            CommandContext::Image,
            CommandContext::Audio,
        ]
    }

    pub fn from_mode_and_path(mode: Mode, path: &str) -> Self {
        if path.contains("/assets/document.md") {
            return CommandContext::Office;
        }
        match mode {
            Mode::Browse if is_media_path(path) => CommandContext::Audio,
            Mode::Browse => CommandContext::Browse,
            Mode::Web => CommandContext::Web,
            Mode::Image | Mode::Vector => CommandContext::Image,
            Mode::Document => CommandContext::Office,
            Mode::Text => {
                if is_code_path(path) {
                    CommandContext::Code
                } else if is_office_path(path) {
                    CommandContext::Office
                } else {
                    CommandContext::Text
                }
            }
        }
    }
}

fn is_code_path(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "rb"
            | "swift"
            | "kt"
            | "sh"
            | "bash"
            | "zsh"
            | "json"
            | "toml"
            | "yaml"
            | "yml"
    )
}

fn is_office_path(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "doc" | "docx" | "odt" | "xls" | "xlsx" | "ods" | "ppt" | "pptx" | "odp"
    )
}

fn is_media_path(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "aac"
            | "flac"
            | "m4a"
            | "mkv"
            | "mov"
            | "mp3"
            | "mp4"
            | "ogg"
            | "wav"
            | "webm"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browse_mode_uses_audio_context_for_media_files() {
        assert_eq!(
            CommandContext::from_mode_and_path(Mode::Browse, "clip.mp4"),
            CommandContext::Audio,
        );
        assert_eq!(
            CommandContext::from_mode_and_path(Mode::Browse, "sample.wav"),
            CommandContext::Audio,
        );
    }
}
