use unibrowser_mod_api::Mode;

use crate::document::DocumentSession;

/// Derives active mode from document and optional user pin (pin not implemented in skeleton).
#[derive(Debug, Clone)]
pub struct ModeController {
    pinned: Option<Mode>,
}

impl Default for ModeController {
    fn default() -> Self {
        Self { pinned: None }
    }
}

impl ModeController {
    pub fn active_mode(&self, doc: &DocumentSession) -> Mode {
        self.pinned.unwrap_or_else(|| doc.backend.mode())
    }

    pub fn tab_for_mode(mode: Mode) -> &'static str {
        mode.tab_id()
    }
}
