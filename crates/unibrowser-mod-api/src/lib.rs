//! Stable mod API for Unibrowser. Semver bumps follow ADR 0006.

pub const MOD_API_VERSION: &str = "0.1.0";

pub mod command_system;
pub mod contributes;
pub mod context;
pub mod handlers;
pub mod manifest;
pub mod navigation;
pub mod projections;
pub mod runtime;
pub mod types;

pub use command_system::*;
pub use contributes::{
    CommandLayoutFileV2, ContextLayout, ContributedControl, ContributedFamily,
    ContributionValidationError, HiddenControlEntry, InsertPoint, LAYOUT_VERSION_V2,
    ModContributes, MonitorAnchor, ToolbarLayoutEntry, validate_contributes,
};
pub use context::CommandContext;
pub use manifest::ModManifest;
pub use projections::{RasterProjection, RasterProjectionSource, TextProjection};
pub use navigation::{BrowserLocation, DownloadDisposition, TabId};
pub use runtime::{
    ControlRuntimeSnapshot, FamilyRuntimeSnapshot, HostEditRequest, ModPermissions, ModRuntimeDescriptor,
    RuntimeCallbackError, RuntimeCapabilityKind, MOD_CAPABILITY_TIMEOUT, MOD_EXECUTION_TIMEOUT,
    MOD_RUNTIME_API_V1,
};
pub use types::*;
