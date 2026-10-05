//! Command registry hooks so `.unibrow` tabs expose undo/redo in the Command Manager.

use unibrowser_engine::{CommandRegistry, HandlerRegistration};
use unibrowser_mod_api::Enablement;
use unibrowser_mod_api::handlers::{DOCUMENT_REDO_ID, DOCUMENT_UNDO_ID};

use crate::project_backend::BACKEND_ID;

fn undo_redo_enable(doc: &unibrowser_engine::document::DocumentSession) -> Enablement {
    if doc.backend.backend_id() != BACKEND_ID {
        return Enablement::Disabled {
            reason: "Wrong backend".into(),
        };
    }
    Enablement::Enabled
}

pub fn register_handlers(registry: &mut CommandRegistry) {
    registry.register_handler(HandlerRegistration {
        command_id: DOCUMENT_UNDO_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: undo_redo_enable,
        execute: |_doc| Ok("document-undo".into()),
    });
    registry.register_handler(HandlerRegistration {
        command_id: DOCUMENT_REDO_ID.into(),
        backend_id: BACKEND_ID.into(),
        enable: undo_redo_enable,
        execute: |_doc| Ok("document-redo".into()),
    });
}
