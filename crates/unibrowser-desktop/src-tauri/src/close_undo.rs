//! Last closed tab with optional autosave payload for toast Undo (no modal close).

use serde::{Deserialize, Serialize};
use unibrowser_mod_api::BrowserLocation;

use crate::autosave::AutosaveSlotRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseUndoEntry {
    pub location: BrowserLocation,
    pub insert_index: usize,
    pub autosave: Option<AutosaveSlotRecord>,
}

#[derive(Default)]
pub struct CloseUndoStack {
    last: Option<CloseUndoEntry>,
}

impl CloseUndoStack {
    pub fn push(&mut self, entry: CloseUndoEntry) {
        self.last = Some(entry);
    }

    pub fn take(&mut self) -> Option<CloseUndoEntry> {
        self.last.take()
    }

    pub fn peek_available(&self) -> bool {
        self.last.is_some()
    }
}
