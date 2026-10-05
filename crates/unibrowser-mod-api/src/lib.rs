//! Stable mod API for Unibrowser. Semver bumps follow ADR 0006.

pub const MOD_API_VERSION: &str = "0.1.0";

pub mod handlers;
pub mod manifest;
pub mod navigation;
pub mod types;

pub use manifest::ModManifest;
pub use navigation::{BrowserLocation, DownloadDisposition, TabId};
pub use types::*;
