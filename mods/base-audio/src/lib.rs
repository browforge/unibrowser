use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, CommandMeta, Enablement, Mode, SelectionSnapshot,
    handlers::PICTURE_IN_PICTURE_ID,
};

pub const BACKEND_ID: &str = "base-audio";
pub const VIEWER_KIND: &str = "audio";

pub const CMD_SPLIT_CLIP: &str = "audio.splitClip";
pub const CMD_EXPORT_STEM: &str = "audio.exportStem";
pub const CMD_AUTOMATE: &str = "audio.automation";

pub struct AudioBackend {
    pub path: String,
    pub probe_summary: String,
}

impl AudioBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let path_str = path.to_string_lossy().into_owned();
        let probe_summary = match unibrowser_media::probe_media_file(path) {
            Ok(p) => p.ffprobe_summary.unwrap_or_else(|| "probed".into()),
            Err(e) => format!("no ffprobe: {}", e),
        };
        Ok(Self {
            path: path_str,
            probe_summary,
        })
    }
}

impl unibrowser_engine::document::DocumentBackend for AudioBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "audio/*"
    }
    fn display_name(&self) -> &str {
        "Audio"
    }
    fn mode(&self) -> Mode {
        Mode::Browse
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!("Audio · {} · {}", self.path, self.probe_summary)
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn daw_tool_disabled(_doc: &DocumentSession) -> Enablement {
    Enablement::Disabled {
        reason: "Proving viewer: needs a DAW / multitrack engine (not bundled)".into(),
    }
}

fn daw_tool_stub(_doc: &mut DocumentSession) -> Result<String, String> {
    Err("DAW tool not available in audio proving viewer".into())
}

pub fn register(registry: &mut CommandRegistry) {
    for (id, title) in [
        (CMD_SPLIT_CLIP, "Split clip (DAW)"),
        (CMD_EXPORT_STEM, "Export stem (DAW)"),
        (CMD_AUTOMATE, "Volume automation (DAW)"),
    ] {
        registry.register_command(CommandMeta {
            id,
            title,
            icon: "extension",
            default_keybinding: None,
            aliases: &[],
        });
        registry.register_handler(HandlerRegistration {
            command_id: id.into(),
            backend_id: BACKEND_ID.into(),
            enable: daw_tool_disabled,
            execute: daw_tool_stub,
        });
    }
    registry.register_handler(HandlerRegistration {
        command_id: PICTURE_IN_PICTURE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: pip_enable,
        execute: pip_exec,
    });
}

fn pip_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() == BACKEND_ID {
        Enablement::Enabled
    } else {
        Enablement::Disabled {
            reason: "Picture-in-picture needs an audio tab".into(),
        }
    }
}

fn pip_exec(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("media-pip".into())
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(AudioBackend::from_path(path)?))
}
