//! Assistant session: projections, grants, pending writes (ADR 0026).

use thiserror::Error;
use unibrowser_mod_api::TextProjection;

use crate::permissions::{
    ActiveDocContext, AiPermission, AiScope, PermissionStore, WriteMode,
};
use crate::redact::redact_sensitive_projection;

#[derive(Debug, Error)]
pub enum AssistantError {
    #[error("read not granted for active document")]
    ReadDenied,
    #[error("write not granted for active document")]
    WriteDenied,
    #[error("user has not invoked the assistant")]
    NotInvoked,
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub id: u64,
    pub summary: String,
    pub replacement_text: String,
}

#[derive(Debug)]
pub struct AssistantSession {
    permissions: PermissionStore,
    invoked: bool,
    next_pending_id: u64,
    pending: Vec<PendingAction>,
}

impl AssistantSession {
    pub fn new(permissions: PermissionStore) -> Self {
        Self {
            permissions,
            invoked: false,
            next_pending_id: 1,
            pending: Vec::new(),
        }
    }

    pub fn permissions(&self) -> &PermissionStore {
        &self.permissions
    }

    pub fn permissions_mut(&mut self) -> &mut PermissionStore {
        &mut self.permissions
    }

    /// Nothing is sent to a model until the user explicitly invokes the assistant.
    pub fn user_invoke(&mut self) {
        self.invoked = true;
    }

    pub fn can_see_active_tab(&self, ctx: &ActiveDocContext) -> bool {
        self.permissions.has_read(ctx)
    }

    pub fn read_active_projection<T: TextProjection>(
        &self,
        doc: &T,
        ctx: &ActiveDocContext,
    ) -> Result<String, AssistantError> {
        if !self.invoked {
            return Err(AssistantError::NotInvoked);
        }
        if !self.permissions.has_read(ctx) {
            return Err(AssistantError::ReadDenied);
        }
        Ok(redact_sensitive_projection(&doc.text_projection()))
    }

    pub fn propose_write(
        &mut self,
        ctx: &ActiveDocContext,
        summary: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Result<Option<u64>, AssistantError> {
        if !self.invoked {
            return Err(AssistantError::NotInvoked);
        }
        match self.permissions.write_mode(ctx) {
            WriteMode::Direct => Ok(None),
            WriteMode::AskBeforeActing => {
                let id = self.next_pending_id;
                self.next_pending_id += 1;
                self.pending.push(PendingAction {
                    id,
                    summary: summary.into(),
                    replacement_text: replacement.into(),
                });
                Ok(Some(id))
            }
            WriteMode::Denied => Err(AssistantError::WriteDenied),
        }
    }

    pub fn pending_actions(&self) -> &[PendingAction] {
        &self.pending
    }

    pub fn approve_pending(&mut self, id: u64) -> Option<String> {
        let idx = self.pending.iter().position(|p| p.id == id)?;
        let action = self.pending.remove(idx);
        Some(action.replacement_text)
    }
}

/// Convenience for tests and mods.
pub fn default_read_grant(store: &mut PermissionStore) {
    store.grant(AiPermission::Read, AiScope::Global);
}
