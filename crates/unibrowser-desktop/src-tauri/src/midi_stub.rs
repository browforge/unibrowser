//! MIDI file shell backend (native SMF metadata until encode lane).

use std::path::Path;

use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

use crate::smf::{NoteOnEvent, SmfSummary};

pub const BACKEND_ID: &str = "base-midi-stub";
pub const VIEWER_KIND: &str = "midi";

pub struct MidiStubBackend {
    pub path: String,
    pub summary: SmfSummary,
    pub note_preview: Vec<NoteOnEvent>,
}

impl MidiStubBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let summary = crate::smf::read_smf_summary(path)?;
        let note_preview = crate::smf::read_note_on_events(path, 32).unwrap_or_default();
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            summary,
            note_preview,
        })
    }
}

impl unibrowser_engine::document::DocumentBackend for MidiStubBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "audio/midi"
    }
    fn display_name(&self) -> &str {
        "MIDI"
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
    fn status_line(&self) -> String {
        format!(
            "MIDI format {} · {} tracks · {} ticks/qn",
            self.summary.format,
            self.summary.track_count,
            self.summary.ticks_per_quarter
        )
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(MidiStubBackend::from_path(path)?))
}

pub fn try_midi_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut MidiStubBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<MidiStubBackend>()
}
