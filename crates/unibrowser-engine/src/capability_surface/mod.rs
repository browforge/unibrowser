//! Runtime registry for document surface capability providers (SKEL-01).

mod registry;
mod routing;
mod resolve;

pub use registry::{
    CapabilityDefaultConflict, SurfaceProviderRecord, SurfaceProviderRegistry,
    build_surface_provider_registry,
};
pub use resolve::{CapabilityPrefs, ResolvedSurfaceProvider, resolve_surface_provider};
pub use routing::capability_for_effective_extension;
