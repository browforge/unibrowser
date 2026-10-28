//! Diagnostics host (ADR 0021): problem list, locators, LSP client, dictionary store.

mod code_regions;
mod markdown_prose;
mod dictionary;
mod host;
mod locator;
mod navigate;
mod lsp;
mod native;
mod problem;
mod web_fields;

pub use code_regions::{code_check_segments, CodeSegment, CodeSegmentKind};
pub use markdown_prose::markdown_prose_ranges;
pub use dictionary::{DictionarySnapshot, DictionaryStore, IgnoreScope};
pub use host::{DiagnosticsHost, TextCheckContext};
pub use locator::Locator;
pub use navigate::{NavigationTarget, navigation_target};
pub use lsp::client::{LspClient, LspServerSpec};
pub use native::NativeDiagnosticsChecker;
pub use problem::{CodeAction, Problem, Severity};
pub use web_fields::{classify_field, field_may_proof, WebFieldKind, WebFieldProbe};
