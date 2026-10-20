//! Stable mod API for Unibrowser. Semver bumps follow ADR 0006.

pub const MOD_API_VERSION: &str = "0.1.0";

pub mod capability_surface;
pub mod command_system;
pub mod content_kind;
pub mod content_tree;
pub mod contributes;
pub mod context;
pub mod handlers;
pub mod manifest;
pub mod permission_grants;
pub mod permissions;
pub mod navigation;
pub mod projections;
pub mod runtime;
pub mod types;

pub use capability_surface::{
    CapabilityProvideEntry, CAPABILITY_SCHEMA_VERSION, validate_capability_id,
    validate_capability_provide,
};
pub use content_kind::{
    ContentKindRecord, ContentKindRegistry, CONTENT_KIND_SCHEMA_VERSION,
    validate_content_kind_id,
};
pub use content_tree::{
    ContentKindNode, DocumentContentTree, CONTENT_TREE_SCHEMA_VERSION,
};
pub use command_system::*;
pub use contributes::{
    CommandLayoutFileV2, ContextLayout, ContributedCapabilityProvider,
    ContributedControl, ContributedFamily, ContributedHostFormatCapability, ContributedPanel,
    ContributedService, ContributedStatusItem, ContributedToolbar, ContributedView,
    ContributionValidationError, HiddenControlEntry, InsertPoint, LAYOUT_VERSION_V2,
    ModContributes, MonitorAnchor, ToolbarLayoutEntry, validate_contributes,
};
pub use context::CommandContext;
pub use manifest::ModManifest;
pub use permission_grants::{
    GrantChoice, GrantRecord, GrantStoreFile, PermissionGrantStore, PluginGrantState,
};
pub use permissions::{
    permissions_fingerprint, validate_network_host_entry, validate_permissions, ModPermissionsBlock,
    NetworkPermissions, PERMISSION_SCHEMA_VERSION,
};
pub use projections::{RasterProjection, RasterProjectionSource, TextProjection};
pub use navigation::{BrowserLocation, DownloadDisposition, TabId};
pub use runtime::{
    ControlRuntimeSnapshot, FamilyRuntimeSnapshot, HostEditRequest, ModPermissions, ModRuntimeDescriptor,
    RuntimeCallbackError, RuntimeCapabilityKind, MOD_CAPABILITY_TIMEOUT, MOD_EXECUTION_TIMEOUT,
    MOD_RUNTIME_API_V1,
};
pub use types::*;
