use std::path::Path;

use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "base-audio";

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

pub fn register(_registry: &mut CommandRegistry) {}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(AudioBackend::from_path(path)?))
}
