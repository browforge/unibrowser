use std::path::Path;

use unibrowser_engine::{CommandRegistry, HandlerRegistration, document::DocumentSession};
use unibrowser_mod_api::{
    Capabilities, Enablement, Mode, SelectionSnapshot,
    handlers::PICTURE_IN_PICTURE_ID,
};

pub const BACKEND_ID: &str = "base-video";
pub const VIEWER_KIND: &str = "video";

pub struct VideoBackend {
    pub path: String,
    pub probe_summary: String,
}

impl VideoBackend {
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

impl unibrowser_engine::document::DocumentBackend for VideoBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "video/*"
    }
    fn display_name(&self) -> &str {
        "Video"
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
        format!("Video · {} · {}", self.path, self.probe_summary)
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn pip_enable(doc: &DocumentSession) -> Enablement {
    if doc.backend.backend_id() == BACKEND_ID {
        Enablement::Enabled
    } else {
        Enablement::Disabled {
            reason: "Picture-in-picture needs a video tab".into(),
        }
    }
}

fn pip_exec(_doc: &mut DocumentSession) -> Result<String, String> {
    Ok("media-pip".into())
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register_handler(HandlerRegistration {
        command_id: PICTURE_IN_PICTURE_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: pip_enable,
        execute: pip_exec,
    });
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(VideoBackend::from_path(path)?))
}
