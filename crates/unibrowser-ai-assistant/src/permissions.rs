//! Per-grant Read / Write / Ask-before-acting with scopes (ADR 0026).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiPermission {
    Read,
    Write,
    Ask,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiScope {
    Global,
    Workspace { root: String },
    FileType { ext: String },
    SiteOrigin { origin: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub permission: AiPermission,
    pub scope: AiScope,
}

#[derive(Debug, Default)]
pub struct PermissionStore {
    grants: HashSet<(AiPermission, AiScope)>,
}

impl PermissionStore {
    pub fn grant(&mut self, permission: AiPermission, scope: AiScope) {
        self.grants.insert((permission, scope));
    }

    pub fn revoke(&mut self, permission: AiPermission, scope: AiScope) {
        self.grants.remove(&(permission, scope));
    }

    pub fn has_read(&self, ctx: &ActiveDocContext) -> bool {
        self.grants.iter().any(|(p, s)| {
            *p == AiPermission::Read && scope_matches(s, ctx)
        })
    }

    pub fn can_write_direct(&self, ctx: &ActiveDocContext) -> bool {
        self.grants.iter().any(|(p, s)| {
            *p == AiPermission::Write && scope_matches(s, ctx)
        })
    }

    pub fn write_mode(&self, ctx: &ActiveDocContext) -> WriteMode {
        if self.can_write_direct(ctx) {
            return WriteMode::Direct;
        }
        if self.grants.iter().any(|(p, s)| *p == AiPermission::Ask && scope_matches(s, ctx)) {
            return WriteMode::AskBeforeActing;
        }
        WriteMode::Denied
    }
}

#[derive(Debug, Clone)]
pub struct ActiveDocContext {
    pub workspace_root: Option<String>,
    pub file_ext: Option<String>,
    pub site_origin: Option<String>,
}

fn scope_matches(scope: &AiScope, ctx: &ActiveDocContext) -> bool {
    match scope {
        AiScope::Global => true,
        AiScope::Workspace { root } => ctx
            .workspace_root
            .as_ref()
            .map(|w| w == root)
            .unwrap_or(false),
        AiScope::FileType { ext } => ctx
            .file_ext
            .as_ref()
            .map(|e| e.eq_ignore_ascii_case(ext))
            .unwrap_or(false),
        AiScope::SiteOrigin { origin } => ctx
            .site_origin
            .as_ref()
            .map(|o| o == origin)
            .unwrap_or(false),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    Direct,
    AskBeforeActing,
    Denied,
}
